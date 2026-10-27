# FFI — Config

Configuration file helpers and path getters.

These exports mirror `core::config` — see [Config](../core/config.md) for the Rust-side behavior, default values, and file locations.

All exports on this page require the `core` feature **and** the `ffi` feature.

```c
#include "rmlib.h"
```

## `rmlib_config_init`

```c
int32_t rmlib_config_init(bool force);
```

Creates the configuration file if it does not exist — same as `rmlib::core::config::init(force)` in Rust.

### Parameters

| Parameter | Meaning |
|---|---|
| `force` | `true` → rewrite the file with defaults (discarding user edits). `false` → create only if missing. |

### Behaviour

- On success, prints status messages to stdout / stderr (via the Rust `init` helper). This is what the Rust `init` does — it does not change for FFI.
- On failure, prints the error to stderr. The integer return code is still `0` in that case, because the underlying `init` does not return an error — it only logs.

### Returns

| Value | Meaning |
|---|---|
| `0` | The `init` call completed. (This does **not** guarantee the file was created — check the filesystem, or use `rmlib_paths_load` afterwards.) |
| `-1` | The FFI wrapper itself failed (panic during `init`). The panic message is available via `rmlib_last_error`. |

### Example (Python)

```python
lib.rmlib_config_init.argtypes = [ctypes.c_bool]
lib.rmlib_config_init.restype = ctypes.c_int

if lib.rmlib_config_init(False) != 0:
    err = lib.rmlib_last_error()
    print("rmlib_config_init failed:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
```

### Example (C)

```c
if (rmlib_config_init(false) != 0) {
    char *err = rmlib_last_error();
    fprintf(stderr, "%s\n", err ? err : "(no message)");
    rmlib_string_free(err);
}
```

## `rmlib_get_versions_path`

```c
char *rmlib_get_versions_path(void);
```

Returns `core::config::get_versions_path()` — the `versions_path` string from `paths.json`.

### Returns

NUL-terminated UTF-8 string (e.g. `"~/rml/versions"`). The value is the **raw** string from the config file — `~` is **not** expanded here. Free with `rmlib_string_free`.

### Errors

Returns `NULL` and sets `rmlib_last_error` if the configuration cannot be loaded. The underlying Rust helper `get_versions_path()` panics on failure; the panic is caught by the FFI wrapper and reported as a `"panic: …"` message.

### Example (Python)

```python
lib.rmlib_get_versions_path.restype = ctypes.c_void_p
ptr = lib.rmlib_get_versions_path()
if ptr:
    print(ctypes.string_at(ptr).decode())   # "~/rml/versions"
    lib.rmlib_string_free(ptr)
else:
    err = lib.rmlib_last_error()
    print("error:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
```

## `rmlib_get_libs_path`

```c
char *rmlib_get_libs_path(void);
```

Returns `core::config::get_libs_path()` — the `libs_path` string from `paths.json`.

Same return / error semantics as `rmlib_get_versions_path`.

## `rmlib_get_assets_path`

```c
char *rmlib_get_assets_path(void);
```

Returns `core::config::get_assets_path()` — the `assets_path` string from `paths.json`.

Same return / error semantics as `rmlib_get_versions_path`.

## `rmlib_paths_load`

```c
char *rmlib_paths_load(void);
```

Loads the configuration and returns all three values as JSON — equivalent to calling `Paths::load()` in Rust.

### Schema

```json
{
  "versions": "~/rml/versions",
  "libs": "~/rml/libs",
  "assets": "~/rml/assets"
}
```

The three values are the **raw** strings from `paths.json` — `~` is not expanded. Expand them yourself (`musutils::fs::expand` does that in Rust; the C side gets the raw strings).

### Returns

NUL-terminated UTF-8 JSON string. Free with `rmlib_string_free`.

### Errors

Returns `NULL` and sets `rmlib_last_error` if:

- the config file cannot be created / read,
- it is not valid JSON,
- one of the three fields is missing or has the wrong type.

The error message is the `Display` output of `musutils::fs::config::ConfigError` — e.g. `"key not found: versions_path"`, `"type mismatch for key \`libs_path\`, expected alloc::string::String"`, `"parse error: …"`.

### Example (Python)

```python
import json

lib.rmlib_paths_load.restype = ctypes.c_void_p
ptr = lib.rmlib_paths_load()
if not ptr:
    err = lib.rmlib_last_error()
    print("error:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    paths = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    print(paths["versions"], paths["libs"], paths["assets"])
```

## `rmlib_paths_init`

```c
int32_t rmlib_paths_init(bool force);
```

Equivalent to `Paths::init(force)` — creates the configuration file, or rewrites it when `force` is `true`.

Unlike `rmlib_config_init`, this one **does** report failure through its return code: the underlying `Paths::init` returns `Result<(), ConfigError>`, and the wrapper maps it to `0` / `-1`.

### Parameters

| Parameter | Meaning |
|---|---|
| `force` | `true` → overwrite with defaults. `false` → create only if missing. |

### Returns

| Value | Meaning |
|---|---|
| `0` | The file was created / kept. |
| `-1` | I/O error, parse error, or panic. The message is available via `rmlib_last_error`. |

### Example (C)

```c
if (rmlib_paths_init(false) != 0) {
    char *err = rmlib_last_error();
    fprintf(stderr, "paths init failed: %s\n", err ? err : "(no message)");
    rmlib_string_free(err);
}
```

## Which of `rmlib_config_init` and `rmlib_paths_init` should I use?

They call different Rust functions:

| FFI export | Rust function | Returns an error? |
|---|---|---|
| `rmlib_config_init` | `core::config::init(force)` | No — logs and returns `()`. The FFI wrapper returns `0` on that path. |
| `rmlib_paths_init` | `core::config::Paths::init(force)` | Yes — returns `Result<(), ConfigError>`. The FFI wrapper maps `Err` to `-1`. |

For new code, prefer **`rmlib_paths_init`** — it gives you a real success/failure signal. `rmlib_config_init` exists for parity with the Rust `init` helper that the CLI uses.

## Typical flow

```text
rmlib_paths_init(false)    → ensure paths.json exists
rmlib_paths_load()         → read all three paths
rmlib_get_versions_path()  → or read one at a time
```

The values are raw strings (`~` unexpanded). If you need resolved paths for FFI calls that take path strings (e.g. `rmlib_version_install`), pass the raw strings — the Rust side expands them.

## See also

- [Errors and Strings](error.md) — reading `rmlib_last_error` and freeing strings
- [Config](../core/config.md) — the Rust-side behavior and file locations
- [Version](version.md) — functions that consume these paths via JSON options