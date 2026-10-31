# extra::modrinth

Client for the [Modrinth API v2](https://docs.modrinth.com/api/): project search, project/version metadata, user and team info, tag directories, and content installation.

This is an optional area behind the `modrinth` feature (implies `core`). It does not depend on `ely`/`fabric`/`neoforge`.

!!! note "Not supported"
    - **Server plugins** are out of scope for this module.
    - Modrinth has **no server entity** in its public API — there is no server search here.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["modrinth"] }
```

```rust
use rmlib::extra::modrinth::{
    search, SearchOptions, ProjectType, SearchIndex,
    project, version, user, tag, install, hash,
};
```

## Module layout

| Module | Purpose |
|---|---|
| `client` | HTTP client, `GET` with query, API error parsing. |
| `dto` | Response/request data types (`Project`, `Version`, `SearchResponse`, …). |
| `error` | `ModrinthError`. |
| `facets` | `FacetBuilder`, query building, pagination helpers. |
| `hash` | SHA-1 / SHA-512 computation and verification. |
| `search` | Project search. |
| `project` | Project metadata, dependencies, members. |
| `version` | Version listing, lookup by ID / hash, compatibility selection. |
| `user` | Users and team members. |
| `tag` | Categories, loaders, game versions, project types. |
| `install` | Download and install content. |

## Search

```rust
pub async fn search(options: &SearchOptions) -> Result<SearchResponse, ModrinthError>
```

`SearchOptions`:

| Field | Meaning |
|---|---|
| `query` | Free-text search string. |
| `project_type` | `ProjectType::{Mod, Modpack, Resourcepack, Shader, Datapack, World}`. |
| `game_versions` | Minecraft versions to filter by. |
| `loaders` | Mod loaders (`fabric`, `forge`, `neoforge`, …). |
| `categories` | Category names. |
| `client_side` / `server_side` | `SideSupport::{Required, Optional, Unsupported, Unknown}`. |
| `index` | `SearchIndex::{Relevance, Downloads, Follows, Newest, Updated}`. |
| `page` | 1-based page number (converted to `offset`). |
| `limit` | Page size, clamped to `1..=100`. |
| `offset` | Explicit offset; wins over `page`. |

Filters are assembled by `FacetBuilder` into the Modrinth `facets` format; values of the same facet key are OR-ed, different keys are AND-ed.

```rust
use rmlib::extra::modrinth::{search, SearchOptions, ProjectType, SearchIndex};

let opts = SearchOptions {
    query: Some("sodium".to_string()),
    project_type: Some(ProjectType::Mod),
    game_versions: vec!["1.21.1".to_string()],
    loaders: vec!["fabric".to_string()],
    index: Some(SearchIndex::Downloads),
    page: Some(1),
    limit: Some(20),
    ..Default::default()
};
let response = search(&opts).await?;
for hit in &response.hits {
    println!("{} — {} downloads", hit.title, hit.downloads);
}
println!("total: {}", response.total_hits);
```

`SearchResponse` carries `hits`, `offset`, `limit`, `total_hits`.

## Project metadata

```rust
pub async fn get_project(id_or_slug: &str) -> Result<Project, ModrinthError>
pub async fn get_projects(ids: &[String]) -> Result<Vec<Project>, ModrinthError>
pub async fn get_project_by_hash(hash: &str) -> Result<Project, ModrinthError>
pub async fn get_project_dependencies(project_id: &str) -> Result<Vec<Dependency>, ModrinthError>
pub async fn get_project_members(project_id: &str) -> Result<Vec<ProjectMember>, ModrinthError>
```

`get_project_by_hash` resolves the hash to a version, then to the project.

`Project` keeps the API fields: slug, title, description, body, categories, client/server side, downloads, followers, links (`source_url`, `issues_url`, `wiki_url`, `discord_url`, `donation_urls`), license, versions, game versions, loaders, gallery, team, status, moderator message.

`Dependency` distinguishes types via `is_required()`, `is_optional()`, `is_incompatible()`, `is_embedded()`.

## Versions

```rust
pub async fn get_version(version_id: &str) -> Result<Version, ModrinthError>
pub async fn get_version_by_hash(hash: &str) -> Result<Version, ModrinthError>
pub async fn get_project_versions(project_id: &str, filters: &VersionFilters) -> Result<Vec<Version>, ModrinthError>

pub fn filter_versions(versions: &[Version], filters: &VersionFilters) -> Vec<Version>
pub fn select_compatible_version<'a>(
    versions: &'a [Version],
    game_version: Option<&str>,
    loader: Option<&str>,
    version_type: Option<VersionType>,
) -> Option<&'a Version>
```

`VersionFilters`:

| Field | Meaning |
|---|---|
| `game_versions` | Filter by Minecraft versions. |
| `loaders` | Filter by loaders. |
| `version_type` | `VersionType::{Release, Beta, Alpha}`. |

`Version` exposes `primary_file()`, `is_release()`, `is_beta()`, `is_alpha()`, `supports_game_version()`, `supports_loader()`. The version type is separate from compatibility (game versions / loaders).

```rust
use rmlib::extra::modrinth::{version, VersionFilters, VersionType};

let filters = VersionFilters {
    game_versions: Some(vec!["1.21.1".to_string()]),
    loaders: Some(vec!["fabric".to_string()]),
    version_type: Some(VersionType::Release),
};
let versions = version::get_project_versions("sodium", &filters).await?;
if let Some(v) = version::select_compatible_version(&versions, Some("1.21.1"), Some("fabric"), Some(VersionType::Release)) {
    println!("use version {}", v.version_number);
}
```

## Users, teams, tags

```rust
pub async fn get_user(id_or_username: &str) -> Result<User, ModrinthError>
pub async fn get_user_projects(user_id: &str) -> Result<Vec<Project>, ModrinthError>
pub async fn get_team_members(team_id: &str) -> Result<Vec<TeamMember>, ModrinthError>

pub async fn get_categories() -> Result<Vec<Category>, ModrinthError>
pub async fn get_loaders() -> Result<Vec<Loader>, ModrinthError>
pub async fn get_game_versions() -> Result<Vec<GameVersion>, ModrinthError>
pub async fn get_project_types() -> Result<Vec<String>, ModrinthError>
```

## Hashes

```rust
pub fn sha1_file(path: &Path) -> Result<String, ModrinthError>
pub fn sha512_file(path: &Path) -> Result<String, ModrinthError>
pub fn sha1_bytes(bytes: &[u8]) -> String
pub fn sha512_bytes(bytes: &[u8]) -> String
pub fn verify_sha1(path: &Path, expected: &str) -> Result<(), ModrinthError>
pub fn verify_sha512(path: &Path, expected: &str) -> Result<(), ModrinthError>
```

A mismatch returns `ModrinthError::HashMismatch { expected, actual }`.

## Installation

```rust
pub async fn install_project(id_or_slug: &str, params: &InstallParams) -> Result<InstallResult, ModrinthError>
pub async fn install_version(version_id: &str, params: &InstallParams) -> Result<InstallResult, ModrinthError>
pub async fn install_from_hash(hash: &str, params: &InstallParams) -> Result<InstallResult, ModrinthError>
pub async fn install_datapack(id_or_slug: &str, world_name: &str, game_path: &Path, params: &InstallParams) -> Result<InstallResult, ModrinthError>
```

`InstallParams`:

| Field | Meaning |
|---|---|
| `game_path` | Game directory, used for automatic target resolution. |
| `install_dir` | Explicit target directory; wins over `game_path`. |
| `minecraft_version` | Filter versions by this Minecraft version. |
| `loader` | Filter versions by this loader. |
| `version_type` | `Release` / `Beta` / `Alpha`. |
| `install_required_dependencies` | Also install required dependencies (cycle-safe). |
| `overwrite` | Overwrite an existing file with the same name. |

### Automatic directories

With `game_path` and no `install_dir`:

| Content type | Directory |
|---|---|
| Mod | `<game>/mods/` |
| Resource pack | `<game>/resourcepacks/` |
| Shader | `<game>/shaderpacks/` |
| Data pack | use `install_datapack` → `<game>/saves/<world>/datapacks/` |
| Modpack | requires explicit `install_dir` |
| World | requires explicit `install_dir` |

### Version selection

If `install_project` is used, the version is chosen by `minecraft_version` + `loader` + `version_type`. When no compatible version exists, the call fails with `ModrinthError::NoCompatibleVersion` — a random version is never installed.

### Filename conflicts

If the target file exists:

- `overwrite = true` — replace it;
- otherwise, if the existing file's SHA-1 matches the expected one, keep it;
- otherwise, a numbered name is chosen (`name (1).jar`, `name (2).jar`, …).

Downloads are verified against SHA-1 and SHA-512 from the API, then written atomically — a failed download never leaves a partial file.

```rust
use rmlib::extra::modrinth::{install, InstallParams};
use rmlib::extra::modrinth::dto::VersionType;

let params = InstallParams {
    game_path: Some("./game".into()),
    minecraft_version: Some("1.21.1".to_string()),
    loader: Some("fabric".to_string()),
    version_type: Some(VersionType::Release),
    install_required_dependencies: true,
    ..Default::default()
};
let result = install::install_project("sodium", &params).await?;
for f in result.installed {
    println!("{} ({} bytes)", f.path.display(), f.bytes);
}
```

### Data packs

```rust
use rmlib::extra::modrinth::install::install_datapack;

let result = install_datapack("terralith", "MyWorld", std::path::Path::new("./game"), &params).await?;
```

The world directory must already exist; the module does not create a fake save. An invalid world name (empty, `.`, `..`, containing `/` or `\`) is rejected.

### Dependencies

`install_required_dependencies` walks the version dependencies, installs required ones that have a compatible version, and avoids cycles via a visited-project set.

## Error type

```rust
pub enum ModrinthError {
    Http(reqwest::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Config(musutils::fs::config::ConfigError),
    Status { code: u16, body: String },
    Api { code: String, description: String },
    InvalidResponse(String),
    NotFound(String),
    RateLimited { retry_after: Option<u64> },
    HashMismatch { expected: String, actual: String },
    UnsupportedContentType(String),
    NoCompatibleVersion(String),
    DependencyCycle(String),
    InvalidArgument(String),
}
```

HTTP 4xx/5xx are converted from Modrinth's JSON error body into `Api { code, description }`; `404` also surfaces as `NotFound` for direct lookups; `429` maps to `RateLimited`.

## Limitations

- Plugins and servers are not supported.
- `modpack` / `world` installs require an explicit `install_dir`.
- Network calls are not covered by the default test suite; tests cover query building, pagination, DTO deserialization, hash checks and install-path logic.

## See also

- [Config](../core/config.md)
- [Fabric](fabric.md) / [NeoForge](neoforge.md)
- [FFI — Modrinth](../ffi/modrinth.md)