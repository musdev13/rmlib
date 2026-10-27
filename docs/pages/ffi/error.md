# FFI — Errors and Strings

Two functions every caller needs: `rmlib_last_error` to read the reason a call failed, and `rmlib_string_free` to release strings returned by `rmlib_*` exports.

Both are always available when `ffi` is enabled — they do not depend on any area feature.

## `rmlib_last_error`

```c
char *rmlib_last_error(void);
```

Returns the error message produced by the most recent failed `rmlib_*` call on the current thread, or `NULL` if no error is pending.

### Semantics

- **Thread-local.** The error slot is per-thread. An error raised on thread A is not visible on thread B. Each thread has its own last-error.
- **One-shot.** Reading the error **clears** it. A second call returns `NULL` unless another function failed in between.
- **Panic-aware.** If the underlying Rust code panicked, the string starts with `"panic: "` followed by the panic payload (the `&str` / `String` that was passed to `panic!`). Otherwise the message comes from the function's own error path (e.g. `"invalid maven coordinate"`, `"missing version_id"`, `"invalid json: …"`).

### Returns

| Case | Return value |
|---|---|
| An error is pending on this thread | NUL-terminated UTF-8 string. Free with `rmlib_string_free`. |
| No error pending | `NULL`. |

**Returns:** `*mut c_char`.

**Errors:** does not fail.

### When the slot is filled

The slot is written by every `rmlib_*` export that can fail, at the moment it fails:

- Return `NULL` for pointer-returning functions (e.g. `rmlib_version_get_json`, `rmlib_build_maven_path`, `rmlib_ely_login`).
- Return `-1` for integer-returning functions (e.g. `rmlib_config_init`, `rmlib_version_install`, `rmlib_run_client`).

Successful calls leave the slot untouched — an old error from a previous call is **not** cleared by a later success. This is a common source of confusion: always check the return value first, then read the error only if the return value indicates failure.

### Usage pattern

The correct order in any language:

```text
result = call_function(...)
if result indicates failure:
    err = rmlib_last_error()
    handle err
    rmlib_string_free(err)
```

Never read `rmlib_last_error()` "just in case" — you would consume the slot and lose the real error.

### Example (Python)

```python
import ctypes

lib = ctypes.CDLL("./target/release/librmlib.so")
lib.rmlib_last_error.restype = ctypes.c_void_p
lib.rmlib_string_free.argtypes = [ctypes.c_void_p]
lib.rmlib_build_maven_path.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
lib.rmlib_build_maven_path.restype = ctypes.c_void_p

ptr = lib.rmlib_build_maven_path(b"bad", None)
if not ptr:
    err = lib.rmlib_last_error()
    print("error:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
    # second read returns NULL:
    assert not lib.rmlib_last_error()
```

### Example (C)

```c
char *p = rmlib_build_maven_path("bad", NULL);
if (p == NULL) {
    char *err = rmlib_last_error();
    fprintf(stderr, "error: %s\n", err ? err : "(no message)");
    rmlib_string_free(err);
} else {
    printf("%s\n", p);
    rmlib_string_free(p);
}
```

## `rmlib_string_free`

```c
void rmlib_string_free(char *s);
```

Frees a string previously returned by any `rmlib_*` export.

### Semantics

- **`NULL`-safe.** `rmlib_string_free(NULL)` is a no-op; it does not panic, does not corrupt anything.
- **One-shot.** Each returned pointer must be freed exactly once. Freeing it twice is undefined behaviour (double free), same as any C allocator.
- **Rust allocator.** The pointer comes from Rust (`CString::into_raw`). It must be freed by `rmlib_string_free`, **not** by `free()`, `delete`, `std::free`, `gc` finalisers, or any other allocator.

### Returns

Nothing.

**Errors:** does not fail. Passing an invalid pointer (not produced by an `rmlib_*` export, or already freed) is UB — but passing `NULL` is fine.

### Which strings to free

Every `*mut c_char` returned by an `rmlib_*` export must be freed:

| Function | Returned string |
|---|---|
| `rmlib_version` | Yes |
| `rmlib_name` | Yes |
| `rmlib_client_token` | Yes |
| `rmlib_client_id` | Yes |
| `rmlib_available_modules` | Yes |
| `rmlib_last_error` | Yes (when non-`NULL`) |
| `rmlib_get_versions_path` | Yes |
| `rmlib_get_libs_path` | Yes |
| `rmlib_get_assets_path` | Yes |
| `rmlib_paths_load` | Yes |
| `rmlib_version_fetch_list` | Yes |
| `rmlib_version_get_json` | Yes |
| `rmlib_version_get_json_url` | Yes |
| `rmlib_version_sort_by_type` | Yes |
| `rmlib_version_strip_version` | Yes |
| `rmlib_build_maven_path` | Yes |
| `rmlib_fabric_fetch_list` | Yes |
| `rmlib_neoforge_fetch_list` | Yes |
| `rmlib_build_argv` | Yes |
| `rmlib_build_command` | Yes |
| `rmlib_ely_login` | Yes |

The opaque `*mut c_void` handles (`rmlib_start_beta_proxy`) are **not** strings — free them with `rmlib_proxy_guard_free` (see [Ely.by](ely.md)).

### Example (Python)

```python
lib.rmlib_string_free.argtypes = [ctypes.c_void_p]

ptr = lib.rmlib_version()
if ptr:
    print(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)

lib.rmlib_string_free(None)   # no-op, safe
```

### Example (C)

```c
char *v = rmlib_version();
rmlib_string_free(v);
rmlib_string_free(NULL);   /* no-op */
```

## Common mistakes

### Using `c_char_p` as `restype` in Python

```python
lib.rmlib_version.restype = ctypes.c_char_p        # WRONG
s = lib.rmlib_version()
# `s` is now `bytes`; the original pointer is lost.
# rmlib_string_free(s) crashes, because ctypes made a temporary.
```

Use `c_void_p` instead:

```python
lib.rmlib_version.restype = ctypes.c_void_p       # RIGHT
ptr = lib.rmlib_version()
s = ctypes.string_at(ptr).decode()
lib.rmlib_string_free(ptr)
```

### Freeing with `free()`

```c
char *v = rmlib_version();
free(v);                 /* WRONG — different allocator */
```

Always `rmlib_string_free`.

### Reading `last_error` after a success

```python
ptr = lib.rmlib_version()      # succeeds
err = lib.rmlib_last_error()   # returns whatever the previous failure left behind
```

The slot is not cleared on success. Only call `rmlib_last_error()` when a return value indicates failure.

### Freeing twice

```python
lib.rmlib_string_free(ptr)
lib.rmlib_string_free(ptr)     # WRONG — double free
```

`rmlib_string_free` guards against `NULL` but not against double frees. Null your pointer after freeing it if you are not sure.

## See also

- [Overview](index.md) — ABI conventions
- [Library Info](lib.md) — the functions whose return values are freed here
- [Ely.by](ely.md) — `rmlib_proxy_guard_free`, the non-string counterpart