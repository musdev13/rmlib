# FFI — Auth

Official Microsoft / Minecraft account authentication.

These exports mirror [`core::auth`](../core/auth.md) — see that page for the workflow, token storage, and the `McProfile` / `MsTokens` shapes.

All exports on this page require the `core` feature **and** the `ffi` feature.

```c
#include "rmlib.h"
```

Each operation has two exports:

- a plain one (no `dir` parameter) that uses `auth_path` from `paths.json`;
- an `_at` one taking `const char *dir` — a custom auth directory, or `NULL` to use the configured `auth_path` (equivalent to the plain variant).

## `rmlib_ms_auth_login` / `rmlib_ms_auth_login_at`

```c
char *rmlib_ms_auth_login(void);
char *rmlib_ms_auth_login_at(const char *dir);
```

Runs the Microsoft device-code login flow (`core::auth::login` / `login_at`): prints the verification URL and user code to stdout, waits for the user to confirm, then saves the tokens to `ms.json` (in `dir`, or in `auth_path` when `dir` is `NULL`).

**This call blocks** until the user completes the login in a browser or the device code expires.

### Parameters

| Parameter | Meaning |
|---|---|
| `dir` | Directory where `ms.json` is saved, or `NULL` to use `auth_path`. Ignored by the plain variant. |

### Returns

NUL-terminated UTF-8 JSON object with the saved tokens:

```json
{
  "access_token": "…",
  "refresh_token": "…",
  "expires_in": 3600
}
```

Free with `rmlib_string_free`.

### Errors

Returns `NULL` and sets `rmlib_last_error` on any failure (network, expired device code, I/O). The message is the `Display` output of `AuthError`.

### Example (Python)

```python
import json

lib.rmlib_ms_auth_login.restype = ctypes.c_void_p
ptr = lib.rmlib_ms_auth_login()
if not ptr:
    err = lib.rmlib_last_error()
    print("login failed:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    tokens = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    print("logged in, refresh_token:", tokens["refresh_token"][:8], "…")
```

Custom directory:

```python
lib.rmlib_ms_auth_login_at.argtypes = [ctypes.c_char_p]
lib.rmlib_ms_auth_login_at.restype = ctypes.c_void_p

ptr = lib.rmlib_ms_auth_login_at(b"/home/me/other-auth")
# … same handling as above
```

## `rmlib_ms_auth_refresh` / `rmlib_ms_auth_refresh_at`

```c
char *rmlib_ms_auth_refresh(void);
char *rmlib_ms_auth_refresh_at(const char *dir);
```

Refreshes the tokens stored in `ms.json` (`core::auth::refresh` / `refresh_at`), saves the new tokens, and returns them. No user interaction.

The `_at` variant reads the existing `refresh_token` from `dir/ms.json`; the plain variant uses `auth_path`.

### Returns

Same JSON shape as `rmlib_ms_auth_login`. Free with `rmlib_string_free`.

### Errors

Returns `NULL` and sets `rmlib_last_error` if:

- there is no saved `ms.json` in the target directory (`"no saved ms tokens; run rmlib_ms_auth_login first"`),
- the refresh token is expired or revoked,
- the network or filesystem fails.

### Example (C)

```c
char *tokens = rmlib_ms_auth_refresh();
if (!tokens) {
    char *err = rmlib_last_error();
    fprintf(stderr, "refresh failed: %s\n", err ? err : "(no message)");
    rmlib_string_free(err);
} else {
    printf("%s\n", tokens);
    rmlib_string_free(tokens);
}
```

## `rmlib_ms_auth_get_data` / `rmlib_ms_auth_get_data_at`

```c
char *rmlib_ms_auth_get_data(void);
char *rmlib_ms_auth_get_data_at(const char *dir);
```

High-level entry point (`core::auth::get_data` / `get_data_at`): loads `ms.json` from the target directory, refreshes the tokens, falls back to the device-code login if refreshing fails or no tokens exist, then runs the Xbox Live → XSTS → Minecraft profile chain.

**This call can block** on the device-code login if no valid tokens are available.

### Returns

NUL-terminated UTF-8 JSON object — the `McProfile`:

```json
{
  "username": "Steve",
  "uuid": "…",
  "access_token": "…",
  "xuid": "…",
  "has_license": true
}
```

For an account without a Minecraft license, `has_license` is `false` and the license-dependent fields are `"no_license"` (see [core::auth](../core/auth.md#types) for the exact table). This is **not** an error — the call still returns a JSON object.

Free with `rmlib_string_free`.

### Errors

Returns `NULL` and sets `rmlib_last_error` only on real failures (network, I/O, malformed response). A missing license is reported inside the JSON via `has_license: false`.

### Example (Python)

```python
import json

lib.rmlib_ms_auth_get_data.restype = ctypes.c_void_p
ptr = lib.rmlib_ms_auth_get_data()
if not ptr:
    err = lib.rmlib_last_error()
    print("auth failed:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    profile = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    if not profile["has_license"]:
        print("warning: account has no Minecraft license")
    print(profile["username"], profile["uuid"])
```

## `rmlib_ms_auth_clear` / `rmlib_ms_auth_clear_at`

```c
int32_t rmlib_ms_auth_clear(void);
int32_t rmlib_ms_auth_clear_at(const char *dir);
```

Deletes `ms.json` from the target directory (`core::auth::clear_tokens` / `clear_tokens_at`) — logs the account out on this machine.

### Returns

| Value | Meaning |
|---|---|
| `0` | The file was removed, or did not exist. |
| `-1` | I/O error or panic. The message is available via `rmlib_last_error`. |

### Example (C)

```c
if (rmlib_ms_auth_clear() != 0) {
    char *err = rmlib_last_error();
    fprintf(stderr, "clear failed: %s\n", err ? err : "(no message)");
    rmlib_string_free(err);
}
```

## Typical flow

```text
rmlib_ms_auth_get_data()      → everything: refresh or login, then profile
rmlib_ms_auth_login()         → force an interactive login
rmlib_ms_auth_refresh()       → refresh an existing session
rmlib_ms_auth_clear()         → log out
```

For most callers `rmlib_ms_auth_get_data` is the only export needed.

Pass a `dir` to any `_at` variant to keep a separate account/session in a custom directory; pass `NULL` to use the `auth_path` from `paths.json`.

## See also

- [Errors and Strings](error.md) — reading `rmlib_last_error` and freeing strings
- [Auth](../core/auth.md) — the Rust-side behavior, token storage and `McProfile` shape
- [Ely.by](ely.md) — the alternative (non-official) auth flow