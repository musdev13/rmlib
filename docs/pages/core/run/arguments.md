# core::run — Arguments

Rule evaluation, placeholder substitution, argument-list parsing, and building the final JVM + game argument vectors.

This module is the "assembly" layer: it takes a version JSON and a `LaunchContext`, and produces the two `Vec<String>`s that go before and after the main class in the final `java` command.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

```rust
use rmlib::core::run::{
    build_arguments, check_rules, get_library_group_artifact, parse_argument_list,
    replace_placeholders, assemble_argv, assemble_command,
    LaunchArguments,
};
```

## `check_rules`

```rust
pub fn check_rules(rules: &[Value], context: &LaunchContext) -> bool
```

Evaluates a `rules` array from a library or an argument entry. Returns whether the entry is **allowed** on the current OS/architecture.

### Semantics

- Starts with `allowed = false`.
- Iterates the rules in order.
- Rules with a `features` field are **skipped** (this launcher does not implement the `features` system — those entries are treated as "does not apply").
- For each remaining rule:
  - `action` is read (`"allow"` or `"disallow"`; default `"allow"` if missing).
  - If the rule has no `os` filter, it always matches → `allowed = is_allow`.
  - If the rule has an `os` filter, it matches only when both `os.name` (if present) equals `context.my_os` and `os.arch` (if present) normalizes to the same architecture as `context.my_arch`.
  - When the rule matches, `allowed` is set to `is_allow`.
- The **last matching** rule wins.

### Architecture normalization

`os.arch` and `context.my_arch` are both normalized before comparison:

| Input | Normalized |
|---|---|
| `"64"`, `"x86_64"`, `"amd64"`, `"x64"` | `"x86_64"` |
| `"32"`, `"x86"`, `"i386"`, `"i686"` | `"x86"` |
| `"arm64"`, `"aarch64"` | `"arm64"` |
| `"arm"`, `"arm32"` | `"arm"` |
| anything else | `"unknown"` |

So `{"os": {"arch": "x86_64"}}` matches `my_arch = "64"`, and `{"os": {"arch": "64"}}` matches `my_arch = "x86_64"`.

### Examples

```rust
use rmlib::core::run::check_rules;
use rmlib::core::run::dto::LaunchContext;
use serde_json::json;
use std::collections::HashMap;

let ctx = LaunchContext {
    my_os: "linux".to_string(),
    my_arch: "64".to_string(),
    placeholders: HashMap::new(),
};

assert!(!check_rules(&[], &ctx));

assert!(check_rules(&[json!({"action": "allow"})], &ctx));
assert!(!check_rules(&[json!({"action": "disallow"})], &ctx));

// Missing action defaults to allow.
assert!(check_rules(&[json!({})], &ctx));

// OS-specific allow.
assert!(check_rules(
    &[json!({"action": "allow", "os": {"name": "linux"}})],
    &ctx,
));
assert!(!check_rules(
    &[json!({"action": "allow", "os": {"name": "windows"}})],
    &ctx,
));

// Architecture, with normalization.
assert!(check_rules(
    &[json!({"action": "allow", "os": {"arch": "x86_64"}})],
    &ctx,
));

// `features` entries are skipped.
assert!(!check_rules(
    &[json!({"action": "allow", "features": {"is_demo_user": true}})],
    &ctx,
));

// Last matching rule wins.
assert!(!check_rules(
    &[
        json!({"action": "allow"}),
        json!({"action": "disallow", "os": {"name": "linux"}}),
    ],
    &ctx,
));
```

## `get_library_group_artifact`

```rust
pub fn get_library_group_artifact(name: &str) -> String
```

Returns the first two `:`-separated parts of a Maven coordinate (group + artifact).

- `"a:b:c"` → `"a:b"`
- `"a:b"` → `"a:b"`
- `"a"` → `"a"` (no change)
- `""` → `""`

Used as the deduplication key in `verify_libraries` — see [Classpath](classpath.md#resolvedlibrary).

```rust
use rmlib::core::run::get_library_group_artifact;

assert_eq!(get_library_group_artifact("net.minecraft:client:1.20.1"), "net.minecraft:client");
```

## `replace_placeholders`

```rust
pub fn replace_placeholders(arg: &str, placeholders: &HashMap<String, String>) -> String
```

Replaces every `${key}` in `arg` with the value from `placeholders`. Keys that do not appear in the map are left as-is.

- Replaces **all** occurrences of each `${key}`.
- Does not interpret `${…}` recursively; the replacement is inserted literally.
- Iteration order over the map does not affect the result, because keys are distinct.

```rust
use std::collections::HashMap;
use rmlib::core::run::replace_placeholders;

let mut map = HashMap::new();
map.insert("name".to_string(), "Steve".to_string());

assert_eq!(replace_placeholders("hi ${name}!", &map), "hi Steve!");
assert_eq!(replace_placeholders("${name}+${name}", &map), "Steve+Steve");
assert_eq!(replace_placeholders("${unknown}", &map), "${unknown}");
```

The set of placeholders used by the launcher is produced by `build_launch_context` — see [Arguments](arguments.md).

## `parse_argument_list`

```rust
pub fn parse_argument_list(args_value: &Value, context: &LaunchContext, out: &mut Vec<String>)
```

Parses a version JSON's `arguments.<group>` array (either `"jvm"` or `"game"`) into a flat `Vec<String>`.

Two entry shapes are supported (via `serde(untagged)` on `JsonArgument`):

```json
"a plain string"
```

```json
{
  "rules": [ ... ],
  "value": "single string"
}
```

```json
{
  "rules": [ ... ],
  "value": ["array", "of", "strings"]
}
```

For each entry:

- A plain string is pushed as-is.
- A conditional entry with `rules` is evaluated via `check_rules`; if allowed, its `value` (string or array of strings) is pushed.

Entries that don't deserialize (unknown shape, non-string array elements, etc.) are **silently skipped**.

`args_value` is typically `version_json["arguments"]["jvm"]` or `version_json["arguments"]["game"]`. If `args_value` is not an array, nothing is pushed.

### Example

```rust
use rmlib::core::run::parse_argument_list;
use rmlib::core::run::dto::LaunchContext;
use serde_json::json;
use std::collections::HashMap;

let ctx = LaunchContext {
    my_os: "linux".to_string(),
    my_arch: "64".to_string(),
    placeholders: HashMap::new(),
};

let jvm = json!([
    "-Xmx2G",
    {"rules": [{"action": "allow", "os": {"name": "linux"}}], "value": "-Dlinux=1"},
    {"rules": [{"action": "allow", "os": {"name": "windows"}}], "value": "-Dwin=1"}
]);

let mut out = Vec::new();
parse_argument_list(&jvm, &ctx, &mut out);
assert_eq!(out, vec!["-Xmx2G", "-Dlinux=1"]);
```

## `LaunchArguments`

```rust
pub struct LaunchArguments {
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
}
```

The two argument vectors produced by `build_arguments`.

| Field | Meaning |
|---|---|
| `jvm_args` | Arguments to pass **before** the main class (`-Xmx…`, `-D…`, `-cp …`, `-javaagent:…`, …). |
| `game_args` | Arguments to pass **after** the main class (`--username`, `--gameDir`, …). |

## `build_arguments`

```rust
#[allow(clippy::too_many_arguments)]
pub fn build_arguments(
    ram_count: &str,
    ely: bool,
    betafix: bool,
    neoforge_version: Option<&str>,
    paths: &LaunchPaths,
    version_data: &VersionData,
    context: &LaunchContext,
    classpath: &ClasspathData,
    custom_jvm_args: &[String],
    custom_game_args: &[String],
    logs: bool,
) -> LaunchArguments
```

Assembles the JVM and game argument vectors for a launch.

### What goes into `jvm_args`, in order

1. `-Xmx<ram_count>` — heap size.
2. `-Djava.library.path=<paths.native>` — native library directory.
3. If `ely == true`:
   - If `betafix == false`: if `paths.libs/authlib-injector.jar` exists, `-javaagent:<path>=ely.by`; otherwise a warning is printed.
   - If `betafix == true`: `-Dhttp.proxyHost=127.0.0.1 -Dhttp.proxyPort=43523 -Dhttp.nonProxyHosts=localhost|127.0.0.1`.
4. If `ely == false` and `betafix == true`: `-Dhttp.proxyHost=betacraft.uk`.
5. From `version_data.version_json.arguments.jvm` (if present) — parsed by `parse_argument_list`.
6. From `version_data.modloader_json.arguments.jvm` (if present) — parsed by `parse_argument_list`.
7. If `neoforge_version.is_some()` and neither `-p`/`--module-path` nor `-cp`/`-classpath` already appear in `jvm_args`, appends `-cp <classpath>`.
8. If `neoforge_version.is_none()` and no `-cp`/`-classpath` already appears, appends `-cp <classpath>`.
9. All `custom_jvm_args`, in order.

### What goes into `game_args`, in order

1. From `version_data.version_json.arguments.game` (if present) — parsed by `parse_argument_list`.
2. Otherwise, if `version_data.version_json.minecraftArguments` is a string (pre-1.13 versions), each whitespace-separated token is pushed. Placeholders are **not** substituted here; they are left in the string and resolved later by `assemble_argv`.
3. From `version_data.modloader_json.arguments.game` (if present) — parsed by `parse_argument_list`.
4. All `custom_game_args`, in order.

### Parameters

| Parameter | Meaning |
|---|---|
| `ram_count` | e.g. `"2G"`, `"3500M"` — used verbatim as `-Xmx<ram_count>`. |
| `ely` | Whether to inject `authlib-injector` (or the beta proxy). |
| `betafix` | Whether to enable the beta-fix proxy. |
| `neoforge_version` | If `Some`, assumes NeoForge and does the `-cp` fallback only if the loader did not add its own. |
| `paths` | `paths.native` is used for `-Djava.library.path`; `paths.libs` is used to look for `authlib-injector.jar`. |
| `version_data` | Provides `arguments.jvm`, `arguments.game`, `minecraftArguments`, and the mod loader JSON. |
| `context` | Used by `parse_argument_list` when evaluating rule-gated entries. |
| `classpath` | Used when `-cp` needs to be added. |
| `custom_jvm_args` / `custom_game_args` | Appended at the end of the respective vector, verbatim. |
| `logs` | If `true`, prints an extra message when `ely` is set but `authlib-injector.jar` is missing. |

### Example

```rust
use rmlib::core::run::{
    build_arguments, build_classpath, build_launch_context, load_version_data,
    prepare_paths, resolve_environment,
};

let paths = prepare_paths(
    "1.20.1",
    "~/rml/versions", "~/rml/assets", "~/rml/libs",
    "~/rml/libs/natives", "~/.minecraft",
).unwrap();
let data = load_version_data("1.20.1", None, None, &paths, false).unwrap();
let env = resolve_environment();
let classpath = build_classpath(
    &data.version_json, data.modloader_json.as_ref(), &paths,
    &data.loader_libs_path, &env,
);
let context = build_launch_context(
    "1.20.1", "Steve", "uuid", "xuid", "token",
    false, &data, &paths, &env, &classpath,
);

let args = build_arguments(
    "4G", false, false, None,
    &paths, &data, &context, &classpath,
    &["-XX:+UseG1GC".to_string()],
    &[],
    false,
);

assert!(args.jvm_args.iter().any(|a| a == "-Xmx4G"));
assert!(args.jvm_args.iter().any(|a| a == "-XX:+UseG1GC"));
```

### Note on placeholders

`build_arguments` does **not** replace `${…}` placeholders in the argument strings it returns. Substitution happens one step later, in `assemble_argv` — see below.

## `assemble_argv`

```rust
pub fn assemble_argv(
    main_class: &str,
    arguments: LaunchArguments,
    context: &LaunchContext,
) -> Vec<String>
```

Combines `LaunchArguments` and the main class into a full `java` argument vector.

1. Runs `replace_placeholders` on every string in `jvm_args` (using `context.placeholders`).
2. Runs `replace_placeholders` on every string in `game_args`.
3. Builds the final vector: `["java", <jvm args…>, <main_class>, <game args…>]`.

The first element is always the literal `"java"`.

```rust
use rmlib::core::run::assemble_argv;
use rmlib::core::run::LaunchArguments;
use rmlib::core::run::dto::LaunchContext;
use std::collections::HashMap;

let mut placeholders = HashMap::new();
placeholders.insert("auth_player_name".to_string(), "Steve".to_string());

let ctx = LaunchContext {
    my_os: "linux".to_string(),
    my_arch: "64".to_string(),
    placeholders,
};

let args = LaunchArguments {
    jvm_args: vec!["-Xmx2G".to_string(), "-Duser=${auth_player_name}".to_string()],
    game_args: vec!["--username".to_string(), "${auth_player_name}".to_string()],
};

let argv = assemble_argv("Main", args, &ctx);
assert_eq!(
    argv,
    vec!["java", "-Xmx2G", "-Duser=Steve", "Main", "--username", "Steve"]
);
```

## `assemble_command`

```rust
pub fn assemble_command(
    main_class: &str,
    arguments: LaunchArguments,
    context: &LaunchContext,
) -> String
```

Same as `assemble_argv`, but returns the arguments joined into a single shell-style string.

- Each argument that contains a space is wrapped in `"…"`.
- Arguments without spaces are emitted verbatim.
- No shell escaping beyond the space rule — this is intended for display and for `--just-command`-style output, not for direct `sh -c` execution.

```rust
use rmlib::core::run::assemble_command;
use rmlib::core::run::LaunchArguments;
use rmlib::core::run::dto::LaunchContext;
use std::collections::HashMap;

let ctx = LaunchContext {
    my_os: "linux".to_string(),
    my_arch: "64".to_string(),
    placeholders: HashMap::new(),
};

let args = LaunchArguments {
    jvm_args: vec!["-Dkey=has space".to_string()],
    game_args: vec![],
};

let cmd = assemble_command("Main", args, &ctx);
assert!(cmd.contains("\"-Dkey=has space\""));
```

## Order of operations in a full launch

```text
load_version_data
   ├── version_json.arguments.jvm / .game
   └── modloader_json.arguments.jvm / .game
            │
            ▼
build_arguments ──> LaunchArguments { jvm_args, game_args }
            │        (placeholders still ${…})
            ▼
assemble_argv ──> Vec<String> ["java", …jvm…, main_class, …game…]
            │        (placeholders substituted here)
            ▼
run_client ──> spawn java
```

## Placeholders used

The launcher's `build_launch_context` populates `context.placeholders` with these keys. `assemble_argv` replaces `${key}` for each:

| Placeholder | Value |
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

See [dto](dto.md) for the full `LaunchContext` structure.

## See also

- [Classpath](classpath.md) — `build_classpath`, `verify_libraries`
- [Command](command.md) — `build_argv`, `build_command`, `run_client`
- [DTO](dto.md) — `LaunchContext`, `JsonArgument`, `JsonArgValue`