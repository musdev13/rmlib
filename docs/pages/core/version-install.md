# core::version::install

Downloading and installing a Minecraft version: client JAR, libraries, assets, native libraries.

The full install process for a vanilla version:

1. `resolve_version_data` — pick the version (from the manifest or a local JSON).
2. `resolve_installation_paths` — compute the versions/libs/assets directories.
3. `prepare_version_files` — create directories and write the version JSON.
4. `download_client_jar` — fetch the client JAR.
5. `download_libraries` — fetch all libraries (with SHA-1 verification).
6. `extract_native_libraries` — unpack `.so` / `.dll` / `.dylib` from the libraries into `natives/`.
7. `process_assets` — fetch the asset index and all asset objects.

`install` orchestrates all of the above.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

```rust
use rmlib::core::version::install::{
    install, download_client_jar, download_libraries, extract_native_libraries,
    prepare_version_files, process_assets, resolve_installation_paths, resolve_version_data,
};
```

## `install`

```rust
#[allow(clippy::too_many_arguments)]
pub async fn install(
    version_id: Option<String>,
    json_path: Option<PathBuf>,
    versions_path_option: Option<PathBuf>,
    libs_path_option: Option<PathBuf>,
    assets_path_option: Option<PathBuf>,
    soft: bool,
    iclient: bool,
    ilibs: bool,
    iassets: bool,
    alibs: usize,
    aassets: usize,
)
```

Installs a vanilla Minecraft version.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_id` | Install this version by ID (fetched from Mojang). Conflicts with `json_path`. If both are `None`, the process exits with an error. |
| `json_path` | Install from a local JSON file. Used only if `version_id` is `None`. |
| `versions_path_option` | Override the versions directory. `None` uses `config::get_versions_path()`. |
| `libs_path_option` | Override the libraries directory. `None` uses `config::get_libs_path()`. |
| `assets_path_option` | Override the assets directory. `None` uses `config::get_assets_path()`. |
| `soft` | If `true`, skip already-present files whose SHA-1 (when available) matches. If `false`, redownload everything. |
| `iclient` | Download the client JAR. |
| `ilibs` | Download libraries and extract natives. |
| `iassets` | Download the asset index and asset objects. |
| `alibs` | Max concurrent library downloads. Suggested `150`. |
| `aassets` | Max concurrent asset downloads. Suggested `350`. |

### Behaviour

- Prints progress messages (`Status::Ok` / `Status::Task` / `Status::Err` in colored output) to stdout / stderr.
- **Panics** on network errors in `resolve_version_data` (via `get_version_json`), in `download_client_jar`, and in `prepare_version_files` if the JSON cannot be written.
- Library and asset downloads **do not panic on network failure**. Instead, each phase collects errors and prints them; the install completes with a summary. Rerunning with the same parameters (and `soft = true`) resumes from where it left off.

### Example

```rust
use rmlib::core::version::install::install;

install(
    Some("1.20.1".into()),
    None,
    None, None, None,
    true,   // soft
    true,   // iclient
    true,   // ilibs
    true,   // iassets
    150,    // alibs
    350,    // aassets
).await;
```

## `resolve_version_data`

```rust
pub async fn resolve_version_data(
    version_id: Option<String>,
    json_path: Option<PathBuf>,
) -> (String, String)
```

Resolves the version to install and returns `(version_id, json_text)`.

- `Some(id)` → fetches `get_version_json(id)` from Mojang. Panics on network or JSON error.
- `None, Some(path)` → reads `path` (after tilde expansion), parses it, extracts the `id` field. Panics on read/parse/missing-`id` error.
- `None, None` → prints `Status::Err: neither version_id nor json_path was provided` and exits the process (`std::process::exit(1)`).

```rust
let (id, json) = resolve_version_data(Some("1.20.1".into()), None).await;
println!("{id}");
```

This function is exported mainly because `install` calls it; it is not usually called directly.

## `resolve_installation_paths`

```rust
pub fn resolve_installation_paths(
    versions_path_option: Option<PathBuf>,
    libs_path_option: Option<PathBuf>,
    assets_path_option: Option<PathBuf>,
) -> (bool, bool, bool, PathBuf, PathBuf, PathBuf)
```

Resolves the three directory paths, using overrides where given and the `core::config` values otherwise.

Returns a 6-tuple:

| Position | Meaning |
|---|---|
| 0 | `true` if `versions_path_option` was `Some` |
| 1 | `true` if `libs_path_option` was `Some` |
| 2 | `true` if `assets_path_option` was `Some` |
| 3 | the resolved versions path (tilde-expanded) |
| 4 | the resolved libraries path (tilde-expanded) |
| 5 | the resolved assets path (tilde-expanded) |

The three booleans are used by `install` only to print `*` marks in the log, so you can see at a glance which paths were overridden.

```rust
use std::path::PathBuf;
use rmlib::core::version::install::resolve_installation_paths;

let (custom_v, custom_l, custom_a, versions, libs, assets) =
    resolve_installation_paths(
        Some(PathBuf::from("/tmp/versions")),
        None,
        None,
    );
assert!(custom_v);
assert!(!custom_l);
assert!(!custom_a);
```

## `prepare_version_files`

```rust
pub fn prepare_version_files(
    versions_path: &Path,
    libs_path: &Path,
    assets_path: &Path,
    version_id: &str,
    version_json_str: &str,
    line: &str,
) -> PathBuf
```

Creates the directory structure for a version and writes its JSON.

- Creates `versions_path`, `libs_path`, `assets_path` if missing (via `musutils::fs::new_dir`).
- Creates `versions_path/<version_id>/`.
- Writes `version_json_str` to `versions_path/<version_id>/<version_id>.json`.
- Returns the version directory path.

**Panics** if the JSON cannot be written.

`line` is the horizontal separator string used in the progress log — typically produced by `musutils::types::line::draw_colored('=', 35, Colors::Yellow)`. Pass any string; it is only used for display.

```rust
use std::path::Path;
use rmlib::core::version::install::prepare_version_files;

let dir = prepare_version_files(
    Path::new("/tmp/versions"),
    Path::new("/tmp/libs"),
    Path::new("/tmp/assets"),
    "1.20.1",
    r#"{"id":"1.20.1","mainClass":"net.minecraft.client.main.Main"}"#,
    "==========",
);
assert!(dir.ends_with("1.20.1"));
```

## `download_client_jar`

```rust
pub async fn download_client_jar(
    version_json: &serde_json::Value,
    version_path: &Path,
    version_id: &str,
    soft: bool,
)
```

Downloads the client JAR (`downloads.client.url` in the version JSON) to `version_path/<version_id>.jar`.

- Reads `downloads.client.url` and `downloads.client.sha1` from `version_json`.
- **Panics** if `downloads`, `client`, `sha1` or `url` is missing or the wrong type.
- With `soft = true`, if the JAR already exists and its SHA-1 matches, it is skipped.
- Otherwise calls `musutils::http::download_to_sha1` with 3 retries. **Panics** if all retries fail.

```rust
use std::path::Path;
use rmlib::core::version::install::download_client_jar;

let version_json: serde_json::Value = serde_json::from_str(&json_text).unwrap();
download_client_jar(
    &version_json,
    Path::new("/tmp/versions/1.20.1"),
    "1.20.1",
    true,
).await;
```

## `download_libraries`

```rust
pub async fn download_libraries(
    version_json: &serde_json::Value,
    libs_path: &Path,
    line: &str,
    alibs: usize,
    soft: bool,
) -> Result<(), Vec<HttpError>>
```

Downloads every library the version needs into `libs_path`, following each library's `downloads` metadata (or falling back to `build_maven_path`).

- Filters out libraries whose `rules` disallow the current OS/arch.
- Downloads natives for the current OS (`natives-<os>` classifiers).
- Skips native classifiers that do not match the current architecture.
- Skips files that already exist when `soft = true`.
- Uses `musutils::http::AsyncDownloader` with `alibs` concurrent workers and SHA-1 verification.

Returns:

- `Ok(())` — every download succeeded.
- `Err(Vec<HttpError>)` — one entry per failed download. The caller is expected to print the list and let the user rerun `install`.

```rust
use std::path::Path;
use rmlib::core::version::install::download_libraries;

match download_libraries(&version_json, Path::new("/tmp/libs"), "====", 150, true).await {
    Ok(()) => println!("all libraries downloaded"),
    Err(errors) => eprintln!("{} downloads failed", errors.len()),
}
```

`line` is the horizontal separator used in progress messages, same as `prepare_version_files`.

## `extract_native_libraries`

```rust
pub fn extract_native_libraries(libs_path: &Path)
```

Walks `libs_path`, opens every `.jar`, and extracts any `.so` / `.dll` / `.dylib` (whichever matches the current OS) into `libs_path/natives/`.

- Creates `libs_path/natives/` if missing.
- Skips `.jar` files that contain no native-extension files.
- Skips entries under `META-INF/`.
- Flattens the destination: files are written directly into `natives/`, not into subdirectories.
- Silently ignores I/O errors during extraction.

```rust
use std::path::Path;
use rmlib::core::version::install::extract_native_libraries;

extract_native_libraries(Path::new("/tmp/libs"));
```

This must run **after** `download_libraries`, because the native JARs must exist on disk first.

## `process_assets`

```rust
pub async fn process_assets(
    version_json: &serde_json::Value,
    assets_path: &Path,
    aassets: usize,
    soft: bool,
) -> Result<(), Vec<HttpError>>
```

Downloads the asset index and every asset object the version needs.

Steps:

1. Reads `assetIndex.id` and `assetIndex.url` from `version_json`.
2. Creates `assets_path/indexes/` and `assets_path/objects/`.
3. If the index file does not exist, downloads it from `assetIndex.url` (SHA-1-verified via the `assetIndex.sha1` field).
4. Parses the index, iterates `objects`, and downloads each object from `resources.download.minecraft.net/<first-2-chars-of-hash>/<hash>` into `objects/<first-2>/<hash>`.
5. For legacy versions (`assets == "legacy"` or `assets == "pre-1.6"`), copies each object into `assets/virtual/legacy/<resource_name>`.

Returns:

- `Ok(())` — everything downloaded (and legacy copies succeeded).
- `Err(Vec<HttpError>)` — one entry per failed object download (and, if the index cannot be downloaded, one entry for the index).

```rust
use std::path::Path;
use rmlib::core::version::install::process_assets;

process_assets(&version_json, Path::new("/tmp/assets"), 350, true).await.ok();
```

`soft = true` skips objects that already exist on disk. This is what makes `install` resume-friendly.

## Typical `install` flow at the API level

If you need more control than `install` gives you, call the helpers directly:

```rust
use std::path::PathBuf;
use rmlib::core::version::install::{
    download_client_jar, download_libraries, extract_native_libraries,
    prepare_version_files, process_assets, resolve_version_data,
};

let (id, json_text) = resolve_version_data(Some("1.20.1".into()), None).await;
let version_json: serde_json::Value = serde_json::from_str(&json_text).unwrap();

let versions = PathBuf::from("/tmp/versions");
let libs = PathBuf::from("/tmp/libs");
let assets = PathBuf::from("/tmp/assets");

let version_dir = prepare_version_files(&versions, &libs, &assets, &id, &json_text, "====");
download_client_jar(&version_json, &version_dir, &id, true).await;
let _ = download_libraries(&version_json, &libs, "====", 150, true).await;
extract_native_libraries(&libs);
let _ = process_assets(&version_json, &assets, 350, true).await;
```

## Error handling summary

| Function | On failure |
|---|---|
| `install` | Prints errors from libraries and assets; panics on client-jar / JSON errors. |
| `resolve_version_data` | Panics on network / JSON error; `process::exit(1)` on missing arguments. |
| `resolve_installation_paths` | Never fails (falls back to config values). |
| `prepare_version_files` | Panics if the JSON cannot be written. |
| `download_client_jar` | Panics on missing fields or after 3 failed retries. |
| `download_libraries` | Returns `Err(Vec<HttpError>)`. |
| `extract_native_libraries` | Silent on I/O errors (best effort). |
| `process_assets` | Returns `Err(Vec<HttpError>)`. |

## See also

- [Version](version.md) — `get_version_json` feeds `resolve_version_data`
- [Config](config.md) — default paths
- [core::run — Paths](run/paths.md) — the same directories used at launch time