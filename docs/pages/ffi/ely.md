# FFI — Ely.by

Authentication, the `authlib-injector` patch, and the local beta skin proxy.

Mirrors `extra::ely` — see [Ely.by](../extra/ely.md) for the Rust-side semantics and the end-to-end flow.

All exports on this page require `ely` **and** `ffi`.

```c
#include "rmlib.h"
```

## `rmlib_ely_login`

```c
char *rmlib_ely_login(const char *username, const char *password);
```

Authenticates against Ely.by — same as `extra::ely::login`.

### Parameters

| Parameter | Meaning |
|---|---|
| `username` | NUL-terminated UTF-8 Ely.by username / email. Required. |
| `password` | NUL-terminated UTF-8 Ely.by password. Required. |

### Returns

NUL-terminated UTF-8 **JSON object** — the serialized `AuthResult`:

```json
{
  "access_token": "…",
  "client_token": "…",
  "profile_id": "…",
  "profile_name": "…"
}
```

Free with `rmlib_string_free`.

### Errors

`NULL` on:

- `NULL` / non-UTF-8 input;
- network error;
- invalid JSON from Ely.by;
- missing `selectedProfile` field.

Message via `rmlib_last_error`. Panics (very unlikely — the Rust function returns `Result`, it does not panic) are reported as `"panic: …"`.

### Example (Python)

```python
import ctypes, json

lib = ctypes.CDLL("./target/release/librmlib.so")
lib.rmlib_ely_login.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.rmlib_ely_login.restype = ctypes.c_void_p
lib.rmlib_last_error.restype = ctypes.c_void_p
lib.rmlib_string_free.argtypes = [ctypes.c_void_p]

ptr = lib.rmlib_ely_login(b"PlayerName", b"hunter2")
if not ptr:
    err = lib.rmlib_last_error()
    print("login failed:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    auth = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    print("logged in as", auth["profile_name"])
```

**Network required.** Passes the credentials to `authserver.ely.by`.

### Security note

The password is passed as a plain C string. Do not keep it in memory longer than necessary; the FFI layer does not zero it. The `AuthResult` returned contains an access token — treat it like a password.

## `rmlib_ely_install_patch`

```c
int32_t rmlib_ely_install_patch(const char *opts_json);
```

Downloads `authlib-injector.jar` into the libraries directory — same as `extra::ely::install_patch`.

### Options schema

```json
{
  "libs_path": "~/rml/libs",
  "is_json": false
}
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `libs_path` | string or `null` | config default | Override the libraries directory. |
| `is_json` | bool | `false` | If `true`, suppresses progress output. |

### Returns

| Value | Meaning |
|---|---|
| `0` | The JAR was downloaded and written. |
| `-1` | JSON parse error, network failure, invalid release JSON, missing asset, download failure, or a panic. |

### Example (C)

```c
const char *opts = "{\"is_json\":false}";
if (rmlib_ely_install_patch(opts) != 0) {
    char *err = rmlib_last_error();
    fprintf(stderr, "%s\n", err ? err : "(no message)");
    rmlib_string_free(err);
}
```

**Network required.** Fetches the GitHub releases API and downloads the JAR.

After this succeeds, a `run_client` with `ely = true` (or `rmlib_run_client` with `"ely": true` in its JSON options) will find `authlib-injector.jar` and add `-javaagent:…=ely.by`.

## `rmlib_start_beta_proxy`

```c
void *rmlib_start_beta_proxy(uint16_t port);
```

Starts the local beta skin proxy — same as `extra::ely::start_beta_proxy`.

### Parameters

| Parameter | Meaning |
|---|---|
| `port` | TCP port on `127.0.0.1` to listen on. |

### Returns

| Case | Return value |
|---|---|
| Proxy started | An opaque handle (`*mut c_void`). Must be freed with `rmlib_proxy_guard_free`. |
| Port busy / bind failed | `NULL`, message via `rmlib_last_error`. |

The handle owns an `Arc<AtomicBool>` (a `ProxyGuard` in Rust). Dropping it stops the proxy thread. There is no explicit "stop" other than freeing the handle.

### Example (Python)

```python
lib.rmlib_start_beta_proxy.argtypes = [ctypes.c_uint16]
lib.rmlib_start_beta_proxy.restype = ctypes.c_void_p
lib.rmlib_proxy_guard_free.argtypes = [ctypes.c_void_p]
lib.rmlib_last_error.restype = ctypes.c_void_p
lib.rmlib_string_free.argtypes = [ctypes.c_void_p]

handle = lib.rmlib_start_beta_proxy(43523)
if not handle:
    err = lib.rmlib_last_error()
    print("proxy failed:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    # ... game is running, proxy serves /MinecraftSkins/<name> ...
    lib.rmlib_proxy_guard_free(handle)
```

### Example (C)

```c
void *guard = rmlib_start_beta_proxy(43523);
if (guard) {
    /* ... game runs ... */
    rmlib_proxy_guard_free(guard);
} else {
    char *err = rmlib_last_error();
    fprintf(stderr, "%s\n", err ? err : "(no message)");
    rmlib_string_free(err);
}
```

### Port choice

The launcher's `run_client` hard-codes port `43523` when `ely = true, betafix = true`. If you start the proxy on that port, the game's `-Dhttp.proxyPort=43523` will route through it. If you start it on a different port, you also need to adjust the JVM arguments yourself — `rmlib_run_client` does not expose a port override for `betafix`.

### Lifetime

The proxy runs on a detached background thread. Dropping the handle (via `rmlib_proxy_guard_free`) flips the shared `AtomicBool` to `false` and the accept loop exits within ~50 ms.

- Freeing the handle **before** the game exits will stop skin serving mid-session. Keep the handle alive for the duration of the launch.
- The handle can be freed from a different thread than the one that created it — the underlying `ProxyGuard` `Drop` is just an atomic store.
- Forgetting to free the handle leaks the proxy thread and its listener. It is not harmful beyond holding the port open, but the port cannot be reused.

## End-to-end (Python)

```python
import ctypes, json

lib = ctypes.CDLL("./target/release/librmlib.so")

# types
lib.rmlib_ely_login.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.rmlib_ely_login.restype = ctypes.c_void_p
lib.rmlib_ely_install_patch.argtypes = [ctypes.c_char_p]
lib.rmlib_ely_install_patch.restype = ctypes.c_int
lib.rmlib_run_client.argtypes = [ctypes.c_char_p]
lib.rmlib_run_client.restype = ctypes.c_int
lib.rmlib_start_beta_proxy.argtypes = [ctypes.c_uint16]
lib.rmlib_start_beta_proxy.restype = ctypes.c_void_p
lib.rmlib_proxy_guard_free.argtypes = [ctypes.c_void_p]
lib.rmlib_last_error.restype = ctypes.c_void_p
lib.rmlib_string_free.argtypes = [ctypes.c_void_p]

def take(ptr):
    if not ptr: return None
    s = ctypes.string_at(ptr).decode()
    lib.rmlib_string_free(ptr)
    return s

# 1. login
auth = json.loads(take(lib.rmlib_ely_login(b"PlayerName", b"hunter2")))

# 2. install patch (once)
rc = lib.rmlib_ely_install_patch(b'{"is_json":true}')
assert rc == 0, take(lib.rmlib_last_error())

# 3a. modern version — authlib-injector
opts = {
    "version_id": "1.20.1",
    "ram_count": "4G",
    "username": auth["profile_name"],
    "uuid": auth["profile_id"],
    "token": auth["access_token"],
    "xuid": "noxuid",
    "ely": True,
    "versions_path": "~/rml/versions",
    "assets_path": "~/rml/assets",
    "libs_path": "~/rml/libs",
    "native_path": "~/rml/libs/natives",
    "game_path": "~/.minecraft",
    "spawn_path": "~/.minecraft",
}
lib.rmlib_run_client(json.dumps(opts).encode())

# 3b. beta — local proxy
handle = lib.rmlib_start_beta_proxy(43523)
opts["version_id"] = "b1.7.3"
opts["betafix"] = True
lib.rmlib_run_client(json.dumps(opts).encode())
lib.rmlib_proxy_guard_free(handle)
```

## See also

- [Errors and Strings](error.md)
- [Ely.by](../extra/ely.md) — Rust-side semantics and end-to-end flow
- [Run](run.md) — `rmlib_run_client` options schema (includes `ely` / `betafix`)