# core::types

Shared types used across the launcher: the version classification enum and the Maven coordinate helper.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

```rust
use rmlib::core::types::{VersionType, build_maven_path};
```

## `VersionType`

```rust
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}
```

Classification of a Minecraft version, matching the `type` field of entries in Mojang's version manifest. Derives `ValueEnum`, `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq` — `ValueEnum` is what makes it usable as a `clap` CLI argument.

### `VersionType::as_str`

```rust
pub fn as_str(&self) -> &'static str
```

Returns the string used in the manifest's `type` field:

| Variant | `as_str()` |
|---|---|
| `Release` | `"release"` |
| `Snapshot` | `"snapshot"` |
| `OldBeta` | `"old_beta"` |
| `OldAlpha` | `"old_alpha"` |

```rust
use rmlib::core::types::VersionType;

assert_eq!(VersionType::Release.as_str(), "release");
assert_eq!(VersionType::OldBeta.as_str(), "old_beta");
```

This is the same string `version::sort_by_type` filters by — see [Version](version.md#sort_by_type).

## `build_maven_path`

```rust
pub fn build_maven_path(name: &str, classifier: Option<&str>) -> Option<String>
```

Converts a Maven coordinate into a relative file path in a Maven repository.

### Input

`name` is a Maven coordinate of the form `group:artifact:version[:classifier][@ext]`:

- `group` — dotted, e.g. `net.minecraft`.
- `artifact` — e.g. `client`.
- `version` — e.g. `1.20.1`.
- Optional `classifier` — a fourth segment, e.g. `natives-linux`. Only used if the `classifier` argument is `None`.
- Optional `@ext` — the extension, e.g. `@jar`. Ignored (the resulting path always ends in `.jar`).

`classifier` — an override for the classifier. If `Some`, it takes priority over any fourth segment in `name`. Pass `None` to use the fourth segment (or omit the classifier entirely).

### Output

Returns `Some(path)` where `path` is a relative file path:

```text
<group with dots replaced by slashes>/<artifact>/<version>/<artifact>-<version>[-<classifier>].jar
```

Returns `None` when `name` has fewer than three `:`-separated parts.

### Examples

Basic coordinate:

```rust
use rmlib::core::types::build_maven_path;

assert_eq!(
    build_maven_path("net.minecraft:client:1.20.1", None).unwrap(),
    "net/minecraft/client/1.20.1/client-1.20.1.jar"
);
```

Classifier as an argument:

```rust
assert_eq!(
    build_maven_path("net.minecraft:client:1.20.1", Some("natives-linux")).unwrap(),
    "net/minecraft/client/1.20.1/client-1.20.1-natives-linux.jar"
);
```

Classifier as a fourth segment:

```rust
assert_eq!(
    build_maven_path("net.minecraft:client:1.20.1:natives-linux", None).unwrap(),
    "net/minecraft/client/1.20.1/client-1.20.1-natives-linux.jar"
);
```

The argument wins over the segment:

```rust
assert_eq!(
    build_maven_path(
        "net.minecraft:client:1.20.1:natives-windows",
        Some("natives-linux")
    ).unwrap(),
    "net/minecraft/client/1.20.1/client-1.20.1-natives-linux.jar"
);
```

`@jar` suffix:

```rust
assert_eq!(
    build_maven_path("net.minecraft:client:1.20.1@jar", None).unwrap(),
    "net/minecraft/client/1.20.1/client-1.20.1.jar"
);
```

Dotted group:

```rust
assert_eq!(
    build_maven_path("com.example.deep.group:artifact:1.0", None).unwrap(),
    "com/example/deep/group/artifact/1.0/artifact-1.0.jar"
);
```

Too few parts — `None`:

```rust
assert!(build_maven_path("net.minecraft:client", None).is_none());
assert!(build_maven_path("onlyone", None).is_none());
assert!(build_maven_path("", None).is_none());
```

An empty classifier argument is kept verbatim (produces a trailing `-.jar`), not treated as "no classifier":

```rust
assert_eq!(
    build_maven_path("net.minecraft:client:1.20.1", Some("")).unwrap(),
    "net/minecraft/client/1.20.1/client-1.20.1-.jar"
);
```

## Where these types are used

- `VersionType` — `version::sort_by_type` filters manifest entries by type. In the CLI it is the value type of the `--type` argument.
- `build_maven_path` — `run::verify_libraries` and `version::install::download_libraries` build file paths inside the libraries directory from the `name` field of each library in a version JSON.

## See also

- [Version](version.md) — `sort_by_type`, `strip_version`
- [core::run — Classpath](run/classpath.md) — `verify_libraries`