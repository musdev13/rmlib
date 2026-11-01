# resman::mods

Discover, inspect and toggle locally installed mods.

Feature: `resman-mods` (implies `resman` → `core`).

```rust
use rmlib::extra::resman::mods::{
    self, ModInfo, ModDependency, ModLoader, ModError,
    Source, Priority, FallbackPolicy, Field, Format,
};
```

## Data model

```rust
pub struct ModInfo {
    pub path: PathBuf,
    pub filename: String,
    pub enabled: bool,
    pub size: u64,
    pub id: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub minecraft: Option<Vec<String>>,
    pub loader: Option<ModLoader>,
    pub icon: Option<PathBuf>,
    pub dependencies: Vec<ModDependency>,
}

pub struct ModDependency {
    pub id: String,
    pub kind: String,          // "required" | "optional" | ...
    pub version_range: Option<String>,
    pub fabric_api: bool,
}

pub enum ModLoader { Fabric, NeoForge, Universal, Unknown }
```

`filename` is the file name without the `.disabled` suffix; `path` is the real on-disk path.

`fabric_api: true` marks Fabric API modules (`fabric-*`); they are shipped by the single `fabric-api` mod, not installed separately.

`ModLoader::Universal` means the JAR carries **both** a valid `fabric.mod.json` and a NeoForge manifest — a multi-loader mod.

## Scanning

```rust
pub fn scan(game_path: &Path) -> Result<Vec<ModInfo>, ModError>
pub fn scan_dir(mods_dir: &Path) -> Result<Vec<ModInfo>, ModError>
```

`scan` reads `<game_path>/mods/`; `scan_dir` reads an explicit directory.

Both list `.jar` and `.jar.disabled` files, include disabled mods (`enabled: false`), sort by file name, and fall back to a minimal `ModInfo` (path/filename/enabled/size only) when a JAR cannot be parsed.

```rust
let mods = mods::scan(std::path::Path::new("./game"))?;
for m in &mods {
    println!("{} {:?}", m.filename, m.id);
}
```

## Single-file info

```rust
pub fn info_from_file(path: &Path) -> Result<ModInfo, ModError>
pub fn info_by_id(mods_dir: &Path, id: &str) -> Result<Option<ModInfo>, ModError>
```

`info_from_file` returns `ModError::NotFound` for a missing file. `info_by_id` scans a directory and matches `ModInfo::id`.

## Enabling and disabling

```rust
pub fn is_enabled(path: &Path) -> bool
pub fn enable(path: &Path) -> Result<PathBuf, ModError>
pub fn disable(path: &Path) -> Result<PathBuf, ModError>
pub fn toggle(path: &Path) -> Result<(PathBuf, bool), ModError>
```

Disabling appends `.disabled` to the file name; enabling removes it. Both are no-ops when the file is already in the requested state and return the resulting path. `toggle` returns the new path and the new enabled state.

```rust
let (path, enabled) = mods::toggle(std::path::Path::new("./game/mods/sodium.jar"))?;
println!("{} -> {}", path.display(), enabled);
```

## Sources, priority and fallback

```rust
pub enum Source { Local, Modrinth }

pub struct Priority;                   // ordered list of sources
impl Priority {
    pub fn parse(s: &str) -> Result<Self, ModError>;
    pub fn sources(&self) -> &[Source];
    pub fn includes(&self, s: Source) -> bool;
}

pub enum FallbackPolicy {
    Placeholder,
    Skip,
    Fields(Vec<Field>),
}
```

`Priority::parse` accepts a comma-separated list, e.g. `"local"`, `"modrinth"`, `"modrinth,local"`. The default is `[Local]`. Using `"modrinth"` requires the `modrinth` feature; otherwise parsing fails with `ModError::InvalidArgument`.

```rust
let p = Priority::parse("modrinth,local")?;
```

Async resolution:

```rust
pub async fn info_priority(
    path: &Path,
    priority: &Priority,
    fallback: &FallbackPolicy,
) -> Result<Option<ModInfo>, ModError>

pub async fn scan_priority(
    game_path: &Path,
    priority: &Priority,
    fallback: &FallbackPolicy,
) -> Result<Vec<ModInfo>, ModError>

pub async fn scan_priority_dir(
    dir: &Path,
    priority: &Priority,
    fallback: &FallbackPolicy,
) -> Result<Vec<ModInfo>, ModError>
```

`scan_priority` reads `<game_path>/mods`; `scan_priority_dir` reads the given directory directly (no `mods` suffix added).

Resolution walks `Priority` in order:

1. `Local` — parse the file.
2. `Modrinth` — look the file up by SHA-1 (only with the `modrinth` feature).

The first source that resolves the file wins.

With **`modrinth,local`**:

- `version`, `loader` and `minecraft` come from the local JAR if it has them (the JAR describes the exact file);
- Modrinth supplies `id`, `name`, `description`, `license`;
- any remaining empty field is filled from the local JAR.

With **`modrinth` alone**, nothing is filled from the local file, and if Modrinth has no match the local file is not consulted.

Modrinth resolution works on the **file**, not the project: the JAR's SHA-1 is looked up with `/version_file/{hash}` to get `version_number`, `loaders` and `game_versions`, then the project is fetched for `id`, `name`, `description` and `license`.

When no source resolves a file, `FallbackPolicy` applies:

| Policy | Behaviour |
|---|---|
| `Placeholder` (default) | Return a `ModInfo` with placeholders: `filename` = file stem, all info fields `None`, `path`/`enabled`/`size` filled. |
| `Skip` | Drop the file from the result (`Ok(None)` / excluded from the list). |
| `Fields(list)` | Build a placeholder, then fill **only** the listed fields from the local file. |

`Fields` takes precedence over `Placeholder`; `Skip` is mutually exclusive with `Fields` at the CLI level.

## Field selection and formats

```rust
pub enum Field {
    Id, Name, Version, Description, Authors, License, Minecraft,
    Dependencies, Icon, Path, Enabled, Loader, Filename, Size, FabricApi,
}

pub fn parse_fields(s: &str) -> Result<Vec<Field>, ModError>

pub enum Format { Json, Table, Plain }
impl Format { pub fn parse(s: &str) -> Result<Self, ModError>; }
```

`parse_fields` accepts a comma-separated list (`"id,name,version"`).

Rendering helpers:

```rust
pub fn info_file_as(path: &Path, fields: Option<&[Field]>, format: Format) -> Result<String, ModError>
pub fn scan_as(dir: &Path, fields: Option<&[Field]>, format: Format) -> Result<String, ModError>
pub fn info_file_fields(path: &Path, fields: &[Field]) -> Result<Map<String, Value>, ModError>
pub fn scan_fields(dir: &Path, fields: &[Field]) -> Result<Vec<Map<String, Value>>, ModError>
pub fn render_value(value: &Value, format: Format) -> String
```

`None` fields means "all fields". `Plain` renders `key: value` lines, `Table` renders an aligned table, `Json` renders pretty JSON.

In `Table` / `Plain` output, `dependencies` is flattened to `id (kind), id (kind)` (for example `fabric-api (required), sodium (required)`), and embedded newlines in string fields are replaced with spaces; `Json` keeps the raw values.

```rust
let out = mods::scan_as(
    std::path::Path::new("./game/mods"),
    Some(&[Field::Id, Field::Name, Field::Version]),
    Format::Table,
)?;
println!("{out}");
```

## Icons

```rust
pub fn icons_dir() -> PathBuf
pub fn save_icon(key: &str, bytes: &[u8]) -> Result<PathBuf, ModError>
```

Icons are written to `std::env::temp_dir()/rmlib_icons/<key>.png`. `key` is sanitized (anything other than alphanumerics, `-`, `_` becomes `_`).

## Parsing rules

- **NeoForge first:** `META-INF/neoforge.mods.toml`, then `META-INF/mods.toml`. A mod may ship a decoy `fabric.mod.json` (JourneyMap-style, `id` ending in `-wrongloader`, `version` `"not a fabric mod"`, or a `preLaunch` entrypoint class ending in `Error`); the NeoForge manifest wins.
- **Fabric second:** `fabric.mod.json`. Decoys are rejected.
- A JAR with **both** a valid `fabric.mod.json` **and** a NeoForge manifest is reported as `ModLoader::Universal` (metadata comes from the NeoForge manifest). A decoy `fabric.mod.json` does not trigger this — such JARs stay `NeoForge`.
- Dependencies come from `depends` / `recommends` (Fabric) or `[[dependencies.<modid>]]` (NeoForge). Built-in ids (`minecraft`, `fabricloader`, `neoforge`, `forge`, …) are skipped. `minecraft` becomes `ModInfo::minecraft`.
- If the manifest declares no Minecraft version, it is inferred from `+mcX.Y` in the version string or from a leading `X.Y` in the file name.

## Errors

```rust
pub enum ModError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Json(serde_json::Error),
    Toml(toml::de::Error),
    Modrinth(String),
    InvalidArgument(String),
    NotFound(String),
}
```

## See also

- [resman](resman.md)
- [resman::resourcepacks](resman-resourcepacks.md)
- [FFI — resman::mods](../ffi/resman-mods.md)