# FFI — Library Info

Crate metadata and the set of enabled modules.

All functions on this page are available whenever the `ffi` feature is enabled, regardless of which area features (`core`, `ely`, `fabric`, `neoforge`) are also on.

```c
#include "rmlib.h"
```

## `rmlib_version`

```c
char *rmlib_version(void);
```

Returns the crate version, identical to `rmlib::VERSION` and to `env!("CARGO_PKG_VERSION")` in Rust.

**Returns:** a NUL-terminated UTF-8 string (e.g. `"0.10.0"`). The caller must free it with `rmlib_string_free`.

**Errors:** does not fail. The string is short and always allocated.

### Example (Python)

```python
lib.rmlib_version.restype = ctypes.c_void_p
ptr = lib.rmlib_version()
print(ctypes.string_at(ptr).decode())   # "0.10.0"
lib.rmlib_string_free(ptr)
```

### Example (C)

```c
char *v = rmlib_version();
printf("rmlib %s\n", v);
rmlib_string_free(v);
```

## `rmlib_name`

```c
char *rmlib_name(void);
```

Returns the crate name — always `"rmlib"`.

**Returns:** NUL-terminated UTF-8 string. Free with `rmlib_string_free`.

**Errors:** does not fail.

## `rmlib_client_token`

```c
char *rmlib_client_token(void);
```

Returns `rmlib::CLIENT_TOKEN` — the Microsoft/Xbox Live client token the launcher uses.

**Returns:** NUL-terminated UTF-8 string. Free with `rmlib_string_free`.

**Errors:** does not fail.

## `rmlib_client_id`

```c
char *rmlib_client_id(void);
```

Returns `rmlib::CLIENT_ID` — the Microsoft/Xbox Live client ID.

**Returns:** NUL-terminated UTF-8 string. Free with `rmlib_string_free`.

**Errors:** does not fail.

## `rmlib_available_modules`

```c
char *rmlib_available_modules(void);
```

Returns a JSON array of the top-level modules compiled into this shared library — the same data `rmlib::available_modules()` returns in Rust.

### Schema

```json
[
  { "name": "core" },
  { "name": "ely" },
  { "name": "fabric" },
  { "name": "neoforge" }
]
```

The array contains one object per enabled feature, in this order: `core`, `ely`, `fabric`, `neoforge`. An entry is present only when the corresponding feature is on. `ffi` itself is **not** listed — it is a build concern, not a launcher module.

**Returns:** NUL-terminated UTF-8 JSON string. Free with `rmlib_string_free`.

**Errors:** does not fail.

### Example (Python)

```python
import json

lib.rmlib_available_modules.restype = ctypes.c_void_p
ptr = lib.rmlib_available_modules()
mods = json.loads(ctypes.string_at(ptr).decode())
lib.rmlib_string_free(ptr)

for m in mods:
    print(m["name"])
```

### Example (C)

```c
char *json = rmlib_available_modules();
/* parse json yourself; the crate does not ship a parser for C */
rmlib_string_free(json);
```

## Notes

- The strings returned here are short and stable — do not store them long-term.
- The JSON output of `rmlib_available_modules` is a single line with no whitespace; parse it with any JSON library.

## See also

- [Errors and Strings](error.md) — `rmlib_last_error`, `rmlib_string_free`
- [Core API](../lib.md) — the same information from the Rust side