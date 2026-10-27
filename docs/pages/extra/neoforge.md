# extra::neoforge

NeoForge version listing and installation.

NeoForge is a fork of Forge. Unlike Fabric, NeoForge does not publish a ready-made profile JSON: installation requires downloading the NeoForge **installer** JAR and running it with `java --install-client`. This module does the whole process.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core", "neoforge"] }
```

```rust
use rmlib::extra::neoforge::{fetch_list, install};
```

## `fetch_list`

```rust
pub async fn fetch_list(version_id: &str) -> Vec<String>
```

Lists available NeoForge versions for a given Minecraft version.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_id` | Minecraft version (e.g. `"1.20.1"`). |

### Behaviour

1. Derives a version prefix from `version_id`:
   - `"1.20.1"` → `"20.1."`
   - `"1.21"` → `"21."`
   - `"20.1"` → `"20.1."` (already looks like a NeoForge prefix)
   - Anything with non-numeric characters after `1.` or not matching the pattern → returns an empty vector without making a network call.
2. Downloads `https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml`.
3. Parses the XML and collects every `<version>` entry whose text starts with the prefix.
4. Reverses the result (newest first).

### Returns

A `Vec<String>` of NeoForge versions matching the Minecraft version, newest first. An empty vector means:

- `version_id` does not match the expected NeoForge pattern, **or**
- NeoForge has no releases for that Minecraft version.

### Panics

- Fetch fails → `Status::Err: Failed to fetch NeoForge maven-metadata string`.
- Response is not UTF-8 → `Status::Err: Failed to parse NeoForge response as UTF-8`.
- XML parse error → `Status::Err: Failed to parse NeoForge XML`.

### Example

```rust
use rmlib::extra::neoforge::fetch_list;

let versions = fetch_list("1.20.1").await;
if let Some(latest) = versions.first() {
    println!("latest NeoForge for 1.20.1: {latest}");
}
```

The first entry is a reasonable default for the `neoforge_version` parameter of `install` and of `core::run::run_client`.

### Prefix mapping

NeoForge uses a naming scheme where `1.<minor>.<patch>` maps to `<minor>.<patch>.<build>`. Some examples:

| Minecraft | NeoForge prefix | Example NeoForge version |
|---|---|---|
| `1.20.1` | `20.1.` | `20.1.100` |
| `1.20.4` | `20.4.` | `20.4.240` |
| `1.21` | `21.` | `21.0.100` |
| `1.21.1` | `21.1.` | `21.1.100` |

`fetch_list` does the mapping automatically; you pass the Minecraft version and it filters NeoForge versions to the matching prefix.

## `install`

```rust
pub async fn install(
    version_id: String,
    neoforge_version: String,
    versions_path_option: Option<PathBuf>,
    libs_path_option: Option<PathBuf>,
    assets_path_option: Option<PathBuf>,
)
```

Installs a NeoForge-enabled version.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_id` | The vanilla Minecraft version to base on (e.g. `"1.20.1"`). |
| `neoforge_version` | The NeoForge version (e.g. `"20.4.100"`). |
| `versions_path_option` | Override the versions directory. `None` uses `core::config::get_versions_path()`. |
| `libs_path_option` | Override the libraries directory. `None` uses `core::config::get_libs_path()`. |
| `assets_path_option` | Override the assets directory. `None` uses `core::config::get_assets_path()`. |

### Behaviour

1. **Soft-installs the vanilla base version.** Calls `core::version::install::install` with `soft = true, iclient = true, ilibs = true, iassets = true, alibs = 150, aassets = 350`.
2. Resolves the three directories (tilde-expanded).
3. Creates `versions/modloaders/neoforge/temp/`.
4. Downloads `https://maven.neoforged.net/releases/net/neoforged/neoforge/<neoforge_version>/neoforge-<neoforge_version>-installer.jar` into `temp/`.
5. Creates a `temp/fakemc/` directory and writes `temp/fakemc/launcher_profiles.json` with `{}` — the NeoForge installer requires a launcher profile file to exist.
6. Runs `java -jar neoforge-<version>-installer.jar --install-client ./fakemc` from inside `temp/`.
7. Moves `temp/fakemc/libraries/` into `libs/modloaders/neoforge/`.
8. Moves `temp/fakemc/versions/neoforge-<version>/neoforge-<version>.json` into `versions/modloaders/neoforge/<version_id>/<neoforge_version>/<neoforge_version>.json`.
9. Removes `temp/`.

On failure at steps 4–8, `install` cleans up `temp/` and returns early (no `Result` — failures are printed to stdout).

### Panics

The vanilla base install (step 1) has the same panics as `core::version::install::install` — see [Version installation](../core/version-install.md#install).

Steps 4–8 do not panic on network or I/O failure — they print `Status::Err` messages and clean up.

### Requirements

- **`java` must be on `PATH`.** The installer JAR is executed as a subprocess.
- **Network access** to `maven.neoforged.net`.
- **Disk space** for the installer and the NeoForge libraries it extracts (a few hundred MB).

### Example

```rust
use rmlib::extra::neoforge::install;

install("1.20.1".into(), "20.4.100".into(), None, None, None).await;
```

With explicit paths:

```rust
use std::path::PathBuf;
use rmlib::extra::neoforge::install;

install(
    "1.20.1".into(),
    "20.4.100".into(),
    Some(PathBuf::from("/tmp/versions")),
    Some(PathBuf::from("/tmp/libs")),
    Some(PathBuf::from("/tmp/assets")),
).await;
```

## What ends up on disk

After a successful NeoForge install for `1.20.1` / `20.4.100`:

```text
versions/
├── 1.20.1/                          # vanilla base, written by core::install
│   ├── 1.20.1.json
│   └── 1.20.1.jar
└── modloaders/neoforge/1.20.1/
    └── 20.4.100/
        └── 20.4.100.json            # NeoForge profile, produced by the installer

libs/
├── (vanilla libraries)              # written by core::install
└── modloaders/neoforge/
    └── (NeoForge libraries)         # moved from installer output
```

The `temp/` directory is removed at the end, whether the install succeeded or failed.

To launch with NeoForge, pass `Some("20.4.100")` as the `neoforge_version` argument of `core::run::run_client` (or `build_argv` / `build_command`). `load_version_data` will then find the profile at `versions/modloaders/neoforge/1.20.1/20.4.100/20.4.100.json`.

## End-to-end flow

```rust
use rmlib::core::run::run_client;
use rmlib::extra::neoforge::{fetch_list, install};

// 1. Pick a NeoForge version.
let versions = fetch_list("1.20.1").await;
let neo = versions.first().cloned().expect("no NeoForge for 1.20.1");

// 2. Install.
install("1.20.1".into(), neo.clone(), None, None, None).await;

// 3. Launch.
run_client(
    "1.20.1",
    None,
    Some(&neo),
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

## Differences from Fabric

| Aspect | Fabric | NeoForge |
|---|---|---|
| Profile source | HTTP JSON from `meta.fabricmc.net` | Installer JAR (run as a subprocess) |
| Version listing | JSON array | XML `<version>` entries |
| Requires `java` at install time | No | **Yes** — the installer is a JAR |
| Writes profile directly | Yes | No — installer writes it, this module moves it |
| Vanilla base install | Yes (shared with NeoForge) | Yes (shared with Fabric) |

The shared "install the vanilla base first" step is identical — see `core::version::install::install`.

## Interaction with other features

- **`neoforge` implies `core`.**
- **`fabric` conflicts** at the CLI level (`--fabric` and `--neoforge` are `conflicts_with`). At the API level, only one of `fabric_version` / `neoforge_version` should be `Some` when calling `load_version_data`.
- **`ely` is orthogonal.** `ely` / `betafix` flags work with NeoForge the same way they work with vanilla.

## Notes

- **The NeoForge installer expects an empty-ish directory.** `install` prepares `fakemc/launcher_profiles.json` to satisfy it. Any leftover files in `temp/fakemc/` are moved (libraries) or deleted.
- **`--install-client` is the only installer mode used.** The installer's CLI has other options (server install, `--installServer`), which this crate does not use.
- **No incremental install.** Unlike the library and asset downloads, NeoForge install does not check what is already present — it re-runs the installer each time. Running `install` twice is safe (the installer overwrites), just slower than a Fabric re-install.

## See also

- [Fabric](fabric.md) — same shape, different source
- [core::run — Paths](../core/run/paths.md) — how the NeoForge profile is picked up at launch time
- [core::version — Installation](../core/version-install.md) — the vanilla base install
- [Features](../getting-started/features.md) — `neoforge` feature