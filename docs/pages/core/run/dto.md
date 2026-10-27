# core::run — DTO

Plain data types used by `core::run`. None of them have behaviour beyond their fields — they exist so the assembly steps can pass structured data around instead of long argument lists.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

```rust
use rmlib::core::run::dto::{LaunchContext, JsonArgument, JsonArgValue, ResolvedLibrary};
```

## `LaunchContext`

```rust
pub struct LaunchContext {
    pub my_os: String,
    pub my_arch: String,
    pub placeholders: HashMap<String, String>,
}
```

Everything the argument and library layers need to know about the current launch.

| Field | Meaning |
|---|---|
| `my_os` | Normalized OS string used to match `rules[i].os.name` — `"windows"`, `"linux"`, `"osx"` or `"unknown"`. |
| `my_arch` | Normalized arch string used to match `rules[i].os.arch` — `"64"`, `"32"`, `"arm64"`, `"arm32"` or `"unknown"`. |
| `placeholders` | The map used by `replace_placeholders` — `${key}` → value. Populated by `build_launch_context`, applied later by `assemble_argv`. |

`LaunchContext` is built from a `LaunchEnvironment` (which only has `my_os` / `my_arch`) plus the placeholder map. `build_classpath` constructs a `LaunchContext` internally with an **empty** placeholder map — that is enough because `verify_libraries` only reads `my_os` / `my_arch`.

### Example

```rust
use rmlib::core::run::dto::LaunchContext;
use std::collections::HashMap;

let mut placeholders = HashMap::new();
placeholders.insert("auth_player_name".to_string(), "Steve".to_string());

let ctx = LaunchContext {
    my_os: "linux".to_string(),
    my_arch: "64".to_string(),
    placeholders,
};

assert_eq!(ctx.placeholders["auth_player_name"], "Steve");
```

### Placeholder keys

`build_launch_context` populates these keys. Hand-rolled `LaunchContext`s should include at least the ones referenced by the arguments in your version JSON — otherwise those placeholders stay as literal `${…}` in the final argv.

| Key | Value |
|---|---|
| `auth_player_name` | `username` |
| `auth_uuid` | `uuid` |
| `auth_access_token` | `token` |
| `auth_xuid` | `xuid` |
| `user_type` | `"msa"` or `"mojang"` (mojang when `ely`) |
| `version_name` | `version_id` |
| `version_type` | `version_json.type` (default `"release"`) |
| `game_directory` | `paths.game` |
| `assets_root` | `paths.assets` |
| `assets_index_name` | `version_data.asset_index_name` |
| `natives_directory` | `paths.native` |
| `game_assets` | `paths.assets` |
| `classpath` | `classpath.classpath` |
| `classpath_separator` | `classpath.separator` |
| `library_directory` | `version_data.loader_libs_path` |
| `launcher_name` | `rmlib::NAME` |
| `launcher_version` | `rmlib::VERSION` |
| `clientid` | `rmlib::CLIENT_ID` |

## `JsonArgument`

```rust
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum JsonArgument {
    Simple(String),
    Conditional {
        rules: Vec<Value>,
        value: JsonArgValue,
    },
}
```

The two entry shapes allowed in a version JSON's `arguments.<group>` array. It exists so `parse_argument_list` can deserialize the entries with `serde_json` and branch on the shape.

| Variant | JSON shape |
|---|---|
| `Simple` | `"some string"` |
| `Conditional` | `{ "rules": [...], "value": <JsonArgValue> }` |

Entries that match neither shape fail to deserialize and are silently skipped by `parse_argument_list`.

## `JsonArgValue`

```rust
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum JsonArgValue {
    Single(String),
    Many(Vec<String>),
}
```

The `value` field of a `Conditional` entry. Mojang's version JSONs use both shapes — a conditional can expand to a single argument or to several.

| Variant | JSON shape |
|---|---|
| `Single` | `"one string"` |
| `Many` | `["a", "b", "c"]` |

`parse_argument_list` pushes a `Single` as one entry and extends the output with every element of a `Many`.

## `ResolvedLibrary`

```rust
pub struct ResolvedLibrary {
    pub key: String,
    pub path: PathBuf,
}
```

A library that was found on disk, plus the key used to deduplicate it.

| Field | Meaning |
|---|---|
| `key` | Deduplication key — `<group>:<artifact>`, plus classifier for native libraries. |
| `path` | Absolute path to the `.jar` on disk. |

Produced by `verify_libraries`, consumed by `build_classpath`. Full details of the `key` format are on the [Classpath](classpath.md#resolvedlibrary) page.

## How the DTOs flow

```text
prepare_paths         →  LaunchPaths
load_version_data     →  VersionData
resolve_environment   →  LaunchEnvironment ──┐
                                             ├── build_launch_context ──> LaunchContext
classpath             →  ClasspathData ──────┘
                                                 │
                                                 ├── parse_argument_list  (uses LaunchContext)
                                                 ├── build_arguments      (uses LaunchContext)
                                                 └── assemble_argv        (uses LaunchContext)

verify_libraries      →  Vec<ResolvedLibrary>  ──> build_classpath
```

- `LaunchContext` is built **after** the classpath (it needs `${classpath}` and `${classpath_separator}`).
- `parse_argument_list` needs a `LaunchContext` to evaluate rule-gated arguments.
- `LaunchPaths`, `VersionData`, `LaunchEnvironment` are covered on the [Paths](paths.md) page; `ClasspathData` on the [Classpath](classpath.md) page.

## See also

- [Paths](paths.md) — `LaunchPaths`, `VersionData`, `LaunchEnvironment`
- [Classpath](classpath.md) — `ResolvedLibrary`, `ClasspathData`
- [Arguments](arguments.md) — `check_rules`, `parse_argument_list`, `build_arguments`