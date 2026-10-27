# extra::ely

Ely.by integration: authentication, `authlib-injector` patch, and a local beta skin proxy.

Ely.by is a third-party authentication service for Minecraft that emulates the Mojang auth API. It has two ways of working with the game:

- **`authlib-injector`** — a JVM agent injected into the game process that intercepts auth calls and redirects them to Ely.by. Used for all modern versions.
- **Beta skin proxy** — a local HTTP proxy that serves Ely.by skins to old beta clients that request `/MinecraftSkins/<name>` directly. Used only for old beta versions.

This module gives you both, plus a login helper for the Ely.by auth endpoint.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core", "ely"] }
```

```rust
use rmlib::extra::ely::{login, install_patch, start_beta_proxy, AuthResult, ProxyGuard};
```

## `AuthResult`

```rust
#[derive(Debug, Serialize)]
pub struct AuthResult {
    pub access_token: String,
    pub client_token: String,
    pub profile_id: String,
    pub profile_name: String,
}
```

The result of a successful Ely.by login. Derives `Debug`, `Serialize` (so it can be stored as JSON directly).

| Field | Meaning |
|---|---|
| `access_token` | The token to pass as `-Dhttp.proxy…` / the game's access token. |
| `client_token` | The client token Ely.by assigned for this session. |
| `profile_id` | The Ely.by profile UUID. |
| `profile_name` | The profile display name (the Minecraft username). |

The values map directly to the `--uuid` / `--token` / `-u` parameters you would pass to `core::run::run_client`.

## `login`

```rust
pub async fn login(
    login: String,
    passwd: String,
) -> Result<AuthResult, Box<dyn std::error::Error>>
```

Authenticates against `https://authserver.ely.by/auth/authenticate`.

### Parameters

| Parameter | Meaning |
|---|---|
| `login` | Ely.by username or email. |
| `passwd` | Ely.by password. |

### Behaviour

1. Sends a JSON POST to `authserver.ely.by/auth/authenticate` with the launcher's hard-coded client token (`93301754-cd11-4a00-bd50-f69dffaf4b0b`) and `requestUser: true`.
2. Parses the response into `AuthApiResponse` (internal).
3. Extracts `selected_profile` from the response. If it is missing → `Err("Selected profile not found in response")`.
4. Returns `AuthResult` with the access token, client token, profile UUID and profile name.

### Errors

Returns `Err` for:

- Network failure (from `reqwest`).
- Invalid JSON from Ely.by (`reqwest::Error` from `.json::<AuthApiResponse>()`).
- Missing `selectedProfile` field.

### Example

```rust
use rmlib::extra::ely::login;

let result = login("PlayerName".into(), "hunter2".into()).await?;
println!("logged in as {} ({})", result.profile_name, result.profile_id);
println!("token = {}", result.access_token);
```

Store the token; it is what you later pass to `core::run::run_client` as the `token` parameter (and `profile_id` as `uuid`, `profile_name` as `username`).

## `install_patch`

```rust
pub async fn install_patch(
    libs_path_option: Option<PathBuf>,
    is_json: bool,
) -> std::io::Result<()>
```

Downloads the latest `authlib-injector.jar` from GitHub into `<libs>/authlib-injector.jar`.

The JAR is what `core::run` adds as `-javaagent:<path>=ely.by` when `ely = true` — see [Arguments](../core/run/arguments.md#build_arguments).

### Parameters

| Parameter | Meaning |
|---|---|
| `libs_path_option` | Override the libraries directory. `None` uses `core::config::get_libs_path()`. |
| `is_json` | If `true`, suppresses progress printing. Use `true` when the caller is producing machine-readable output. |

### Behaviour

1. Resolves the libs path (tilde-expanded).
2. Fetches `https://api.github.com/repos/yushijinhun/authlib-injector/releases` (uses `get_github_json_str`).
3. Picks the first release, finds its `.jar` asset, and downloads it to `<libs>/authlib-injector.jar`.
4. Creates the libs directory if missing.

### Errors

Returns `Err(std::io::Error)` for:

- Network / HTTP errors from GitHub.
- Invalid JSON in the release response.
- Missing release / missing JAR asset.
- Download failure.

The error messages are best-effort and may be wrapped from lower layers (`HttpError`, `serde_json::Error`) as `std::io::Error`.

### Example

```rust
use rmlib::extra::ely::install_patch;

install_patch(None, false).await?;
// now run_client(..., ely = true, ...) will find authlib-injector.jar
```

## `start_beta_proxy`

```rust
pub fn start_beta_proxy(port: u16) -> Option<ProxyGuard>
```

Starts a local HTTP server that forwards legacy skin requests to Ely.by.

Old beta clients (pre-1.6) request skins by hitting `http://<host>:<port>/MinecraftSkins/<username>`. This proxy intercepts those requests, fetches the skin from `https://skinsystem.ely.by/skins/<username>.png`, and returns it as `image/png`.

### Parameters

| Parameter | Meaning |
|---|---|
| `port` | The TCP port to listen on (bound on `127.0.0.1`). |

### Returns

- `Some(ProxyGuard)` — the proxy is running. Dropping the guard stops it.
- `None` — the port is busy or binding failed. A short message is printed to stderr.

### Behaviour

- Spawns a background thread with a `TcpListener`.
- Accepts connections in a loop; each connection is handled in its own thread.
- Reads the request line, extracts `/MinecraftSkins/<name>`, fetches the corresponding skin from `skinsystem.ely.by`, and writes it back as `200 OK` with `Content-Type: image/png`.
- Any other URL → `404 Not Found`.
- Requests are logged to stdout.
- The loop checks a shared `AtomicBool` every 50 ms; `ProxyGuard::drop` flips it to `false` and the loop exits.

### Example

```rust
use rmlib::extra::ely::start_beta_proxy;

let _proxy = start_beta_proxy(43523).expect("proxy port busy");
// game is launched with `-Dhttp.proxyHost=127.0.0.1 -Dhttp.proxyPort=43523`
// (that is what core::run does when ely = true and betafix = true)
std::thread::sleep(std::time::Duration::from_secs(60));
// _proxy drops here, stopping the proxy thread
```

The port `43523` is what `core::run` hardcodes when `ely = true, betafix = true` — see [Arguments](../core/run/arguments.md#build_arguments). If you want a different port, you must both call `start_beta_proxy(your_port)` **and** customise the JVM args yourself (`build_argv` accepts custom jvm args).

## `ProxyGuard`

```rust
pub struct ProxyGuard {
    running: Arc<AtomicBool>,
}

impl Drop for ProxyGuard {
    fn drop(&mut self) { /* stops the proxy thread */ }
}
```

RAII handle to a running beta proxy.

- Holding the guard keeps the proxy running.
- Dropping it signals the background thread to stop; the thread exits within ~50 ms.
- There is no explicit `stop` method — drop is the only way.

### Notes

- The proxy thread is detached. Dropping the guard does **not** wait for in-flight connections to finish; it just stops accepting new ones.
- Dropping the guard while the proxy is serving a request may cause that request to end abruptly. That is acceptable for the beta skin use case.

## End-to-end Ely.by flow

```rust
use rmlib::core::run::run_client;
use rmlib::extra::ely::{login, install_patch, start_beta_proxy};

// 1. Log in to Ely.by.
let auth = login("PlayerName".into(), "hunter2".into()).await?;

// 2. Install the authlib-injector patch (once).
install_patch(None, false).await?;

// 3a. Modern version: launch with the authlib-injector agent.
run_client(
    "1.20.1",
    None, None,
    "4G",
    &auth.profile_name,
    &auth.profile_id,
    "noxuid",
    &auth.access_token,
    true,   // ely — injects -javaagent:authlib-injector.jar=ely.by
    false,
    "~/rml/versions", "~/rml/assets", "~/rml/libs",
    "~/rml/libs/natives", "~/.minecraft", "~/.minecraft",
    &[], &[],
).await;

// 3b. Old beta: launch with the skin proxy.
let _proxy = start_beta_proxy(43523).expect("proxy port busy");
run_client(
    "b1.7.3",
    None, None,
    "4G",
    &auth.profile_name,
    &auth.profile_id,
    "noxuid",
    &auth.access_token,
    true,   // ely — combined with betafix below uses the local proxy
    true,   // betafix — sets -Dhttp.proxyHost=127.0.0.1 -Dhttp.proxyPort=43523
    "~/rml/versions", "~/rml/assets", "~/rml/libs",
    "~/rml/libs/natives", "~/.minecraft", "~/.minecraft",
    &[], &[],
).await;
```

## Notes

- **Ely.by is a third-party service.** It can change its API without notice. The `login` endpoint is the traditional Mojang-compatible `auth/authenticate`; if that changes, `login` will need updating.
- **The client token is hard-coded** (`93301754-cd11-4a00-bd50-f69dffaf4b0b`). This is what the CLI passes to Ely.by. It is not a secret, but every installation of the launcher uses the same token — Ely.by treats it as a "launcher identifier", not per-user.
- **The auth token is passed on the command line.** `-Dhttp.proxy…` and `-javaagent:…` are visible in `ps`. That is inherent to how authlib-injector works; it is not a leak introduced by this crate.

## See also

- [Arguments](../core/run/arguments.md) — `ely` / `betafix` flags in `build_arguments`
- [core::run — Command](../core/run/command.md) — `run_client` parameters
- [Features](../getting-started/features.md) — `ely` feature