# FFI — resman::mods

Local mod discovery, inspection and toggling.

These exports mirror [`resman::mods`](../extra/resman-mods.md) — see that page for the data model, parsing rules, priority and fallback behaviour.

All exports on this page require the `resman-mods` feature **and** the `ffi` feature.

```c
#include "rmlib.h"
```

Conventions:

- Functions returning `char *` return NUL-terminated UTF-8; JSON unless stated otherwise. Free with `rmlib_string_free`.
- Functions returning `int32_t` return `0` on success and `-1` on error.
- On error the pointer is `NULL` / the code is `-1`, and the message is available via `rmlib_last_error()`.
- `priority` is `NULL` (default `local`) or a comma-separated string: `"local"`, `"modrinth"`, `"modrinth,local"`.
- `fallback_json` is `NULL` (default placeholder) or `{"policy": "placeholder"}` / `{"policy": "skip"}` / `{"policy": "fields", "fields": ["icon", "description"]}`.
- `fields` is a comma-separated list of field names, or `NULL` / `""` for all fields.
- `format` is `"json"`, `"table"` or `"plain"`.

## `rmlib_resman_mods_scan`

```c
char *rmlib_resman_mods_scan(const char *game_path);
```

Lists mods in `<game_path>/mods/`. Returns a JSON array of `ModInfo`.

## `rmlib_resman_mods_scan_dir`

```c
char *rmlib_resman_mods_scan_dir(const char *dir);
```

Lists mods in an explicit directory. Returns a JSON array of `ModInfo`.

## `rmlib_resman_mods_scan_priority`

```c
char *rmlib_resman_mods_scan_priority(
    const char *game_path,
    const char *priority,
    const char *fallback_json
);
```

Resolves mods in `<game_path>/mods/` using the given source priority and fallback policy. Returns a JSON array of `ModInfo`.

## `rmlib_resman_mods_scan_priority_dir`

```c
char *rmlib_resman_mods_scan_priority_dir(
    const char *dir,
    const char *priority,
    const char *fallback_json
);
```

Like `rmlib_resman_mods_scan_priority`, but reads the given directory **directly** (no `mods` suffix added). Use this when you already have the mods directory path instead of a game directory.

## `rmlib_resman_mods_info`

```c
char *rmlib_resman_mods_info(const char *path);
```

Returns a single `ModInfo` as JSON.

## `rmlib_resman_mods_info_priority`

```c
char *rmlib_resman_mods_info_priority(
    const char *path,
    const char *priority,
    const char *fallback_json
);
```

Resolves one file with priority + fallback. Returns a JSON `ModInfo`, or `NULL` + `last_error` when the fallback policy is `skip`.

## `rmlib_resman_mods_by_id`

```c
char *rmlib_resman_mods_by_id(const char *dir, const char *id);
```

Finds a mod by `ModInfo::id` inside `dir`. Returns a JSON `ModInfo`, or `NULL` + `last_error` (`mod not found: <id>`).

## `rmlib_resman_mods_fields`

```c
char *rmlib_resman_mods_fields(const char *path, const char *fields);
```

Returns a JSON object with only the requested fields of one mod.

## `rmlib_resman_mods_info_as`

```c
char *rmlib_resman_mods_info_as(const char *path, const char *fields, const char *format);
```

Renders one mod in the requested format (`json` / `table` / `plain`), optionally limited to `fields`.

### Example (Python)

```python
import ctypes

lib.rmlib_resman_mods_info_as.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
lib.rmlib_resman_mods_info_as.restype = ctypes.c_void_p

ptr = lib.rmlib_resman_mods_info_as(
    b"./game/mods/sodium.jar",
    b"id,name,version",
    b"table",
)
if not ptr:
    err = lib.rmlib_last_error()
    print("error:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    print(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
```

## `rmlib_resman_mods_scan_as`

```c
char *rmlib_resman_mods_scan_as(const char *dir, const char *fields, const char *format);
```

Renders a directory listing in the requested format, optionally limited to `fields`.

## `rmlib_resman_mods_enable`

```c
char *rmlib_resman_mods_enable(const char *path);
```

Removes a trailing `.disabled`. Returns the resulting path (string, not JSON).

## `rmlib_resman_mods_disable`

```c
char *rmlib_resman_mods_disable(const char *path);
```

Appends `.disabled`. Returns the resulting path (string, not JSON).

## `rmlib_resman_mods_toggle`

```c
char *rmlib_resman_mods_toggle(const char *path);
```

Toggles the state. Returns JSON:

```json
{ "path": "./game/mods/sodium.jar.disabled", "enabled": false }
```

## `rmlib_resman_mods_is_enabled`

```c
int32_t rmlib_resman_mods_is_enabled(const char *path);
```

Returns `0` if the file name does not end with `.disabled`, `-1` otherwise (the message via `last_error` is `disabled`).

### Example (C)

```c
if (rmlib_resman_mods_is_enabled("./game/mods/sodium.jar") == -1) {
    char *err = rmlib_last_error();
    /* err == "disabled" */
    rmlib_string_free(err);
}
```

## See also

- [Errors and Strings](error.md)
- [resman::mods](../extra/resman-mods.md)
- [FFI — resman::resourcepacks](resman-resourcepacks.md)