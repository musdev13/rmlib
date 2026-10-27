# core::run — Command

End-to-end command assembly and game launch.

Two building blocks (`build_argv` / `build_command`) produce the final `java` command from the same parameters the CLI passes in; `run_client` actually spawns the process.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

```rust
use rmlib::core::run::{build_argv, build_command, run_client};
```

## Parameter list

`build_argv`, `build_command` and `run_client` share the same first 18 parameters. They are the flattened form of everything `core::config` + `core::run` need to assemble a launch:

| # | Parameter | Type | Meaning |
|---|---|---|---|
| 1 | `version_id` | `&str` | Version to launch (e.g. `"1.20.1"`). |
| 2 | `fabric_version` | `Option<&str>` | Fabric Loader version, if any. |
| 3 | `neoforge_version` | `Option<&str>` | NeoForge version, if any. |
| 4 | `ram_count` | `&str` | Heap size (e.g. `"4G"`). |
| 5 | `username` | `&str` | Player name. |
| 6 | `uuid` | `&str` | Player UUID. |
| 7 | `xuid` | `&str` | Xbox user ID. |
| 8 | `token` | `&str` | Access token. |
| 9 | `ely` | `bool` | Enable Ely.by `authlib-injector` path. |
| 10 | `betafix` | `bool` | Enable the beta-fix proxy. |
| 11 | `versions_path` | `&str` | Versions directory (raw string, `~` allowed). |
| 12 | `assets_path` | `&str` | Assets directory. |
| 13 | `libs_path` | `&str` | Libraries directory. |
| 14 | `native_path` | `&str` | Natives directory. |
| 15 | `game_path` | `&str` | Game directory (working directory for the game). |
| 16 | `custom_jvm_args` | `&[String]` | Extra JVM args appended at the end. |
| 17 | `custom_game_args` | `&[String]` | Extra game args appended at the end. |
| 18 | `logs` | `bool` | Only used by `build_argv` / `build_command`: enable extra logging during assembly. |

`run_client` takes the same 18 parameters **minus** `logs`, and adds `spawn_path` (see below).

## `build_argv`

```rust
#[allow(clippy::too_many_arguments)]
pub fn build_argv(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    ram_count: &str,
    username: &str,
    uuid: &str,
    xuid: &str,
    token: &str,
    ely: bool,
    betafix: bool,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
    custom_jvm_args: &[String],
    custom_game_args: &[String],
    logs: bool,
) -> Result<Vec<String>, String>
```

Returns the full `java` argument vector for the launch — the same vector `run_client` would execute.

Internally:

1. `prepare_paths(version_id, versions_path, assets_path, libs_path, native_path, game_path)`.
2. `load_version_data(version_id, fabric_version, neoforge_version, &paths, logs)`.
3. `resolve_environment()`.
4. `build_classpath(...)`.
5. `build_launch_context(...)`.
6. `build_arguments(...)`.
7. `assemble_argv(main_class, arguments, &context)`.

### Returns

- `Ok(Vec<String>)` — `["java", …jvm args…, main_class, …game args…]`, with placeholders already substituted.
- `Err(String)` — one of the errors from `prepare_paths` (`client json not found at …`) or `load_version_data` (`failed to read client json: …`, `failed to parse client json: …`, `missing mainClass inside json`).

### Example

```rust
use rmlib::core::run::build_argv;

let argv = build_argv(
    "1.20.1",
    None, None,
    "4G",
    "Steve",
    "00000000-0000-0000-0000-000000000000",
    "xuid",
    "token",
    false, false,
    "~/rml/versions",
    "~/rml/assets",
    "~/rml/libs",
    "~/rml/libs/natives",
    "~/.minecraft",
    &[],
    &[],
    false,
)?;

assert_eq!(argv[0], "java");
```

This is what `--just-command`-style CLI flags typically print: the same command `run_client` would run, but without spawning anything.

## `build_command`

```rust
#[allow(clippy::too_many_arguments)]
pub fn build_command(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    ram_count: &str,
    username: &str,
    uuid: &str,
    xuid: &str,
    token: &str,
    ely: bool,
    betafix: bool,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
    custom_jvm_args: &[String],
    custom_game_args: &[String],
    logs: bool,
) -> Result<String, String>
```

Same as `build_argv`, but returns the arguments joined into a single string for display.

- Same 18 parameters.
- Same `Ok` / `Err` semantics.
- Each argument containing a space is wrapped in `"…"`; the rest are emitted verbatim.

### Example

```rust
use rmlib::core::run::build_command;

let cmd = build_command(
    "1.20.1",
    None, None,
    "4G",
    "Steve", "uuid", "xuid", "token",
    false, false,
    "~/rml/versions", "~/rml/assets", "~/rml/libs",
    "~/rml/libs/natives", "~/.minecraft",
    &[], &[], false,
)?;

println!("{cmd}");
// java -Xmx4G -Djava.library.path=… -cp … net.minecraft.client.main.Main --username Steve …
```

## `run_client`

```rust
#[allow(clippy::too_many_arguments)]
pub async fn run_client(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    ram_count: &str,
    username: &str,
    uuid: &str,
    xuid: &str,
    token: &str,
    ely: bool,
    betafix: bool,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
    spawn_path: &str,
    custom_jvm_args: &[String],
    custom_game_args: &[String],
)
```

Assembles the command (exactly as `build_argv` would), then **spawns `java` from `PATH`** and waits for it to exit.

### Parameters

Same 18 as `build_argv`, minus `logs`, plus:

| Parameter | Meaning |
|---|---|
| `spawn_path` | The working directory for the spawned `java` process. Usually the same as `game_path`, but the launcher's CLI lets you set it separately. `~` and `..` are handled by `musutils::fs::expand`. |

### Behaviour

1. Prints `Status::Task: preparing launch sequence...`.
2. Calls `build_argv` internally. On error, prints `Status::Err: <message>` and **returns** (no `Result`, no panic).
3. If the argv is empty, prints `Status::Err: empty command` and returns.
4. Resolves `spawn_path` (tilde-expanded).
5. Prints `Status::Inf: spawn dir - <path>`.
6. Prints `Status::Task: spawning java process...`.
7. Spawns `Command::new(&argv[0]).args(&argv[1..]).current_dir(spawn_path)`.
   - On spawn failure, prints `Status::Err: failed to execute java process: <e>` and returns.
8. Prints `Status::Ok: instance started successfully! Press Ctrl+C to exit.`.
9. Calls `child.wait()`.
   - On success, prints `Status::Ok: game exited with status: <status>`.
   - On error, prints `Status::Err: error waiting for game process: <e>`.

### Returns

`()` — the function communicates through stdout / stderr only. There is no way to get the game's exit status out of this function; use `build_argv` and spawn `java` yourself if you need that.

### Example

```rust
use rmlib::core::config;
use rmlib::core::run::run_client;

let versions = config::get_versions_path();
let assets = config::get_assets_path();
let libs = config::get_libs_path();
let natives = format!("{libs}/natives");
let game = "~/.minecraft";

run_client(
    "1.20.1",
    None, None,
    "4G",
    "Steve",
    "00000000-0000-0000-0000-000000000000",
    "xuid",
    "token",
    false, false,
    &versions, &assets, &libs, &natives, game, game,
    &[],
    &[],
).await;
```

## Dry-run vs real launch

| Goal | Function |
|---|---|
| Inspect the full argv before launching | `build_argv` → `Vec<String>` |
| Print a shell-ish one-liner | `build_command` → `String` |
| Actually start the game | `run_client` |

`build_argv` and `build_command` do not touch the filesystem beyond reading the version JSON and checking library presence (`prepare_paths` / `verify_libraries`). `run_client` spawns a process.

## Error handling

`build_argv` / `build_command` return `Err(String)` when the version JSON is missing, unreadable or invalid, or when `mainClass` is absent. They never panic on their own — the panics from `resolve_version_data` (which is called by `install`, not by `run`) do not occur here.

`run_client` does not return errors — it prints them and returns. If you need the error value, call `build_argv` yourself, handle the error, and then spawn `java` however you prefer.

## See also

- [Paths](paths.md) — `prepare_paths`, `load_version_data`
- [Classpath](classpath.md) — `build_classpath`
- [Arguments](arguments.md) — `build_arguments`, `assemble_argv`
- [Version installation](../version-install.md) — how to populate `versions/` before calling `run_client`