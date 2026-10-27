# FFI — Installation

Installing a vanilla version, Fabric, or NeoForge. All three functions take a single **JSON options string** instead of long argument lists.

Mirrors `core::version::install`, `extra::fabric`, `extra::neoforge` — see [Version installation](../core/version-install.md), [Fabric](../extra/fabric.md), [NeoForge](../extra/neoforge.md).

```c
#include "rmlib.h"
```

## Common conventions

All functions on this page:

- return `int32_t` — `0` on success, `-1` on failure;
- block on the shared Tokio runtime;
- report the failure reason via `rmlib_last_error`;
- print progress to stdout / stderr — the same output the Rust functions produce.

The JSON options objects are decoded into a `serde_json::Value`; unknown keys are ignored, missing keys fall back to documented defaults (see each function).

## `rmlib_version_install`

```c
int32_t rmlib_version_install(const char *opts_json);
```

Installs a vanilla Minecraft version — same as `core::version::install::install`.

**Feature:** `core` + `ffi`.

### Options schema

```json
{
  "version_id": "1.20.1",
  "json_path": null,
  "versions_path": "~/rml/versions",
  "libs_path": "~/rml/libs",
  "assets_path": "~/rml/assets",
  "soft": true,
  "iclient": true,
  "ilibs": true,
  "iassets": true,
  "alibs": 150,
  "aassets": 350
}
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `version_id` | string or `null` | `null` | Install this version by ID. Conflicts with `json_path`. If both are `null`, the Rust code calls `process::exit(1)`. |
| `json_path` | string or `null` | `null` | Install from a local JSON. Used only if `version_id` is `null`. |
| `versions_path` | string or `null` | config default | Override the versions directory. |
| `libs_path` | string or `null` | config default | Override the libraries directory. |
| `assets_path` | string or `null` | config default | Override the assets directory. |
| `soft` | bool | `true` | Skip already-present files whose SHA-1 matches. |
| `iclient` | bool | `true` | Download the client JAR. |
| `ilibs` | bool | `true` | Download libraries + extract natives. |
| `iassets` | bool | `true` | Download assets. |
| `alibs` | number | `150` | Max concurrent library downloads. |
| `aassets` | number | `350` | Max concurrent asset downloads. |

### Returns

| Value | Meaning |
|---|---|
| `0` | The Rust `install` completed. (Library / asset download errors are printed but do not turn this into `-1` — see the Rust docs.) |
| `-1` | JSON parse error, invalid options, or a panic. Message via `rmlib_last_error`. |

### Example (Python)

```python
import ctypes, json

lib = ctypes.CDLL("./target/release/librmlib.so")
lib.rmlib_version_install.argtypes = [ctypes.c_char_p]
lib.rmlib_version_install.restype = ctypes.c_int
lib.rmlib_last_error.restype = ctypes.c_void_p
lib.rmlib_string_free.argtypes = [ctypes.c_void_p]

opts = {
    "version_id": "1.20.1",
    "soft": True,
    "iclient": True,
    "ilibs": True,
    "iassets": True,
    "alibs": 150,
    "aassets": 350,
}

rc = lib.rmlib_version_install(json.dumps(opts).encode())
if rc != 0:
    err = lib.rmlib_last_error()
    print("install failed:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
```

**Network required.** The install downloads the version JSON, client JAR, libraries and assets.

## `rmlib_fabric_install`

```c
int32_t rmlib_fabric_install(const char *opts_json);
```

Installs a Fabric-enabled version — same as `extra::fabric::install`.

**Feature:** `fabric` + `ffi`.

### Options schema

```json
{
  "version_id": "1.20.1",
  "fabric_version": "0.15.0",
  "versions_path": null,
  "libs_path": null,
  "assets_path": null
}
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `version_id` | string | — | Vanilla Minecraft version. **Required** — missing → `-1`, `"missing version_id"`. |
| `fabric_version` | string | — | Fabric Loader version. **Required** — missing → `-1`, `"missing fabric_version"`. |
| `versions_path` | string or `null` | config default | Override the versions directory. |
| `libs_path` | string or `null` | config default | Override the libraries directory. |
| `assets_path` | string or `null` | config default | Override the assets directory. |

### Returns

| Value | Meaning |
|---|---|
| `0` | The Fabric install completed. Fabric library download errors are printed but do not turn this into `-1`. |
| `-1` | Missing required keys, JSON parse error, invalid options, or a panic. |

### Example (C)

```c
const char *opts = "{\"version_id\":\"1.20.1\",\"fabric_version\":\"0.15.0\"}";
if (rmlib_fabric_install(opts) != 0) {
    char *err = rmlib_last_error();
    fprintf(stderr, "%s\n", err ? err : "(no message)");
    rmlib_string_free(err);
}
```

**Network required.** Soft-installs the vanilla base and downloads the Fabric profile and libraries.

## `rmlib_fabric_fetch_list`

```c
char *rmlib_fabric_fetch_list(const char *version_id);
```

Lists available Fabric Loader versions for a Minecraft version — same as `extra::fabric::fetch_list`.

**Feature:** `fabric` + `ffi`.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_id` | NUL-terminated UTF-8 Minecraft version. Required. |

### Returns

NUL-terminated UTF-8 JSON array of strings (e.g. `["0.15.4","0.15.3",…]`). Free with `rmlib_string_free`.

### Errors

`NULL` on `NULL` / non-UTF-8 input, on fetch failure, invalid JSON, or a response that is not a JSON array. Panics are reported via `rmlib_last_error`.

### Example (Python)

```python
import json

lib.rmlib_fabric_fetch_list.argtypes = [ctypes.c_char_p]
lib.rmlib_fabric_fetch_list.restype = ctypes.c_void_p

ptr = lib.rmlib_fabric_fetch_list(b"1.20.1")
if ptr:
    loaders = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    print(loaders[0])   # newest
```

**Network required.** Hits `meta.fabricmc.net`.

## `rmlib_neoforge_install`

```c
int32_t rmlib_neoforge_install(const char *opts_json);
```

Installs a NeoForge-enabled version — same as `extra::neoforge::install`.

**Feature:** `neoforge` + `ffi`.

### Options schema

```json
{
  "version_id": "1.20.1",
  "neoforge_version": "20.4.100",
  "versions_path": null,
  "libs_path": null,
  "assets_path": null
}
```

| Key | Type | Default | Meaning |
|---|---|---|---|
| `version_id` | string | — | Vanilla Minecraft version. **Required**. |
| `neoforge_version` | string | — | NeoForge version. **Required**. |
| `versions_path` | string or `null` | config default | Override the versions directory. |
| `libs_path` | string or `null` | config default | Override the libraries directory. |
| `assets_path` | string or `null` | config default | Override the assets directory. |

### Returns

| Value | Meaning |
|---|---|
| `0` | The NeoForge install completed. |
| `-1` | Missing required keys, JSON parse error, invalid options, or a panic. |

The NeoForge installer runs `java` as a subprocess — `java` must be on `PATH`. Failures of the subprocess are printed by the Rust code and do **not** turn the return value into `-1` (the Rust function returns `()` regardless). Check the filesystem afterwards.

### Example (C)

```c
const char *opts = "{\"version_id\":\"1.20.1\",\"neoforge_version\":\"20.4.100\"}";
if (rmlib_neoforge_install(opts) != 0) {
    char *err = rmlib_last_error();
    fprintf(stderr, "%s\n", err ? err : "(no message)");
    rmlib_string_free(err);
}
```

**Network required** and **`java` on `PATH`**.

## `rmlib_neoforge_fetch_list`

```c
char *rmlib_neoforge_fetch_list(const char *version_id);
```

Lists available NeoForge versions for a Minecraft version — same as `extra::neoforge::fetch_list`.

**Feature:** `neoforge` + `ffi`.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_id` | NUL-terminated UTF-8 Minecraft version. Required. |

### Returns

NUL-terminated UTF-8 JSON array of strings, newest first. Free with `rmlib_string_free`.

### Errors

`NULL` on `NULL` / non-UTF-8 input, fetch failure, invalid UTF-8 in the response, or an XML parse error. Panics reported via `rmlib_last_error`.

Note: an unknown-format `version_id` returns `[]` **without** a network request — the Rust function returns an empty vector without contacting the server.

## Progress output

All three install functions print progress to stdout / stderr, exactly as the Rust versions do (`Status::Task`, `Status::Ok`, `Status::Err` with colored labels). This is not controllable through the FFI — if you are capturing output, expect it to appear on the process' stdout.

## See also

- [Errors and Strings](error.md)
- [Version installation](../core/version-install.md)
- [Fabric](../extra/fabric.md), [NeoForge](../extra/neoforge.md)
- [Run](run.md) — launching after install