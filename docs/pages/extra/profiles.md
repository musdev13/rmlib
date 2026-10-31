# extra::profiles

Saved account profiles and launch instance files.

A **profile** is a small JSON document with the credentials and identifiers of a logged-in account, so a launcher can start the game without asking for the password / device code every time.

An **instance** is a JSON document with launch settings (version, RAM, loader, paths, flags) — no tokens.

Profiles live under the auth directory:

```text
auth_path/profiles/ely/<username>.json
auth_path/profiles/ms/<username>.json
```

Instance files live wherever the caller puts them; by default `<game_path>/rml.json` (`DEFAULT_INSTANCE_FILE`).

`auth_path` comes from `core::config` (`~/rml/auth` by default). Every profile function has an `_at` variant taking `Option<&Path>`:

- `Some(dir)` — use `dir` as the auth directory instead of the configured `auth_path`.
- `None` — resolve `auth_path` from `paths.json`.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["profiles"] }
```

```rust
use rmlib::extra::profiles::{Profile, ProfileKind, save_profile_at, load_profile_at, delete_profile_at, list_profiles_at, profile_path_at, profiles_dir_at, ProfileError};
use rmlib::extra::profiles::{InstanceConfig, DEFAULT_INSTANCE_FILE, instance_default_path, save_instance, load_instance};
```

`profiles` implies `core`.

## Profile types

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

## Profile store functions

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

Usernames are validated: empty, `.`, `..`, and names containing `/` or `\` are rejected with `ProfileError::Io` (`InvalidInput`) to prevent path traversal.

## Instance config

```rust
pub const DEFAULT_INSTANCE_FILE: &str = "rml.json";

pub struct InstanceConfig {
    pub version_id: Option<String>,
    pub ram: Option<String>,
    pub username: Option<String>,
    pub ely: Option<bool>,
    pub betafix: Option<bool>,
    pub profile: Option<bool>,
    pub jvm_args: Option<String>,
    pub game_args: Option<String>,
    pub versions: Option<PathBuf>,
    pub assets: Option<PathBuf>,
    pub libs: Option<PathBuf>,
    pub game_path: Option<PathBuf>,
    pub spawn_path: Option<PathBuf>,
    pub java: Option<PathBuf>,
    pub fabric: Option<String>,
    pub neoforge: Option<String>,
    pub auth_path: Option<PathBuf>,
}
```

Launch settings without account secrets: there is no `uuid`, `token` or `xuid`. Only `username` (and the `profile` flag) relate to the account.

Every field is `Option`; a field that is `None` is omitted from the JSON, so the file stays sparse and portable. Loading a sparse file gives `None` for absent fields.

### Functions

```rust
pub fn instance_default_path(game_path: &Path) -> PathBuf
pub fn save_instance(path: &Path, cfg: &InstanceConfig) -> Result<(), ProfileError>
pub fn load_instance(path: &Path) -> Result<InstanceConfig, ProfileError>
```

- `instance_default_path(game_path)` returns `<game_path>/rml.json`.
- `save_instance` writes the file atomically, creating parent directories.
- `load_instance` errors with `ProfileError::Io` (`NotFound`) if the file is absent, and `ProfileError::Json` if it cannot be parsed.

```rust
use std::path::Path;
use rmlib::extra::profiles::{InstanceConfig, instance_default_path, save_instance};

let game = Path::new("./game");
let cfg = InstanceConfig {
    version_id: Some("26.3".to_string()),
    ram: Some("6G".to_string()),
    fabric: Some("0.20.1".to_string()),
    ..Default::default()
};
save_instance(&instance_default_path(game), &cfg)?;
```

### Merging

```rust
pub fn merge_missing_from(&mut self, other: &InstanceConfig)
```

Fills fields that are `None` in `self` from `other`; fields already set in `self` are kept. This is what a launcher uses to layer sources:

1. command-line flags (highest priority),
2. the instance file,
3. the profile (for the account),
4. built-in defaults.

```rust
let mut cfg = InstanceConfig { username: Some("Nonamepromaks".into()), ..Default::default() };
cfg.merge_missing_from(&loaded_from_file); // fills the rest, keeps the username
```

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
| `Io` | Reading / writing / removing the file failed, the file is missing, or a username / path is invalid. |
| `Json` | A file exists but is not valid JSON or does not match the expected type. |
| `Config` | `paths.json` could not be loaded (so `auth_path` is unknown). |

## See also

- [Config](../core/config.md) — `auth_path`.
- [Auth](../core/auth.md) — official Microsoft login, whose `McProfile` maps onto `Profile`.
- [Ely.by](ely.md) — Ely.by login, whose `AuthResult` maps onto `Profile`.