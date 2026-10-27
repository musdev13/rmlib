# core::config

Launcher configuration: where versions, libraries and assets live on disk.

The configuration file is a small JSON document stored under the platform config directory:

- Linux / macOS: `~/.config/rml/paths.json`
- Windows: `%APPDATA%\rml\paths.json`

It contains three string fields:

```json
{
  "versions_path": "~/rml/versions",
  "assets_path": "~/rml/assets",
  "libs_path": "~/rml/libs"
}
```

`~` is expanded when the values are used, not when the file is written.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

```rust
use rmlib::core::config::{init, Paths, get_versions_path, get_libs_path, get_assets_path};
```

## `DEFAULT_PATHS_CONFIG`

```rust
pub static DEFAULT_PATHS_CONFIG: LazyLock<serde_json::Value>
```

The default configuration content, as a `serde_json::Value`:

```json
{
  "versions_path": "~/rml/versions",
  "assets_path": "~/rml/assets",
  "libs_path": "~/rml/libs"
}
```

Useful if you need to render the defaults to a string yourself:

```rust
use rmlib::core::config::DEFAULT_PATHS_CONFIG;

let pretty = serde_json::to_string_pretty(&*DEFAULT_PATHS_CONFIG).unwrap();
```

## `Paths`

```rust
pub struct Paths {
    pub versions: String,
    pub libs: String,
    pub assets: String,
}
```

The raw (unexpanded) values from `paths.json`. Derives `Debug`, `Clone`.

- `versions` — the `versions_path` field.
- `libs` — the `libs_path` field.
- `assets` — the `assets_path` field.

### `Paths::load`

```rust
pub fn load() -> Result<Self, ConfigError>
```

Loads `paths.json`, creating it from `DEFAULT_PATHS_CONFIG` if it does not exist.

Returns:

- `Ok(Paths { versions, libs, assets })` — the three fields as stored (possibly `~/rml/...`).
- `Err(ConfigError)` — I/O error while creating or reading the file, parse error, or a missing field (mapped to `ConfigError::KeyNotFound`).

```rust
use rmlib::core::config::Paths;

let paths = Paths::load().unwrap();
println!("versions: {}", paths.versions);
```

### `Paths::init`

```rust
pub fn init(force: bool) -> Result<(), ConfigError>
```

Creates the configuration file if it does not exist.

- `force = false` — creates the file only if missing; leaves an existing file untouched.
- `force = true` — rewrites the file with `DEFAULT_PATHS_CONFIG`, discarding any user edits.

Returns `Ok(())` on success, `Err(ConfigError)` on I/O failure.

```rust
use rmlib::core::config::Paths;

// First-time setup, do not clobber an existing file.
Paths::init(false).unwrap();

// Reset to defaults.
Paths::init(true).unwrap();
```

### `Paths::versions_pathbuf`

```rust
pub fn versions_pathbuf(&self) -> PathBuf
```

Returns `versions` with `~` expanded and the path normalized. Does **not** create the directory.

```rust
use rmlib::core::config::Paths;

let paths = Paths::load().unwrap();
let versions: std::path::PathBuf = paths.versions_pathbuf();
assert!(versions.is_absolute());
```

### `Paths::libs_pathbuf`

```rust
pub fn libs_pathbuf(&self) -> PathBuf
```

Same as `versions_pathbuf`, but for `libs`.

### `Paths::assets_pathbuf`

```rust
pub fn assets_pathbuf(&self) -> PathBuf
```

Same as `versions_pathbuf`, but for `assets`.

## `init`

```rust
pub fn init(force: bool)
```

Convenience wrapper that calls `Paths::init(force)` and prints status messages to stdout / stderr.

- On success, prints a hint pointing to `~/.config` or `%APPDATA%`.
- On failure, prints the error to stderr; does **not** panic.

Prefer `Paths::init` directly if you want to handle the error programmatically.

```rust
rmlib::core::config::init(false);
```

## `get_versions_path`

```rust
pub fn get_versions_path() -> String
```

Shorthand for `Paths::load().unwrap().versions`.

**Panics** if the configuration cannot be loaded. Use `Paths::load()` if you need the error.

```rust
let versions = rmlib::core::config::get_versions_path();
```

## `get_libs_path`

```rust
pub fn get_libs_path() -> String
```

Shorthand for `Paths::load().unwrap().libs`. Panics on failure.

```rust
let libs = rmlib::core::config::get_libs_path();
```

## `get_assets_path`

```rust
pub fn get_assets_path() -> String
```

Shorthand for `Paths::load().unwrap().assets`. Panics on failure.

```rust
let assets = rmlib::core::config::get_assets_path();
```

## Example: first run

A typical "first launch" flow:

```rust
use rmlib::core::config::{Paths, init};

init(false);

let paths = Paths::load().expect("config is unreadable");
println!("versions: {:?}", paths.versions_pathbuf());
println!("libs:     {:?}", paths.libs_pathbuf());
println!("assets:   {:?}", paths.assets_pathbuf());
```

After this, the three directories are the values the launcher will use for installation and launches. They are not created — `core::version::install` creates them on demand.

## Error type

Functions that can fail return `musutils::fs::config::ConfigError`:

| Variant | Cause |
|---|---|
| `ConfigError::Io(_)` | The file cannot be created, read or written. |
| `ConfigError::Parse(_)` | The file is not valid JSON, or `DEFAULT_PATHS_CONFIG` failed to serialize (should not happen). |
| `ConfigError::KeyNotFound(_)` | `paths.json` is missing one of `versions_path` / `libs_path` / `assets_path`. |
| `ConfigError::TypeMismatch { .. }` | A field exists but is not a string. |
| `ConfigError::InvalidKeyPath(_)` | Not expected from this module. |

## See also

- [Installation](../getting-started/installation.md)
- [core::run — Paths](run/paths.md) — how these strings are used at launch time.