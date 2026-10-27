# FFI — Overview

The C ABI (`rmlib_*` functions) lets you drive `rmlib` from any language that can load a shared library and call C functions: Python, C, C++, Go, Node.js, Java (via JNA/JNI), Ruby, PHP, Zig, and so on.

The crate produces a shared library on every `cargo build` (because `crate-type = ["rlib", "cdylib"]`), but the `rmlib_*` symbols are only present when the `ffi` feature is enabled:

```bash
make build-ffi          # → target/release/librmlib.so  (Linux)
                        #    target/release/librmlib.dylib (macOS)
                        #    target/release/rmlib.dll    (Windows)
make dist               # same, plus copies into dist/
```

The exports also honour the area features: `rmlib_fabric_*` only exists with `--features fabric`, `rmlib_neoforge_*` with `--features neoforge`, `rmlib_ely_*` with `--features ely`. The rest need `--features core`.

## ABI conventions

The whole C ABI follows four conventions. Every export obeys them; the individual pages only document the parameters and the specifics.

### Return types

| C type | Meaning | On error |
|---|---|---|
| `*mut c_char` | A NUL-terminated UTF-8 string. | `NULL` |
| `*mut c_void` | An opaque handle. | `NULL` |
| `int32_t` (`i32`) | A status code. | `-1` |

### Error reporting

When a function returns `NULL` or `-1`, the reason is stored in a thread-local slot. Read it with:

```c
char *err = rmlib_last_error();
if (err) {
    fprintf(stderr, "rmlib: %s\n", err);
    rmlib_string_free(err);
}
```

`rmlib_last_error()` **takes** the error — after it returns, the slot is empty. A second call returns `NULL` until another function fails. This is deliberate: the error string is a one-shot value, not persistent state.

The slot is thread-local: an error set on thread A is not visible on thread B.

### String ownership

All strings returned by `rmlib_*` functions are allocated by Rust. Free them with:

```c
void rmlib_string_free(char *s);
```

`rmlib_string_free(NULL)` is a no-op.

**Do not** pass the returned pointer to `free()`, `delete`, or any other allocator. The Rust allocator and the C allocator are different.

### Input strings

Every `*const c_char` input must be:

- non-`NULL` (unless the parameter is documented as optional);
- a valid NUL-terminated C string;
- valid UTF-8.

Functions reject `NULL` with an error where a value is required, and return an error for invalid UTF-8. They never dereference a `NULL` pointer as if it were valid.

### JSON for complex arguments

Functions that would otherwise take more than three arguments (`rmlib_version_install`, `rmlib_fabric_install`, `rmlib_neoforge_install`, `rmlib_run_client`, `rmlib_build_argv`, `rmlib_build_command`, `rmlib_ely_install_patch`) take a single **JSON string**. See [Installation](install.md) and [Run](run.md) for the schema.

Functions that return lists or structs (`rmlib_available_modules`, `rmlib_paths_load`, `rmlib_version_fetch_list`, `rmlib_version_sort_by_type`, `rmlib_version_strip_version`, `rmlib_build_argv`, `rmlib_ely_login`) return a **JSON string**. The schema matches the Rust types.

## Panics

Every export is wrapped in `catch_unwind`. If the underlying Rust code panics, the panic is caught, the message is stored via `rmlib_last_error()` (prefixed with `"panic: "`), and the function returns `NULL` / `-1` as if it had failed normally.

This means: **a panic in Rust is a recoverable error from the FFI caller's point of view.** The process does not abort.

## Thread safety

- Every export is safe to call from any thread.
- `rmlib_last_error()` is thread-local.
- `rmlib_start_beta_proxy` returns a handle that must be freed on the **same** thread (it holds an `Arc` with a `Drop` impl; freeing it on another thread is safe as long as the `Drop` runs exactly once — which `rmlib_proxy_guard_free` guarantees).
- Async work is serialised through a shared Tokio runtime (`src/ffi/runtime.rs`); the exports block on it. Calling an export from inside a Tokio runtime you started yourself will panic — do not do that.

## Calling from Python

The quickest smoke test:

```python
import ctypes

lib = ctypes.CDLL("./target/release/librmlib.so")

# Every function that returns a string.
lib.rmlib_version.restype = ctypes.c_void_p
lib.rmlib_last_error.restype = ctypes.c_void_p
lib.rmlib_string_free.argtypes = [ctypes.c_void_p]

ptr = lib.rmlib_version()
print(ctypes.string_at(ptr).decode())   # e.g. "0.10.0"
lib.rmlib_string_free(ptr)
```

Two rules to remember:

1. **`restype` must be `c_void_p`, not `c_char_p`.** With `c_char_p`, ctypes copies the bytes and drops the original pointer — you then cannot free it. Always `c_void_p` + `ctypes.string_at` + `rmlib_string_free`.
2. **Read the error before retrying.** If a function returned `NULL` / `-1`, the first thing to do is `rmlib_last_error()`.

A fuller example that calls `rmlib_build_argv`:

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
    "versions_path": "~/.rml/versions",
    "assets_path": "~/.rml/assets",
    "libs_path": "~/.rml/libs",
}
opts_json = json.dumps(opts).encode()

ptr = lib.rmlib_build_argv(opts_json)
if not ptr:
    err = lib.rmlib_last_error()
    print("error:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    argv = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    print(" ".join(argv))
```

## Calling from C

```c
#include <stdio.h>
#include <stdlib.h>
#include "rmlib.h"

int main(void) {
    char *version = rmlib_version();
    if (!version) {
        char *err = rmlib_last_error();
        fprintf(stderr, "rmlib_version failed: %s\n", err ? err : "(no message)");
        rmlib_string_free(err);
        return 1;
    }
    printf("rmlib %s\n", version);
    rmlib_string_free(version);
    return 0;
}
```

```bash
cc example.c -Ltarget/release -lrmlib -Wl,-rpath,target/release -o example
./example
```

## Reference index

| Page | Functions |
|---|---|
| [Library Info](lib.md) | `rmlib_version`, `rmlib_name`, `rmlib_client_token`, `rmlib_client_id`, `rmlib_available_modules` |
| [Errors and Strings](error.md) | `rmlib_last_error`, `rmlib_string_free` |
| [Config](config.md) | `rmlib_config_init`, `rmlib_get_versions_path`, `rmlib_get_libs_path`, `rmlib_get_assets_path`, `rmlib_paths_load`, `rmlib_paths_init` |
| [Version](version.md) | `rmlib_version_fetch_list`, `rmlib_version_get_json`, `rmlib_version_get_json_url`, `rmlib_version_sort_by_type`, `rmlib_version_strip_version`, `rmlib_build_maven_path` |
| [Installation](install.md) | `rmlib_version_install`, `rmlib_fabric_install`, `rmlib_fabric_fetch_list`, `rmlib_neoforge_install`, `rmlib_neoforge_fetch_list` |
| [Run](run.md) | `rmlib_run_client`, `rmlib_build_argv`, `rmlib_build_command` |
| [Ely.by](ely.md) | `rmlib_ely_login`, `rmlib_ely_install_patch`, `rmlib_start_beta_proxy`, `rmlib_proxy_guard_free` |

## See also

- [Installation](../getting-started/installation.md) — how to build the `.so` / `.dll` / `.dylib`
- [Features](../getting-started/features.md) — `ffi` feature and per-area gating