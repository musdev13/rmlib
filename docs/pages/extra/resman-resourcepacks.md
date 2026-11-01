# resman::resourcepacks

Discover and inspect locally installed resource packs.

Feature: `resman-resourcepacks` (implies `resman` → `core`).

This module has no enable/disable toggle — resource packs are managed differently from mods. It has its own `Source`, `Priority`, `FallbackPolicy`, `Field` and error types, independent of `resman::mods`.

```rust
use rmlib::extra::resman::resourcepacks::{
    self, ResourcepackInfo, RpError,
    Source, Priority, FallbackPolicy, Field, Format,
};
```

## Data model

```rust
pub struct ResourcepackInfo {
    pub path: PathBuf,
    pub filename: String,
    pub enabled: bool,
    pub size: u64,
    pub id: Option<String>,
    pub description: Option<String>,
    pub pack_format: Option<u32>,
    pub supported_formats: Option<serde_json::Value>,
    pub icon: Option<PathBuf>,
}
```

`filename` is the file name without the `.disabled` suffix; `path` is the real on-disk path. `id` is the Modrinth project slug and is filled only when the pack was resolved through Modrinth; local parsing leaves it `None`.

There is deliberately **no** Minecraft version field: the `pack_format` number is what a resource pack declares, and mapping it to a game version would need a hardcoded table that goes stale after every Minecraft release.

## Scanning

```rust
pub fn scan(game_path: &Path) -> Result<Vec<ResourcepackInfo>, RpError>
pub fn scan_dir(dir: &Path) -> Result<Vec<ResourcepackInfo>, RpError>
```

`scan` reads `<game_path>/resourcepacks/`; `scan_dir` reads an explicit directory.

Both list `.zip` and `.zip.disabled` files, include disabled packs (`enabled: false`), sort by file name, and fall back to a minimal `ResourcepackInfo` (path/filename/enabled/size only) when a ZIP has no usable `pack.mcmeta`.

```rust
let packs = resourcepacks::scan(std::path::Path::new("./game"))?;
for p in &packs {
    println!("{} {:?}", p.filename, p.pack_format);
}
```

## Single-file info

```rust
pub fn info_from_file(path: &Path) -> Result<ResourcepackInfo, RpError>
```

Returns `RpError::NotFound` for a missing file and `RpError::InvalidArgument` when the ZIP has no `pack.mcmeta`.

## pack.mcmeta parsing

`pack.mcmeta` comes in two shapes:

```json
{ "pack": { "pack_format": 34, "description": "..." } }
```

```json
{ "pack": { "min_format": 84, "max_format": 97, "description": "..." } }
```

Both are supported. `ResourcepackInfo::pack_format` is taken from `pack_format`, falling back to `min_format`, then `max_format`. When `supported_formats` is absent but `min_format` / `max_format` are present, a `{"min_inclusive": ..., "max_inclusive": ...}` object is synthesised, so table output still shows a range.

## Sources, priority and fallback

```rust
pub enum Source { Local, Modrinth }

pub struct Priority;
impl Priority {
    pub fn parse(s: &str) -> Result<Self, RpError>;
    pub fn sources(&self) -> &[Source];
    pub fn includes(&self, s: Source) -> bool;
}

pub enum FallbackPolicy {
    Placeholder,
    Skip,
    Fields(Vec<Field>),
}
```

`Priority::parse` accepts a comma-separated list, e.g. `"local"`, `"modrinth"`, `"modrinth,local"`. The default is `[Local]`. Using `"modrinth"` requires the `modrinth` feature; otherwise parsing fails with `RpError::InvalidArgument`.

Async resolution:

```rust
pub async fn info_priority(
    path: &Path,
    priority: &Priority,
    fallback: &FallbackPolicy,
) -> Result<Option<ResourcepackInfo>, RpError>

pub async fn scan_priority(
    game_path: &Path,
    priority: &Priority,
    fallback: &FallbackPolicy,
) -> Result<Vec<ResourcepackInfo>, RpError>

pub async fn scan_priority_dir(
    dir: &Path,
    priority: &Priority,
    fallback: &FallbackPolicy,
) -> Result<Vec<ResourcepackInfo>, RpError>
```

`scan_priority` reads `<game_path>/resourcepacks`; `scan_priority_dir` reads the given directory directly (no `resourcepacks` suffix added).

Resolution walks `Priority` in order:

1. `Local` — parse `pack.mcmeta`.
2. `Modrinth` — look the file up by SHA-1 (only with the `modrinth` feature).

The first source that resolves the file wins.

With **`modrinth,local`**, fields present in the local ZIP (`description`, `pack_format`, `supported_formats`, `icon`) take precedence over Modrinth's, and Modrinth fills whatever the ZIP lacks.

With **`modrinth` alone**, nothing is filled from the local file, and if Modrinth has no match the local file is not consulted.

When no source resolves a file, `FallbackPolicy` applies:

| Policy | Behaviour |
|---|---|
| `Placeholder` (default) | Return a `ResourcepackInfo` with placeholders: `filename` = file stem, info fields `None`, `path`/`enabled`/`size` filled. |
| `Skip` | Drop the file from the result. |
| `Fields(list)` | Build a placeholder, then fill **only** the listed fields from the local file. |

## Field selection and formats

```rust
pub enum Field {
    Id, Description, PackFormat, SupportedFormats,
    Icon, Path, Enabled, Filename, Size,
}

pub fn parse_fields(s: &str) -> Result<Vec<Field>, RpError>

pub enum Format { Json, Table, Plain }
```

Rendering helpers:

```rust
pub fn info_file_as(path: &Path, fields: Option<&[Field]>, format: Format) -> Result<String, RpError>
pub fn scan_as(dir: &Path, fields: Option<&[Field]>, format: Format) -> Result<String, RpError>
pub fn info_file_fields(path: &Path, fields: &[Field]) -> Result<Map<String, Value>, RpError>
pub fn scan_fields(dir: &Path, fields: &[Field]) -> Result<Vec<Map<String, Value>>, RpError>
pub fn render_value(value: &Value, format: Format) -> String
```

`None` fields means "all fields". `Plain` renders `key: value` lines, `Table` renders an aligned table, `Json` renders pretty JSON.

In `Table` / `Plain` output, `supported_formats` is flattened to a short string (`84..97`, `15, 255`, a single number) and embedded newlines in `description` are replaced with spaces; `Json` keeps the raw values.

## Icons

```rust
pub fn icons_dir() -> PathBuf
pub fn save_icon(key: &str, bytes: &[u8]) -> Result<PathBuf, RpError>
```

Icons (`pack.png`) are written to `std::env::temp_dir()/rmlib_icons/<key>.png`.

## Errors

```rust
pub enum RpError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Json(serde_json::Error),
    Modrinth(String),
    InvalidArgument(String),
    NotFound(String),
}
```

## See also

- [resman](resman.md)
- [resman::mods](resman-mods.md)
- [FFI — resman::resourcepacks](../ffi/resman-resourcepacks.md)