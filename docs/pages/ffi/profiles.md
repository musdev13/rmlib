# FFI — Profiles

Saved account profiles.

These exports mirror [`extra::profiles`](../extra/profiles.md) — see that page for the profile format and storage layout.

All exports on this page require the `profiles` feature **and** the `ffi` feature.

```c
#include "rmlib.h"
```

## Common parameters

| Parameter | Meaning |
|---|---|
| `kind` | `"ely"` or `"ms"`. Any other value is an error (`invalid profile kind`). |
| `username` | Profile name — the file stem. Empty, `.`, `..`, and names containing `/` or `\` are rejected. |
| `dir` | Auth directory, or `NULL` to use `auth_path` from `paths.json`. |
| `json` | A JSON object matching the `Profile` shape (see below). |

### Profile JSON

```json
{
  "username": "Musek",
  "uuid": "…",
  "access_token": "…",
  "client_token": "…",
  "xuid": "…",
  "has_license": true
}
```

Only `username` and `access_token` are required; the rest are optional and may be omitted.

## `rmlib_profile_save` / `rmlib_profile_save_at`

```c
char *rmlib_profile_save(const char *kind, const char *json);
char *rmlib_profile_save_at(const char *dir, const char *kind, const char *json);
```

Writes `profiles/<kind>/<username>.json` (atomically, creating directories) and returns the absolute path of the written file.

### Returns

NUL-terminated UTF-8 path string. Free with `rmlib_string_free`.

### Errors

Returns `NULL` and sets `rmlib_last_error` on invalid `kind`, invalid profile JSON, an invalid username, or an I/O failure.

### Example (Python)

```python
import ctypes, json

lib.rmlib_profile_save.restype = ctypes.c_void_p
payload = json.dumps({
    "username": "Musek",
    "uuid": "u",
    "access_token": "tok",
    "client_token": "ct",
}).encode()
ptr = lib.rmlib_profile_save(b"ely", payload)
if not ptr:
    err = lib.rmlib_last_error()
    print("save failed:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    print("saved to", ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
```

## `rmlib_profile_load` / `rmlib_profile_load_at`

```c
char *rmlib_profile_load(const char *kind, const char *username);
char *rmlib_profile_load_at(const char *dir, const char *kind, const char *username);
```

Reads a profile and returns it as JSON.

### Returns

NUL-terminated UTF-8 JSON object (the `Profile`). Free with `rmlib_string_free`.

### Errors

Returns `NULL` and sets `rmlib_last_error` if the profile does not exist (`profile not found: <name>`), is not valid JSON, or on an I/O failure.

### Example (C)

```c
char *p = rmlib_profile_load("ely", "Musek");
if (!p) {
    char *err = rmlib_last_error();
    fprintf(stderr, "load failed: %s\n", err ? err : "(no message)");
    rmlib_string_free(err);
} else {
    printf("%s\n", p);
    rmlib_string_free(p);
}
```

## `rmlib_profile_list` / `rmlib_profile_list_at`

```c
char *rmlib_profile_list(const char *kind);
char *rmlib_profile_list_at(const char *dir, const char *kind);
```

Lists the profile names for a kind.

### Returns

NUL-terminated UTF-8 JSON array of strings, sorted alphabetically. A missing profiles directory yields `[]`. Free with `rmlib_string_free`.

### Example (Python)

```python
import json

lib.rmlib_profile_list.restype = ctypes.c_void_p
ptr = lib.rmlib_profile_list(b"ms")
if ptr:
    names = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    print(names)  # e.g. ["alice", "bob"]
```

## `rmlib_profile_delete` / `rmlib_profile_delete_at`

```c
int32_t rmlib_profile_delete(const char *kind, const char *username);
int32_t rmlib_profile_delete_at(const char *dir, const char *kind, const char *username);
```

Deletes `profiles/<kind>/<username>.json`.

### Returns

| Value | Meaning |
|---|---|
| `0` | The file was removed, or did not exist. |
| `-1` | Invalid `kind` / username, I/O error, or panic. The message is available via `rmlib_last_error`. |

## `rmlib_profile_path`

```c
char *rmlib_profile_path(const char *kind, const char *username);
```

Returns the path where the profile would be stored (`auth_path` based), without touching the filesystem. Useful for diagnostics.

### Returns

NUL-terminated UTF-8 path string. Free with `rmlib_string_free`.

### Errors

Returns `NULL` and sets `rmlib_last_error` on an invalid `kind` or username.

## See also

- [Errors and Strings](error.md) — reading `rmlib_last_error` and freeing strings
- [Profiles](../extra/profiles.md) — the Rust-side API and storage layout