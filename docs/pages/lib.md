# Core API

These items live at the crate root and are available regardless of which features are enabled.

```rust
use rmlib::{available_modules, ModuleInfo, CLIENT_ID, CLIENT_TOKEN, NAME, VERSION};
```

## Constants

### `VERSION: &str`

Crate version as recorded in `Cargo.toml` (`env!("CARGO_PKG_VERSION")`).

```rust
assert_eq!(rmlib::VERSION, "0.10.0");
```

### `NAME: &str`

Crate name, `env!("CARGO_PKG_NAME")` — always `"rmlib"`.

### `CLIENT_TOKEN: &str`

Microsoft/Xbox Live client token used by the launcher. Currently `"93301754-cd11-4a00-bd50-f69dffaf4b0b"`. Treat as a build-time constant; it is not a secret.

### `CLIENT_ID: &str`

Microsoft/Xbox Live client ID. Currently `"00000000402b5328"`.

## `ModuleInfo`

```rust
pub struct ModuleInfo {
    pub name: &'static str,
}
```

Describes one top-level module compiled into the current build. Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`.

- `name` — module name (`"core"`, `"ely"`, `"fabric"`, `"neoforge"`).

## `available_modules() -> Vec<ModuleInfo>`

Returns every top-level module compiled into the current build. One entry per enabled feature, in this order: `core`, `ely`, `fabric`, `neoforge`.

```rust
// With default features:
let mods = rmlib::available_modules();
assert!(mods.iter().any(|m| m.name == "core"));
assert!(mods.iter().any(|m| m.name == "ely"));
assert!(mods.iter().any(|m| m.name == "fabric"));
assert!(mods.iter().any(|m| m.name == "neoforge"));
```

With `--no-default-features --features core`, only `core` is present.

## Feature mapping

`available_modules()` reflects enabled features:

| Feature | Entry in `available_modules()` |
|---|---|
| `core` | `{ name: "core" }` |
| `ely` | `{ name: "ely" }` |
| `fabric` | `{ name: "fabric" }` |
| `neoforge` | `{ name: "neoforge" }` |

`ffi` is intentionally **not** listed — it is a build concern (shared library), not a launcher module.

## Module tree

| Path | Feature | What's there |
|---|---|---|
| `rmlib::core::config` | `core` | Configuration files and path getters |
| `rmlib::core::types` | `core` | `VersionType`, `build_maven_path` |
| `rmlib::core::version` | `core` | Manifest, version JSON, sorting, stripping |
| `rmlib::core::version::install` | `core` | Client JAR / libraries / assets / natives |
| `rmlib::core::run` | `core` | Launch preparation and execution |
| `rmlib::extra::ely` | `ely` | Ely.by auth, patch, skin proxy |
| `rmlib::extra::fabric` | `fabric` | Fabric listing and installation |
| `rmlib::extra::neoforge` | `neoforge` | NeoForge listing and installation |
| `rmlib::ffi` | `ffi` | C ABI exports |

## Module documentation

Detailed API reference:

- **[Config](core/config.md)** — `init`, `Paths`, `get_*_path`
- **[Types](core/types.md)** — `VersionType`, `build_maven_path`
- **[Version](core/version.md)** — `fetch_list`, `get_version_json`, `sort_by_type`, `strip_version`
- **[Version installation](core/version-install.md)** — `install`, `download_client_jar`, `download_libraries`, `extract_native_libraries`, `prepare_version_files`, `process_assets`, `resolve_installation_paths`, `resolve_version_data`
- **[Run](core/run/paths.md)** — path preparation, classpath, arguments, command, DTO
- **[Ely.by](extra/ely.md)** — `login`, `install_patch`, `start_beta_proxy`
- **[Fabric](extra/fabric.md)** — `fetch_list`, `install`
- **[NeoForge](extra/neoforge.md)** — `fetch_list`, `install`
- **[FFI](ffi/index.md)** — C ABI reference

## See also

- [Installation](getting-started/installation.md)
- [Features](getting-started/features.md)