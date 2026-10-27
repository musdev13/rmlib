# extra::fabric

Fabric Loader version listing and installation.

Fabric is a lightweight mod loader. Installing a Fabric-enabled launch has two parts: install the vanilla version the mod loader will run on, and install the Fabric profile and its libraries on top. This module does both.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core", "fabric"] }
```

```rust
use rmlib::extra::fabric::{fetch_list, install};
```

## `fetch_list`

```rust
pub async fn fetch_list(version_id: &str) -> Vec<String>
```

Lists available Fabric Loader versions for a given Minecraft version.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_id` | Minecraft version (e.g. `"1.20.1"`). |

### Behaviour

1. Requests `https://meta.fabricmc.net/v2/versions/loader/<version_id>`.
2. Parses the JSON array response.
3. Extracts `loader.version` from each entry and collects the strings.

### Returns

A `Vec<String>` of Fabric Loader version strings, in the order Fabric returns them (typically newest first). An empty vector means Fabric has no loader releases for that Minecraft version.

### Panics

- Manifest fetch fails → `Status::Err: Failed to fetch Fabric manifest string`.
- Invalid JSON → `Status::Err: Failed to parse Fabric manifest JSON`.
- Response is not an array → `Status::Err: Expected JSON array from Fabric API`.

### Example

```rust
use rmlib::extra::fabric::fetch_list;

let loaders = fetch_list("1.20.1").await;
if let Some(latest) = loaders.first() {
    println!("latest Fabric Loader for 1.20.1: {latest}");
}
```

The first entry is a reasonable default for the `fabric_version` parameter of `install` and of `core::run::run_client`.

## `install`

```rust
pub async fn install(
    version_id: String,
    fabric_version: String,
    versions_path_option: Option<PathBuf>,
    libs_path_option: Option<PathBuf>,
    assets_path_option: Option<PathBuf>,
)
```

Installs a Fabric-enabled version.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_id` | The vanilla Minecraft version to base on (e.g. `"1.20.1"`). |
| `fabric_version` | The Fabric Loader version (e.g. `"0.15.0"`). |
| `versions_path_option` | Override the versions directory. `None` uses `core::config::get_versions_path()`. |
| `libs_path_option` | Override the libraries directory. `None` uses `core::config::get_libs_path()`. |
| `assets_path_option` | Override the assets directory. `None` uses `core::config::get_assets_path()`. |

### Behaviour

1. **Soft-installs the vanilla base version.** Calls `core::version::install::install` with `soft = true, iclient = true, ilibs = true, iassets = true, alibs = 150, aassets = 350`. This ensures the vanilla client JAR, vanilla libraries, assets and natives are on disk.
2. Resolves the three directories (tilde-expanded).
3. Creates `versions/modloaders/fabric/<version_id>/`.
4. Creates `libs/modloaders/fabric/`.
5. Fetches `https://meta.fabricmc.net/v2/versions/loader/<version_id>/<fabric_version>/profile/json` — the Fabric profile for that specific loader version.
6. Writes the profile JSON to `versions/modloaders/fabric/<version_id>/<fabric_version>.json`.
7. Iterates the profile's `libraries`, resolves each Maven coordinate with `build_maven_path`, and downloads each library to `libs/modloaders/fabric/<maven-path>`, with SHA-1 verification where the profile provides it.
8. Skips libraries already on disk (that is the `soft = true` behaviour inherited from the downloader).

### Panics

- Profile JSON fetch fails → `Status::Err: failed to fetch Fabric profile JSON`.
- Profile JSON write fails → `Status::Err: can't write <fabric_version>.json`.
- Profile JSON cannot be parsed → `Status::Err: failed to parse Fabric profile JSON`.

The vanilla base install (step 1) has its own panics, inherited from `core::version::install::install` — see [Version installation](../core/version-install.md#install).

### Library download errors

Fabric library downloads are **not** fatal. On failure, `install` prints a `Status::Err: Fabric libraries installation finished with N error(s); rerun install to retry failed downloads` line and returns normally. Re-running `install` with the same parameters resumes the download (already-present libraries are skipped).

### Example

```rust
use rmlib::extra::fabric::install;

// Use defaults from core::config.
install("1.20.1".into(), "0.15.0".into(), None, None, None).await;

// Or override paths explicitly.
use std::path::PathBuf;
install(
    "1.20.1".into(),
    "0.15.0".into(),
    Some(PathBuf::from("/tmp/versions")),
    Some(PathBuf::from("/tmp/libs")),
    Some(PathBuf::from("/tmp/assets")),
).await;
```

## What ends up on disk

After a successful Fabric install for `1.20.1` / `0.15.0`:

```text
versions/
├── 1.20.1/                      # vanilla base, written by core::install
│   ├── 1.20.1.json
│   └── 1.20.1.jar
└── modloaders/fabric/1.20.1/
    └── 0.15.0.json              # Fabric profile, written by this module

libs/
├── (vanilla libraries)          # written by core::install
└── modloaders/fabric/
    └── (Fabric libraries)       # written by this module
```

To launch with Fabric, pass `Some("0.15.0")` as the `fabric_version` argument of `core::run::run_client` (or `build_argv` / `build_command`). `load_version_data` will then find the profile at `versions/modloaders/fabric/1.20.1/0.15.0.json` and use its `mainClass` and libraries.

## End-to-end flow

```rust
use rmlib::core::run::run_client;
use rmlib::extra::fabric::{fetch_list, install};

// 1. Pick a loader version.
let loaders = fetch_list("1.20.1").await;
let loader = loaders.first().cloned().expect("no Fabric loader for 1.20.1");

// 2. Install.
install("1.20.1".into(), loader.clone(), None, None, None).await;

// 3. Launch.
run_client(
    "1.20.1",
    Some(&loader),
    None,
    "4G",
    "Player",
    "00000000-0000-0000-0000-000000000000",
    "xuid",
    "token",
    false, false,
    "~/rml/versions", "~/rml/assets", "~/rml/libs",
    "~/rml/libs/natives", "~/.minecraft", "~/.minecraft",
    &[], &[],
).await;
```

## Interaction with other features

- **`fabric` implies `core`.** `install` uses `core::version::install::install` to prepare the vanilla base.
- **`ely` is orthogonal.** The `ely` / `betafix` flags in `core::run` work with Fabric the same way they work with vanilla; nothing here depends on them.
- **`neoforge` conflicts.** At the CLI level, `--fabric` and `--neoforge` are `conflicts_with` each other. At the API level, `load_version_data` only loads the Fabric profile if `fabric_version` is `Some`, and only loads the NeoForge profile if `neoforge_version` is `Some`; passing both is a caller error (only the Fabric one is used).

## Notes

- **Fabric's API is version-pinned.** The `<version_id>/<fabric_version>/profile/json` URL only returns a profile for valid combinations. `fetch_list` is the reliable way to enumerate the valid `fabric_version` values.
- **Soft install is always on.** `install` does not have a `soft` parameter; it is hard-coded to `soft = true` for both the vanilla base and the Fabric libraries. Re-running it is safe and cheap (already-present files are skipped).
- **No uninstall.** There is no `uninstall` function — deleting `versions/modloaders/fabric/<version_id>/` and `libs/modloaders/fabric/` is what "uninstall" would do.

## See also

- [NeoForge](neoforge.md) — same shape, different source
- [core::run — Paths](../core/run/paths.md) — how the Fabric profile is picked up at launch time
- [core::version — Installation](../core/version-install.md) — the vanilla base install
- [Features](../getting-started/features.md) — `fabric` feature