# core::auth

Official Microsoft / Minecraft account authentication.

This is the counterpart of [`extra::ely::auth`](../extra/ely.md) for official Mojang/Microsoft accounts: device-code login, token refresh, and a full Xbox Live → XSTS → Minecraft Services profile flow.

Credentials are stored in `auth_path/ms.json` (see [`core::config`](config.md)). `auth_path` defaults to `~/rml/auth`.

## Enable

`core::auth` is part of the `core` feature and requires no extra flags:

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

```rust
use rmlib::core::auth::{get_data, login, refresh, load_tokens, save_tokens, clear_tokens, tokens_file, McProfile, MsTokens, AuthError};
```

## Types

```rust
pub struct MsTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: Option<u64>,
}

pub struct McProfile {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub xuid: String,
    pub has_license: bool,
}
```

`MsTokens` is what is written to `ms.json`. `McProfile` is the result of the Minecraft-side flow.

`has_license` is `false` when the account has no Minecraft license. In that case the fields that require a license are set to the `"no_license"` placeholder, while values that could still be obtained (for example `xuid`) are kept:

| Field | With license | Without license (early failure) | Without license (MC profile 404) |
|---|---|---|---|
| `username` | profile name | `"no_license"` | `"no_license"` |
| `uuid` | profile id | `"no_license"` | `"no_license"` |
| `access_token` | Minecraft access token | `"no_license"` | Minecraft access token |
| `xuid` | Xbox user id | Xbox user id | Xbox user id |
| `has_license` | `true` | `false` | `false` |

## `login`

```rust
pub async fn login() -> Result<MsTokens, AuthError>
```

Runs the Microsoft **device code** flow:

1. Requests a device code and prints the verification URL and user code to stdout (via `musutils::types::Status::Note`).
2. Polls the token endpoint until the user confirms in the browser (honours `authorization_pending` / `slow_down`, fails on `expired_token`).
3. Saves the resulting tokens to `auth_path/ms.json` and returns them.

This is interactive: the user has to open the printed URL and enter the code.

## `refresh`

```rust
pub async fn refresh(refresh_token: &str) -> Result<MsTokens, AuthError>
```

Exchanges a `refresh_token` for a new `MsTokens` without user interaction, and saves the refreshed tokens to `ms.json` (Microsoft rotates the refresh token, so persisting it matters).

Returns `Err(AuthError::Status { .. })` if the token is expired or revoked.

## `get_profile`

```rust
pub async fn get_profile(ms_access_token: &str) -> Result<McProfile, AuthError>
```

Runs the Xbox Live → XSTS → Minecraft Services chain with a Microsoft access token and returns an `McProfile`.

If Minecraft Services rejects the login or returns `404` for the profile, the returned `McProfile` has `has_license = false` and placeholder fields as described above — this is not an error.

## `get_data`

```rust
pub async fn get_data() -> Result<McProfile, AuthError>
```

The high-level entry point. Workflow:

1. `load_tokens()` — read `ms.json` if it exists.
2. If tokens are present, `refresh()` them. If the refresh fails (expired / revoked), fall back to `login()`.
3. If no tokens are present, `login()`.
4. `get_profile()` with the resulting Microsoft access token, and return the `McProfile`.

```rust
use rmlib::core::auth::get_data;

let profile = get_data().await?;
if !profile.has_license {
    eprintln!("warning: account has no Minecraft license");
}
println!("{} ({})", profile.username, profile.uuid);
```

## Token storage

```rust
pub fn save_tokens(tokens: &MsTokens) -> Result<(), AuthError>
pub fn load_tokens() -> Result<Option<MsTokens>, AuthError>
pub fn clear_tokens() -> Result<(), AuthError>
pub fn tokens_file() -> Result<PathBuf, AuthError>
```

- `save_tokens` writes `auth_path/ms.json` atomically (creating `auth_path` if needed).
- `load_tokens` returns `Ok(None)` if the file is absent **or** cannot be parsed — a corrupt file is treated as "not logged in" rather than an error.
- `clear_tokens` removes `ms.json` if it exists.
- `tokens_file` returns the resolved path, useful for diagnostics.

```rust
use rmlib::core::auth::{clear_tokens, tokens_file};

println!("tokens are stored at {}", tokens_file()?.display());
clear_tokens()?; // log out
```

## Error type

```rust
pub enum AuthError {
    Io(std::io::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
    Config(musutils::fs::config::ConfigError),
    Status { code: u16, body: String },
    DeviceCode { error: String, description: Option<String> },
    InvalidResponse(String),
    DeviceCodeExpired,
}
```

| Variant | Cause |
|---|---|
| `Io` | `ms.json` cannot be read or written. |
| `Http` | A request failed at the transport level. |
| `Json` | A response body could not be deserialized. |
| `Config` | `paths.json` could not be loaded (so `auth_path` is unknown). |
| `Status { .. }` | A non-success HTTP status that is not handled as "no license". |
| `DeviceCode { .. }` | The device-code endpoint returned an unexpected error. |
| `InvalidResponse(_)` | A response was well-formed JSON but missing expected fields. |
| `DeviceCodeExpired` | The user did not confirm the device code in time. |

## Notes

- The flow requires network access to `login.live.com`, `*.auth.xboxlive.com` and `api.minecraftservices.com`.
- `core::auth` uses `reqwest` directly (like `extra::ely::auth`); there is no shared POST helper in `musutils` yet.
- The module is behind the `core` feature only — it does not depend on `extra`.

## See also

- [Config](config.md) — `auth_path` and `ms.json` location.
- [extra::Ely.by](../extra/ely.md) — the alternative (non-official) auth flow.
- [FFI — Auth](../ffi/auth.md) — C exports for this module.