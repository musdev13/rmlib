# FFI — Version

Version manifest fetching, per-version JSON, sorting, stripping, and Maven coordinate → path.

Mirrors `core::version` and `core::types::build_maven_path` — see [Version](../core/version.md) and [Types](../core/types.md) for the Rust-side semantics and manifest field assumptions.

All exports on this page require `core` **and** `ffi`.

```c
#include "rmlib.h"
```

## `rmlib_version_fetch_list`

```c
char *rmlib_version_fetch_list(void);
```

Fetches Mojang's version manifest and returns its `versions` array as JSON.

### Behaviour

- Blocks on `core::version::fetch_list()`.
- Same panics as the Rust function (fetch failure, invalid JSON, missing `versions`). The FFI wrapper catches those and reports them via `rmlib_last_error` as `"panic: …"`.

### Returns

NUL-terminated UTF-8 JSON array — the same shape as `version_manifest.json` → `versions`. Free with `rmlib_string_free`.

### Errors

`NULL` + `rmlib_last_error()` when the manifest cannot be fetched, is not valid JSON, or lacks a `versions` array.

### Example (Python)

```python
import json

lib.rmlib_version_fetch_list.restype = ctypes.c_void_p
ptr = lib.rmlib_version_fetch_list()
if not ptr:
    err = lib.rmlib_last_error()
    print("error:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    versions = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    print(len(versions), "versions")
```

**Network required.** `rmlib_version_fetch_list` hits `launchermeta.mojang.com`.

## `rmlib_version_get_json`

```c
char *rmlib_version_get_json(const char *version_id);
```

Fetches the full JSON document of a specific version.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_id` | NUL-terminated UTF-8 version ID (e.g. `"1.20.1"`). Required; `NULL` fails with an error. |

### Behaviour

- Blocks on `core::version::get_version_json(version_id)`.
- Same panics as the Rust function (manifest fetch, version not found, missing `url`, fetch failure). Panics are reported via `rmlib_last_error`.

### Returns

NUL-terminated UTF-8 JSON string. Free with `rmlib_string_free`.

### Errors

`NULL` on missing input, invalid UTF-8, or any panic along the way.

### Example (C)

```c
char *json = rmlib_version_get_json("1.20.1");
if (!json) {
    char *err = rmlib_last_error();
    fprintf(stderr, "%s\n", err ? err : "(no message)");
    rmlib_string_free(err);
} else {
    /* parse json */
    rmlib_string_free(json);
}
```

**Network required.** Two requests are made: the manifest, then the version JSON.

## `rmlib_version_get_json_url`

```c
char *rmlib_version_get_json_url(const char *version_id);
```

Returns the URL of a specific version's JSON document — same as `core::version::get_version_json_url`.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_id` | NUL-terminated UTF-8 version ID. Required. |

### Returns

NUL-terminated UTF-8 URL string. Free with `rmlib_string_free`.

### Errors

`NULL` when the version is not found, its `url` field is missing or not a string, or any panic occurs. Message via `rmlib_last_error`.

**Network required.** Fetches the manifest.

## `rmlib_version_sort_by_type`

```c
char *rmlib_version_sort_by_type(const char *json, int32_t version_type);
```

Filters a JSON array of manifest entries by their `type` field.

### Parameters

| Parameter | Meaning |
|---|---|
| `json` | NUL-terminated UTF-8 JSON array (as returned by `rmlib_version_fetch_list`). Required. |
| `version_type` | One of `0` (`release`), `1` (`snapshot`), `2` (`old_beta`), `3` (`old_alpha`). |

### Returns

NUL-terminated UTF-8 JSON array containing only the entries with the requested type, in input order. Free with `rmlib_string_free`.

### Errors

`NULL` when:

- `json` is `NULL` or not valid UTF-8;
- `json` is not valid JSON;
- `version_type` is outside `0..=3` (message: `"invalid VersionType: <n>"`).

Message via `rmlib_last_error`.

### Example (Python)

```python
import json

lib.rmlib_version_sort_by_type.argtypes = [ctypes.c_char_p, ctypes.c_int32]
lib.rmlib_version_sort_by_type.restype = ctypes.c_void_p

releases = json.dumps(manifest["versions"]).encode()
ptr = lib.rmlib_version_sort_by_type(releases, 0)   # 0 = release
if ptr:
    out = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    print(len(out), "releases")
```

## `rmlib_version_strip_version`

```c
char *rmlib_version_strip_version(
    const char *json,
    bool keep_id,
    bool keep_release_time,
    bool keep_time,
    bool keep_type,
    bool keep_url
);
```

Keeps only the selected fields in each entry — same as `core::version::strip_version`.

### Parameters

| Parameter | Meaning |
|---|---|
| `json` | NUL-terminated UTF-8 JSON array. Required. |
| `keep_id` | Keep `id`. |
| `keep_release_time` | Keep `releaseTime`. |
| `keep_time` | Keep `time`. |
| `keep_type` | Keep `type`. |
| `keep_url` | Keep `url`. |

### Returns

NUL-terminated UTF-8 JSON array of the same length as the input. Each element is an object containing exactly the selected fields that were present. Free with `rmlib_string_free`.

### Errors

`NULL` when `json` is `NULL`, not valid UTF-8, or not valid JSON. Message via `rmlib_last_error`.

### Example (C)

```c
/* keep only the "id" field */
char *stripped = rmlib_version_strip_version(manifest_json, true, false, false, false, false);
if (stripped) {
    /* use stripped */
    rmlib_string_free(stripped);
}
```

## `rmlib_build_maven_path`

```c
char *rmlib_build_maven_path(const char *name, const char *classifier);
```

Converts a Maven coordinate to a relative file path — same as `core::types::build_maven_path`.

### Parameters

| Parameter | Meaning |
|---|---|
| `name` | NUL-terminated UTF-8 coordinate, e.g. `"net.minecraft:client:1.20.1"`. Required. |
| `classifier` | Optional classifier override. Pass `NULL` to fall back to the 4th `:`-segment in `name`. |

### Returns

NUL-terminated UTF-8 relative path (e.g. `"net/minecraft/client/1.20.1/client-1.20.1.jar"`). Free with `rmlib_string_free`.

### Errors

`NULL` when:

- `name` is `NULL` or not valid UTF-8;
- `classifier` is non-`NULL` and not valid UTF-8;
- the coordinate has fewer than three `:`-separated parts.

Message via `rmlib_last_error`; the invalid-coordinate case has the message `"invalid maven coordinate"`.

### Example (Python)

```python
lib.rmlib_build_maven_path.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.rmlib_build_maven_path.restype = ctypes.c_void_p

ptr = lib.rmlib_build_maven_path(b"net.minecraft:client:1.20.1", None)
if ptr:
    print(ctypes.string_at(ptr).decode())
    # net/minecraft/client/1.20.1/client-1.20.1.jar
    lib.rmlib_string_free(ptr)
```

## Notes

- All exports on this page are synchronous; they block on a shared Tokio runtime. Do not call them from inside a Tokio runtime you started yourself.
- `rmlib_version_fetch_list`, `rmlib_version_get_json`, `rmlib_version_get_json_url` perform network I/O.
- `rmlib_version_sort_by_type`, `rmlib_version_strip_version`, `rmlib_build_maven_path` are pure — no I/O.

## See also

- [Errors and Strings](error.md) — reading `rmlib_last_error`, freeing strings
- [Version](../core/version.md) — Rust-side semantics
- [Types](../core/types.md) — `VersionType`, `build_maven_path`