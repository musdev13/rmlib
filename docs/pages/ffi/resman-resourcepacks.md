# FFI — resman::resourcepacks

Local resource-pack discovery and inspection.

These exports mirror [`resman::resourcepacks`](../extra/resman-resourcepacks.md) — see that page for the data model, priority and fallback behaviour, and `pack.mcmeta` parsing.

All exports on this page require the `resman-resourcepacks` feature **and** the `ffi` feature.

```c
#include "rmlib.h"
```

Conventions are the same as for [resman::mods](resman-mods.md):

- `char *` results are UTF-8; JSON unless stated otherwise. Free with `rmlib_string_free`.
- `priority` is `NULL` (default `local`) or `"local"`, `"modrinth"`, `"modrinth,local"`.
- `fallback_json` is `NULL` (placeholder) or `{"policy": "..."}` with `placeholder`, `skip` or `fields`.
- `fields` is a comma-separated list, or `NULL` / `""` for all fields. Available fields: `id`, `description`, `pack_format`, `supported_formats`, `icon`, `path`, `enabled`, `filename`, `size`.
- `id` is present only when the pack was resolved through Modrinth (it is the project slug); local parsing leaves it empty.
- `format` is `"json"`, `"table"` or `"plain"`.

There is **no** enable/disable/toggle here: resource packs are not toggled by renaming.

## `rmlib_resman_resourcepacks_scan`

```c
char *rmlib_resman_resourcepacks_scan(const char *game_path);
```

Lists packs in `<game_path>/resourcepacks/`. Returns a JSON array of `ResourcepackInfo`.

## `rmlib_resman_resourcepacks_scan_dir`

```c
char *rmlib_resman_resourcepacks_scan_dir(const char *dir);
```

Lists packs in an explicit directory.

## `rmlib_resman_resourcepacks_scan_priority`

```c
char *rmlib_resman_resourcepacks_scan_priority(
    const char *game_path,
    const char *priority,
    const char *fallback_json
);
```

Resolves packs in `<game_path>/resourcepacks/` with the given priority and fallback policy.

## `rmlib_resman_resourcepacks_scan_priority_dir`

```c
char *rmlib_resman_resourcepacks_scan_priority_dir(
    const char *dir,
    const char *priority,
    const char *fallback_json
);
```

Like `rmlib_resman_resourcepacks_scan_priority`, but reads the given directory **directly** (no `resourcepacks` suffix added).

## `rmlib_resman_resourcepacks_info`

```c
char *rmlib_resman_resourcepacks_info(const char *path);
```

Returns a single `ResourcepackInfo` as JSON. `NULL` + `last_error` when the file is missing or the ZIP has no `pack.mcmeta`.

## `rmlib_resman_resourcepacks_info_priority`

```c
char *rmlib_resman_resourcepacks_info_priority(
    const char *path,
    const char *priority,
    const char *fallback_json
);
```

Resolves one file with priority + fallback. Returns `NULL` + `last_error` when the fallback policy is `skip`.

## `rmlib_resman_resourcepacks_fields`

```c
char *rmlib_resman_resourcepacks_fields(const char *path, const char *fields);
```

Returns a JSON object with only the requested fields of one pack.

## `rmlib_resman_resourcepacks_info_as`

```c
char *rmlib_resman_resourcepacks_info_as(
    const char *path,
    const char *fields,
    const char *format
);
```

Renders one pack in the requested format, optionally limited to `fields`.

### Example (Python)

```python
import ctypes

lib.rmlib_resman_resourcepacks_info_as.argtypes = [
    ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p
]
lib.rmlib_resman_resourcepacks_info_as.restype = ctypes.c_void_p

ptr = lib.rmlib_resman_resourcepacks_info_as(
    b"./game/resourcepacks/Faithful.zip",
    b"description,pack_format,supported_formats",
    b"plain",
)
if not ptr:
    err = lib.rmlib_last_error()
    print("error:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    print(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
```

## `rmlib_resman_resourcepacks_scan_as`

```c
char *rmlib_resman_resourcepacks_scan_as(
    const char *dir,
    const char *fields,
    const char *format
);
```

Renders a directory listing in the requested format, optionally limited to `fields`.

## See also

- [Errors and Strings](error.md)
- [resman::resourcepacks](../extra/resman-resourcepacks.md)
- [FFI — resman::mods](resman-mods.md)