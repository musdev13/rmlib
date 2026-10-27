# core::run — Classpath

Building the Java classpath from a version JSON: filtering libraries against the current OS/architecture, checking each library JAR exists on disk, and joining the result into a classpath string.

Two functions:

- `verify_libraries` — turns a version JSON into a list of existing library JARs.
- `build_classpath` — combines the version's libraries, the mod loader's libraries, and the client JAR into a classpath string.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

```rust
use rmlib::core::run::{build_classpath, verify_libraries, ClasspathData};
use rmlib::core::run::dto::ResolvedLibrary;
```

## `ResolvedLibrary`

```rust
pub struct ResolvedLibrary {
    pub key: String,
    pub path: PathBuf,
}
```

One library resolved to an on-disk path. Returned by `verify_libraries`, consumed by `build_classpath` for deduplication.

| Field | Meaning |
|---|---|
| `key` | A deduplication key. Two entries with the same `key` are the same library — only the first is kept in the classpath. |
| `path` | The absolute path to the library's `.jar` file on disk. |

### Key format

The `key` is derived from the library's Maven coordinate and its classifier:

| Case | `key` |
|---|---|
| Native classifier present | `<group>:<artifact>:<classifier>` |
| Library name contains `natives-` | `<group>:<artifact>:<relative-jar-path>` |
| Otherwise | `<group>:<artifact>` |

`<group>:<artifact>` is `get_library_group_artifact(name)` — the first two `:`-separated parts of the Maven coordinate.

This means a library with and without a native classifier have distinct keys, and only one copy of each logical library ends up in the classpath.

## `verify_libraries`

```rust
pub fn verify_libraries(
    version_json: &serde_json::Value,
    libs_path: &Path,
    context: &LaunchContext,
) -> Vec<ResolvedLibrary>
```

Walks `version_json.libraries`, filters by rules, resolves each library's JAR path inside `libs_path`, and returns only entries whose JAR **exists** on disk.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_json` | A parsed version (or mod loader) JSON. Only `libraries` is read. |
| `libs_path` | The libraries directory. Each library's relative path is joined onto it. |
| `context` | Provides `my_os` and `my_arch` for rule evaluation. `placeholders` is not used by this function. |

### Behaviour

1. Iterates `libraries` (if absent, returns empty).
2. For each library, evaluates `rules` (if any) via `check_rules`. Libraries whose rules disallow the current OS/arch are skipped.
3. Determines the library's relative JAR path(s):
   - If `downloads` is present and the library has a native classifier for `my_os` (with `${arch}` replaced by `my_arch`), the classifier's `path` is used.
   - If `downloads.artifact.path` is present, it is used too.
   - If neither is present but `build_maven_path(name, None)` succeeds, that path is used as a fallback.
   - An `arch_matches_classifier` check drops paths whose file name (without `.jar`) suggests a different architecture than `my_arch`. This filters out, for example, `natives-linux-arm64` on an x86_64 machine.
4. Joins each relative path onto `libs_path`.
5. If the resulting JAR does **not** exist on disk, prints a `Status::Warn: Missing library -> <path>` line to stdout and drops the entry.
6. Otherwise, pushes a `ResolvedLibrary { key, path }`.

### Returns

A `Vec<ResolvedLibrary>` of libraries that are present on disk, in input order. Duplicate entries are **not** removed here — deduplication happens in `build_classpath`.

### Example

```rust
use std::path::Path;
use rmlib::core::run::dto::LaunchContext;
use rmlib::core::run::verify_libraries;
use std::collections::HashMap;

let context = LaunchContext {
    my_os: "linux".to_string(),
    my_arch: "64".to_string(),
    placeholders: HashMap::new(),
};

let libs = verify_libraries(&version_json, Path::new("/tmp/libs"), &context);
for lib in &libs {
    println!("{} -> {:?}", lib.key, lib.path);
}
```

### What ends up in `libs` on disk

For a vanilla 1.20.1 install, the libraries directory looks like:

```text
libs/
├── com/mojang/...
├── net/java/dev/jna/...
├── org/lwjgl/lwjgl/3.3.1/lwjgl-3.3.1.jar
├── org/lwjgl/lwjgl/3.3.1/lwjgl-3.3.1-natives-linux.jar
└── ...
```

`verify_libraries` picks out the entries that (a) survive the rules, (b) match the current architecture, and (c) are present on disk. The classpath for a vanilla launch is the result of this list joined by the platform separator, plus the client JAR.

## `ClasspathData`

```rust
pub struct ClasspathData {
    pub classpath: String,
    pub separator: String,
}
```

The final classpath string and the platform-specific separator used to build it. Produced by [`build_classpath`](#build_classpath).

| Field | Meaning |
|---|---|
| `classpath` | The full classpath string: libraries + client JAR, joined by `separator`. |
| `separator` | `";"` on Windows, `":"` on everything else. |

Both fields are used downstream: `classpath` becomes the value of the `-cp` JVM argument (or the `${classpath}` placeholder), and `separator` becomes the value of the `${classpath_separator}` placeholder (used by NeoForge's argument templates).

## `build_classpath`

```rust
pub fn build_classpath(
    version_json: &serde_json::Value,
    modloader_json: Option<&serde_json::Value>,
    paths: &LaunchPaths,
    loader_libs_path: &Path,
    environment: &LaunchEnvironment,
) -> ClasspathData
```

Builds the classpath for a launch.

### Parameters

| Parameter | Meaning |
|---|---|
| `version_json` | The vanilla version JSON. |
| `modloader_json` | If a mod loader is in use, its profile JSON. Its libraries are resolved against `loader_libs_path`. |
| `paths` | Used for `paths.libs` (vanilla libraries) and `paths.client_jar`. |
| `loader_libs_path` | The directory where the mod loader's libraries are stored (`libs/modloaders/fabric` or `libs/modloaders/neoforge`). Only used when `modloader_json` is `Some`. |
| `environment` | Provides `my_os` / `my_arch` for both `verify_libraries` calls. |

### Behaviour

1. If `modloader_json` is `Some`, calls `verify_libraries(modloader_json, loader_libs_path, context)` and collects the results.
2. Calls `verify_libraries(version_json, paths.libs, context)` and collects the results after the mod loader's libraries.
3. Deduplicates by `ResolvedLibrary.key`: the first occurrence wins. Order in the classpath is: **mod loader libraries first**, then vanilla libraries.
4. If `paths.client_jar` exists on disk, appends it at the end. If it does not exist, it is omitted — the launcher still produces a classpath.
5. Joins all paths with the platform separator (`";"` on Windows, `":"` otherwise).

### Returns

`ClasspathData { classpath, separator }`.

### Example

```rust
use rmlib::core::run::{
    build_classpath, prepare_paths, resolve_environment,
};

let paths = prepare_paths(
    "1.20.1",
    "~/rml/versions", "~/rml/assets", "~/rml/libs",
    "~/rml/libs/natives", "~/.minecraft",
).unwrap();
let environment = resolve_environment();

// Vanilla only.
let cp = build_classpath(&version_json, None, &paths, &paths.libs, &environment);

// Fabric.
let cp = build_classpath(
    &version_json,
    Some(&fabric_profile_json),
    &paths,
    &paths.libs.join("modloaders").join("fabric"),
    &environment,
);

println!("{}", cp.classpath);
println!("separator: {}", cp.separator);
```

### Missing client JAR

If `paths.client_jar` does not exist:

- The classpath still contains the libraries.
- The client JAR entry is skipped.
- `core::run::run_client` will still spawn `java`, but the game will very likely fail to start.

This is deliberate: `build_classpath` does not fail on a missing client JAR — that is the caller's responsibility (`prepare_paths` only checks for the JSON).

## How the pieces fit together

```text
version_json ──┐
               ├─> verify_libraries ──> Vec<ResolvedLibrary> ──┐
modloader_json ┘                                              │
                                                              ├─> dedup by key ──> join ──> ClasspathData
paths.client_jar ─────────────────────────────────────────────┘
```

- `verify_libraries` is called **twice**: once per JSON.
- `build_classpath` performs the deduplication and the final join.
- `separator` is passed onward to `build_launch_context` (it becomes the `${classpath_separator}` placeholder) and to `build_arguments` (it controls how `-cp` is emitted on NeoForge).

## See also

- [Paths](paths.md) — `prepare_paths`, `load_version_data`, `resolve_environment`
- [Arguments](arguments.md) — `build_arguments`, `check_rules`
- [Command](command.md) — `build_argv`, `build_command`