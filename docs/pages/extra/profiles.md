# extra::profiles

Saved account profiles.

A profile is a small JSON document with the credentials and identifiers of a logged-in account, so a launcher can start the game without asking for the password / device code every time.

Profiles live under the auth directory:

```text
auth_path/profiles/ely/<username>.json
auth_path/profiles/ms/<username>.json
```

`auth_path` comes from `core::config` (`~/rml/auth` by default). Every function has an `_at` variant taking `Option<&Path>`:

- `Some(dir)` — use `dir` as the auth directory instead of the configured `auth_path`.
- `None` — resolve `auth_path` from `paths.json`.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["profiles"] }
```

```rust
use rmlib::extra::profiles::{Profile, ProfileKind, save_profile_at, load_profile_at, delete_profile_at, list_profiles_at, profile_path_at, profiles_dir_at, ProfileError};
```

`profiles` implies `core`.

## Types

```rust
pub enum ProfileKind {
    Ely,
    Ms,
}

pub struct Profile {
    pub username: String,
    pub uuid: Option<String>,
    pub access_token: String,
    pub client_token: Option<String>,
    pub xuid: Option<String>,
    pub has_license: Option<bool>,
}
```

- `ProfileKind::Ely` — an Ely.by account; the file goes to `profiles/ely/`.
- `ProfileKind::Ms` — an official Microsoft account; the file goes to `profiles/ms/`.

`ProfileKind::as_dir()` returns `"ely"` / `"ms"`.

Optional fields are omitted from the JSON when they are `None`.

### Example profile

```json
{
  "username": "Musek",
  "uuid": "069a79f4-44e9-4726-a5be-fca90e38aaf5",
  "access_token": "…",
  "client_token": "…",
  "has_license": true
}
```

## Store functions

Each function comes in two forms: `*_at` (explicit auth directory) and a plain variant (configured `auth_path`).

```rust
pub fn save_profile_at(auth_dir: Option<&Path>, kind: ProfileKind, profile: &Profile) -> Result<PathBuf, ProfileError>
pub fn save_profile(kind: ProfileKind, profile: &Profile) -> Result<PathBuf, ProfileError>

pub fn load_profile_at(auth_dir: Option<&Path>, kind: ProfileKind, username: &str) -> Result<Option<Profile>, ProfileError>
pub fn load_profile(kind: ProfileKind, username: &str) -> Result<Option<Profile>, ProfileError>

pub fn delete_profile_at(auth_dir: Option<&Path>, kind: ProfileKind, username: &str) -> Result<(), ProfileError>
pub fn delete_profile(kind: ProfileKind, username: &str) -> Result<(), ProfileError>

pub fn list_profiles_at(auth_dir: Option<&Path>, kind: ProfileKind) -> Result<Vec<String>, ProfileError>
pub fn list_profiles(kind: ProfileKind) -> Result<Vec<String>, ProfileError>
```

- `save_profile*` writes the file atomically, creating `profiles/<kind>/` if needed. Returns the written path.
- `load_profile*` returns `Ok(None)` if the file does not exist, and a `ProfileError::Json` error if it exists but cannot be parsed.
- `delete_profile*` removes the file; deleting a missing profile is not an error.
- `list_profiles*` returns the usernames (file stems), sorted. A missing directory yields an empty list.

```rust
use rmlib::extra::profiles::{load_profile, ProfileKind};

if let Some(profile) = load_profile(ProfileKind::Ely, "Musek")? {
    println!("uuid: {:?}", profile.uuid);
}
```

## Path helpers

```rust
pub fn profiles_dir_at(auth_dir: Option<&Path>, kind: ProfileKind) -> Result<PathBuf, ProfileError>
pub fn profile_path_at(auth_dir: Option<&Path>, kind: ProfileKind, username: &str) -> Result<PathBuf, ProfileError>
pub fn profile_path(kind: ProfileKind, username: &str) -> Result<PathBuf, ProfileError>
```

`profiles_dir_at` returns `<auth>/profiles/<kind>`. `profile_path*` returns `<auth>/profiles/<kind>/<username>.json`.

Usernames are validated: empty, `.`, `..`, and names containing `/` or `\` are rejected with `ProfileError::Io` (`InvalidInput`) to prevent path traversal.

## Error type

```rust
pub enum ProfileError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Config(musutils::fs::config::ConfigError),
}
```

| Variant | Cause |
|---|---|
| `Io` | Reading / writing / removing the file failed, or the username is invalid. |
| `Json` | An existing profile file is not valid JSON or does not match `Profile`. |
| `Config` | `paths.json` could not be loaded (so `auth_path` is unknown). |

## See also

- [Config](../core/config.md) — `auth_path`.
- [Auth](../core/auth.md) — official Microsoft login, whose `McProfile` maps onto `Profile`.
- [Ely.by](ely.md) — Ely.by login, whose `AuthResult` maps onto `Profile`.