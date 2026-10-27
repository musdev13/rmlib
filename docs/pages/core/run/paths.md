# core::run — Paths and Environment

Preparing the paths a launch will use, loading the version JSON for a launch, and detecting the current OS/architecture for rules evaluation.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

```rust
use rmlib::core::run::{prepare_paths, load_version_data, resolve_environment};
use rmlib::core::run::{LaunchPaths, LaunchEnvironment, VersionData};
```

## `LaunchPaths`

```rust
pub struct LaunchPaths {
    pub versions: PathBuf,
    pub assets: PathBuf,
    pub libs: PathBuf,
    pub native: PathBuf,
    pub game: PathBuf,
    pub version_json: PathBuf,
    pub client_jar: PathBuf,
}
```

The set of absolute paths a launch uses. Produced by [`prepare_paths`](#prepare_paths).

| Field | Meaning |
|---|---|
| `versions` | The versions directory (tilde-expanded). |
| `assets` | The assets directory (tilde-expanded). |
| `libs` | The libraries directory (tilde-expanded). |
| `native` | The `natives` directory inside `libs` (tilde-expanded). |
| `game` | The game directory (working directory for the game process). |
| `version_json` | `versions/<version_id>/<version_id>.json`. |
| `client_jar` | `versions/<version_id>/<version_id>.jar`. |

The `version_json` and `client_jar` paths are computed by joining — they are not checked for existence.

## `prepare_paths`

```rust
pub fn prepare_paths(
    version_id: &str,
    versions_path: &str,
    assets_path: &str,
    libs_path: &str,
    native_path: &str,
    game_path: &str,
) -> Result<LaunchPaths, String>
```

Builds a `LaunchPaths` from the six string inputs.

- Each of the six path inputs is passed through `musutils::fs::tilda_desir`, so `~`, relative paths and `..` are handled uniformly.
- `version_json` is `versions/<version_id>/<version_id>.json`.
- `client_jar` is `versions/<version_id>/<version_id>.jar`.
- Checks only that `version_json` exists. `client_jar` is **not** checked (a version can be prepared for launch without a client JAR if the game is expected to run entirely from libraries — rare, but the launcher does not enforce it here).

Returns:

- `Ok(LaunchPaths)` — the version JSON exists.
- `Err(String)` — `client json not found at <path>`. The string contains the resolved path.

```rust
use rmlib::core::run::prepare_paths;

let paths = prepare_paths(
    "1.20.1",
    "~/rml/versions",
    "~/rml/assets",
    "~/rml/libs",
    "~/rml/libs/natives",
    "~/.minecraft",
).unwrap();

println!("{:?}", paths.version_json);
```

**No filesystem side effects.** `prepare_paths` does not create anything; it only computes and checks the version JSON.

## `VersionData`

```rust
pub struct VersionData {
    pub version_json: serde_json::Value,
    pub modloader_json: Option<serde_json::Value>,
    pub main_class: String,
    pub asset_index_name: String,
    pub loader_libs_path: PathBuf,
}
```

The parsed version and mod loader JSONs, plus the fields a launch needs from them. Produced by [`load_version_data`](#load_version_data).

| Field | Meaning |
|---|---|
| `version_json` | The version's `versions/<id>/<id>.json` parsed as JSON. |
| `modloader_json` | The mod loader profile JSON (`fabric` / `neoforge`), if one was requested and found. |
| `main_class` | The Java main class to launch. Takes the mod loader profile's `mainClass` if present, otherwise the version's. |
| `asset_index_name` | The `assetIndex.id` field, or `"legacy"` if absent. |
| `loader_libs_path` | The libraries directory the mod loader's libraries live in. `libs` for vanilla, `libs/modloaders/fabric` for Fabric, `libs/modloaders/neoforge` for NeoForge. |

## `load_version_data`

```rust
pub fn load_version_data(
    version_id: &str,
    fabric_version: Option<&str>,
    neoforge_version: Option<&str>,
    paths: &LaunchPaths,
    logs: bool,
) -> Result<VersionData, String>
```

Reads the version JSON, optionally reads a mod loader profile JSON, and returns a `VersionData`.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_id` | Used only to build the mod loader profile path. |
| `fabric_version` | If `Some`, read `versions/modloaders/fabric/<version_id>/<fabric_version>.json` as the mod loader profile. |
| `neoforge_version` | If `Some`, read `versions/modloaders/neoforge/<version_id>/<neoforge_version>/<neoforge_version>.json`. |
| `paths` | Result of `prepare_paths`. `paths.version_json` is read; `paths.versions` and `paths.libs` are used to build mod loader paths. |
| `logs` | If `true`, print informational messages about mod loader profile loading. |

`fabric_version` and `neoforge_version` are independent — if both are `Some`, only the Fabric profile is loaded (the caller is expected to pass exactly one). The launcher's CLI enforces `conflicts_with` between them.

### Mod loader profile resolution

| Case | `modloader_json` | `main_class` source | `loader_libs_path` |
|---|---|---|---|
| No mod loader | `None` | `version_json.mainClass` | `paths.libs` |
| `fabric_version` given, profile exists | `Some(fabric profile)` | `fabric profile.mainClass` | `paths.libs/modloaders/fabric` |
| `fabric_version` given, profile missing | `None` (warned) | `version_json.mainClass` | `paths.libs/modloaders/fabric` |
| `neoforge_version` given, profile exists | `Some(neoforge profile)` | `neoforge profile.mainClass` | `paths.libs/modloaders/neoforge` |
| `neoforge_version` given, profile missing | `None` (warned) | `version_json.mainClass` | `paths.libs/modloaders/neoforge` |

Note that `loader_libs_path` **always** points at the mod loader directory when a version is requested, even if the profile is missing. That means `verify_libraries` (see [Classpath](classpath.md)) will look there for loader libraries; if the profile was missing, there is nothing to find.

### Returns

- `Ok(VersionData)` — everything parsed.
- `Err(String)` — `failed to read client json: ...` (the version JSON is missing or unreadable), or `failed to parse client json: ...`, or `missing mainClass inside json` (no `mainClass` field in either the profile or the version JSON).

```rust
use rmlib::core::run::{prepare_paths, load_version_data};

let paths = prepare_paths(
    "1.20.1",
    "~/rml/versions", "~/rml/assets", "~/rml/libs",
    "~/rml/libs/natives", "~/.minecraft",
).unwrap();

let data = load_version_data("1.20.1", Some("0.15.0"), None, &paths, false).unwrap();
println!("main class: {}", data.main_class);
```

## `LaunchEnvironment`

```rust
pub struct LaunchEnvironment {
    pub my_os: String,
    pub my_arch: String,
}
```

The current OS and architecture, normalized to the strings used in version JSON `rules`. Produced by [`resolve_environment`](#resolve_environment).

| Field | Possible values |
|---|---|
| `my_os` | `"windows"`, `"linux"`, `"osx"`, `"unknown"` |
| `my_arch` | `"64"`, `"32"`, `"arm64"`, `"arm32"`, `"unknown"` |

These are the strings that `check_rules` compares against `rules[i].os.name` and `rules[i].os.arch`, so they must match what appears in version JSON.

## `resolve_environment`

```rust
pub fn resolve_environment() -> LaunchEnvironment
```

Returns the current OS and architecture, using `musutils::os::get_os()` / `musutils::os::get_arch()`:

| `musutils` OS | `my_os` |
|---|---|
| `OS::Windows` | `"windows"` |
| `OS::Linux` | `"linux"` |
| `OS::MacOs` | `"osx"` |
| other | `"unknown"` |

| `musutils` Arch | `my_arch` |
|---|---|
| `Arch::X64` | `"64"` |
| `Arch::X86` | `"32"` |
| `Arch::Arm64` | `"arm64"` |
| `Arch::Arm` | `"arm32"` |
| other | `"unknown"` |

```rust
use rmlib::core::run::resolve_environment;

let env = resolve_environment();
assert!(matches!(env.my_os.as_str(), "windows" | "linux" | "osx" | "unknown"));
assert!(matches!(env.my_arch.as_str(), "64" | "32" | "arm64" | "arm32" | "unknown"));
```

## Typical usage

```rust
use rmlib::core::run::{prepare_paths, load_version_data, resolve_environment};

let paths = prepare_paths(
    version_id,
    &versions_path, &assets_path, &libs_path,
    &native_path, &game_path,
)?;
let data = load_version_data(version_id, fabric_version, neoforge_version, &paths, logs)?;
let env = resolve_environment();

// hand `paths`, `data`, `env` to build_classpath / build_launch_context / build_arguments
```

## See also

- [Classpath](classpath.md) — `verify_libraries`, `build_classpath`
- [Arguments](arguments.md) — `check_rules`, `parse_argument_list`, `build_arguments`
- [Config](../config.md) — where the default paths come from