# FFI — Run

Assembling the `java` command and launching the game.

Mirrors `core::run::build_argv`, `core::run::build_command`, `core::run::run_client` — see [Command](../core/run/command.md) for the Rust-side semantics and the full list of parameters.

All exports on this page require `core` **and** `ffi`.

```c
#include "rmlib.h"
```

## `rmlib_build_argv`

```c
char *rmlib_build_argv(const char *opts_json);
```

Builds the full `java` argument vector for a launch — same as `core::run::build_argv`.

### Options schema

```json
{
  "version_id": "1.20.1",
  "fabric_version": null,
  "neoforge_version": null,
  "ram_count": "2G",
  "username": "Steve",
  "uuid": "00000000-0000-0000-0000-000000000000",
  "xuid": "noxuid",
  "token": "token",
  "ely": false,
  "betafix": false,
  "versions_path": "~/rml/versions",
  "assets_path": "~/rml/assets",
  "libs_path": "~/rml/libs",
  "native_path": "~/rml/libs/natives",
  "game_path": "~/.minecraft",
  "custom_jvm_args": [],
  "custom_game_args": [],
  "logs": false
}
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `version_id` | string | — | Version to launch. **Required** — missing → `NULL`, `"missing version_id"`. |
| `fabric_version` | string or `null` | `null` | Fabric Loader version. |
| `neoforge_version` | string or `null` | `null` | NeoForge version. |
| `ram_count` | string | `"2G"` | Heap size for `-Xmx`. |
| `username` | string | — | Player name. **Required** — missing → `NULL`, `"missing username"`. |
| `uuid` | string | `""` | Player UUID. |
| `xuid` | string | `""` | Xbox user ID. |
| `token` | string | `""` | Access token. |
| `ely` | bool | `false` | Enable the Ely.by authlib-injector path. |
| `betafix` | bool | `false` | Enable the beta-fix proxy. |
| `versions_path` | string | — | Versions directory. **Required**. |
| `assets_path` | string | — | Assets directory. **Required**. |
| `libs_path` | string | — | Libraries directory. **Required**. |
| `native_path` | string | `""` | Natives directory. |
| `game_path` | string | `""` | Game directory. |
| `custom_jvm_args` | array of strings | `[]` | Extra JVM args. |
| `custom_game_args` | array of strings | `[]` | Extra game args. |
| `logs` | bool | `false` | Enable extra logging during assembly. |

### Returns

NUL-terminated UTF-8 **JSON array of strings** — the same vector the Rust `build_argv` returns. Free with `rmlib_string_free`.

```json
[
  "java",
  "-Xmx2G",
  "-Djava.library.path=…",
  "-cp",
  "…",
  "net.minecraft.client.main.Main",
  "--username",
  "Steve",
  "…"
]
```

### Errors

`NULL` when:

- the JSON options string is missing required keys (`version_id`, `username`, `versions_path`, `assets_path`, `libs_path`);
- the JSON is malformed;
- the version JSON cannot be read or parsed — same conditions that make the Rust `build_argv` return `Err(String)`. The message is the Rust error string;
- the whole call panics.

Message via `rmlib_last_error`.

### Example (Python)

```python
import ctypes, json

lib = ctypes.CDLL("./target/release/librmlib.so")
lib.rmlib_build_argv.argtypes = [ctypes.c_char_p]
lib.rmlib_build_argv.restype = ctypes.c_void_p
lib.rmlib_last_error.restype = ctypes.c_void_p
lib.rmlib_string_free.argtypes = [ctypes.c_void_p]

opts = {
    "version_id": "1.20.1",
    "ram_count": "2G",
    "username": "Steve",
    "uuid": "00000000-0000-0000-0000-000000000000",
    "token": "token",
    "versions_path": "~/rml/versions",
    "assets_path": "~/rml/assets",
    "libs_path": "~/rml/libs",
    "native_path": "~/rml/libs/natives",
    "game_path": "~/.minecraft",
}

ptr = lib.rmlib_build_argv(json.dumps(opts).encode())
if not ptr:
    err = lib.rmlib_last_error()
    print("error:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    argv = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    print(" ".join(argv))
```

**Network required** only if the version JSON or its libraries need fetching — otherwise it reads the local version JSON. No process is spawned.

## `rmlib_build_command`

```c
char *rmlib_build_command(const char *opts_json);
```

Builds the same command as `rmlib_build_argv`, but returns it as a single shell-style string.

### Options schema

Identical to `rmlib_build_argv`.

### Returns

NUL-terminated UTF-8 string — the arguments joined with spaces; arguments containing a space are quoted with `"…"`. Free with `rmlib_string_free`.

### Errors

Same conditions as `rmlib_build_argv`. Message via `rmlib_last_error`.

### Example (C)

```c
const char *opts = "{"
    "\"version_id\":\"1.20.1\","
    "\"ram_count\":\"2G\","
    "\"username\":\"Steve\","
    "\"versions_path\":\"~/rml/versions\","
    "\"assets_path\":\"~/rml/assets\","
    "\"libs_path\":\"~/rml/libs\","
    "\"native_path\":\"~/rml/libs/natives\","
    "\"game_path\":\"~/.minecraft\""
    "}";

char *cmd = rmlib_build_command(opts);
if (cmd) {
    printf("%s\n", cmd);
    rmlib_string_free(cmd);
}
```

No process is spawned. Useful for logging, reproducing a launch manually, or piping into a debugger.

## `rmlib_run_client`

```c
int32_t rmlib_run_client(const char *opts_json);
```

Assembles the command (same as `rmlib_build_argv`) and **spawns `java`**, then blocks until the game exits.

### Options schema

Same as `rmlib_build_argv`, plus one key:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `spawn_path` | string | `""` | Working directory for the `java` process. `~` / `..` expanded. |

### Behaviour

- Prints `preparing launch sequence...`, `spawn dir - …`, `spawning java process...`, `instance started successfully! Press Ctrl+C to exit.`, then waits for the process and prints the exit status.
- Same panic/reporting semantics as the Rust `run_client`: internal `build_argv` errors are printed and the function returns `0` (the Rust function returns `()` — this wrapper maps that to `0`).
- If `java` cannot be spawned, prints an error and returns `0`.
- **Blocks** until the game process exits. This can be hours. Do not call it on a UI thread.

### Returns

| Value | Meaning |
|---|---|
| `0` | `run_client` completed. This is the **Rust-level** completion — `run_client` itself does not surface subprocess errors through its return value. Errors during assembly, spawn, or wait are printed to stdout / stderr, and the return value is still `0`. |
| `-1` | The FFI wrapper failed before `run_client` could run: missing required JSON keys, JSON parse error, or a panic. |

Because `rmlib_run_client` returns `()` in Rust (it reports through output, not through `Result`), the FFI `-1` only covers option-decoding and panic paths, **not** "the game failed to start". If you need to know whether the game actually started, use `rmlib_build_argv` and spawn `java` yourself.

### Example (Python)

```python
import ctypes, json

lib.rmlib_run_client.argtypes = [ctypes.c_char_p]
lib.rmlib_run_client.restype = ctypes.c_int

opts = {
    "version_id": "1.20.1",
    "ram_count": "4G",
    "username": "Steve",
    "uuid": "…",
    "token": "…",
    "versions_path": "~/rml/versions",
    "assets_path": "~/rml/assets",
    "libs_path": "~/rml/libs",
    "native_path": "~/rml/libs/natives",
    "game_path": "~/.minecraft",
    "spawn_path": "~/.minecraft",
}

rc = lib.rmlib_run_client(json.dumps(opts).encode())
# rc == 0 → run_client returned; the game has exited.
# rc == -1 → options decoding or a panic failed the call before launch.
```

**Blocks until the game exits.** `java` must be on `PATH`.

## Which to use?

| Goal | Function |
|---|---|
| Inspect the argv before launching | `rmlib_build_argv` |
| Log / reproduce the command | `rmlib_build_command` |
| Actually start the game (blocking) | `rmlib_run_client` |

If you need the game's exit code, use `rmlib_build_argv` and spawn `java` yourself — `rmlib_run_client` does not return it.

## Note on the `logs` key

`rmlib_build_argv` accepts a `logs` boolean; `rmlib_run_client` accepts it too but the Rust `run_client` does not use it (it has no `logs` parameter — the Rust function passes `false` internally). The key is present for uniformity with `rmlib_build_argv`; it is ignored by `rmlib_run_client`.

## Thread model

`rmlib_run_client` blocks on a shared Tokio runtime while the game process runs. The runtime is single-instance: calling `rmlib_run_client` from one thread and any other blocking `rmlib_*` export from another will serialise on that runtime. Do not call them from inside a Tokio runtime you started yourself.

## See also

- [Errors and Strings](error.md)
- [Command](../core/run/command.md) — Rust-side semantics and the full parameter list
- [Installation](install.md) — populate `versions/` before running