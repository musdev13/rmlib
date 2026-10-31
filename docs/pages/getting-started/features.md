# Features

`rmlib` is fully feature-gated. `core` is the base launcher; each area under `src/extra/` is its own feature; `ffi` turns on the C ABI exports.

## Feature graph

```text
core
├── ely       (implies core)
├── fabric    (implies core)
├── neoforge  (implies core)
└── profiles  (implies core)

ffi           (independent of area features)
```

`default = ["core", "ely", "fabric", "neoforge", "profiles"]`.

## `core`

The base launcher. Turns on:

- `rmlib::core::config` — configuration files and path getters.
- `rmlib::core::types` — `VersionType`, `build_maven_path`.
- `rmlib::core::version` — manifest fetching, version JSON, sorting, stripping.
- `rmlib::core::version::install` — client JAR / libraries / assets download, natives extraction, installation orchestration.
- `rmlib::core::run` — path preparation, classpath, arguments, command assembly, game launching.

`core` has **no** dependency on any `extra` area, and it must keep compiling with:

```bash
cargo build --no-default-features --features core
```

## `extra` areas

### `ely`

Ely.by authentication and the `authlib-injector` patch. Implies `core`.

- `rmlib::extra::ely::login` — authenticate with Ely.by, returns `AuthResult`.
- `rmlib::extra::ely::install_patch` — download `authlib-injector.jar` into the libs directory.
- `rmlib::extra::ely::start_beta_proxy` — start a local skin proxy for old beta versions.

### `profiles`

Saved account profiles for Ely.by and Microsoft accounts. Implies `core`.

- `rmlib::extra::profiles::Profile` / `ProfileKind` — a stored account: username, uuid, access token, and optional client token / xuid / license flag.
- `rmlib::extra::profiles::{save_profile, load_profile, delete_profile, list_profiles}` — store/read profiles under `auth_path/profiles/{ely,ms}/<username>.json`.
- `rmlib::extra::profiles::InstanceConfig` — a sparse launch-settings file (`rml.json`), saved and loaded with `save_instance` / `load_instance`, merged with `merge_missing_from`.
- Every function has an `_at` variant taking an explicit auth directory, mirroring `core::auth`.

### `fabric`

Fabric Loader version listing and installation. Implies `core`.

- `rmlib::extra::fabric::fetch_list` — list Fabric Loader versions for a given Minecraft version.
- `rmlib::extra::fabric::install` — soft-install the base version, write the Fabric profile JSON, and download Fabric libraries.

### `neoforge`

NeoForge version listing and installation. Implies `core`.

- `rmlib::extra::neoforge::fetch_list` — list NeoForge versions matching a Minecraft version.
- `rmlib::extra::neoforge::install` — soft-install the base version, download the NeoForge installer, run it, and move libraries and the version JSON into place.

## `ffi`

Turns on `rmlib::ffi` and the C ABI exports (`rmlib_*`). Does **not** imply any area feature — the exports are gated individually:

- Core exports under `#[cfg(feature = "core")]`.
- `rmlib_fabric_*` under `#[cfg(feature = "fabric")]`.
- `rmlib_neoforge_*` under `#[cfg(feature = "neoforge")]`.
- `rmlib_ely_*` under `#[cfg(feature = "ely")]`.

The crate always sets `crate-type = ["rlib", "cdylib"]`, so `librmlib.so` / `.dylib` / `.dll` is produced on every `cargo build`. Without `ffi` it is an empty `.so` — no exported symbols. With `ffi` it contains the exports.

## Cross-feature interaction

- The launch flags `ely: bool` and `betafix: bool` in `core::run` are **runtime flags**, not a compile-time dependency on `extra::ely`. You can build with only `core` and still pass `ely: true` — the launcher will inject `-javaagent:authlib-injector.jar=ely.by` if the JAR exists.
- `available_modules()` reports the enabled areas (`core`, `ely`, `fabric`, `neoforge`, `profiles`) — see [Core API](../lib.md).
- `ffi` exports for an area are gated with that area's feature. Building `--features "core ffi"` produces a `.so` without `rmlib_fabric_*` / `rmlib_neoforge_*` / `rmlib_ely_*`.

## Inter-feature conflicts

There are none: area features are additive, and `ffi` can be combined with any subset.

## Recommended feature sets

| Use case | Features |
|---|---|
| Library for a custom launcher (vanilla only) | `core` |
| Launcher with mod loaders | `core`, `fabric`, `neoforge` |
| Launcher with Ely.by auth | `core`, `ely` |
| Launcher with saved account profiles | `core`, `profiles` |
| Full featured launcher | `core`, `ely`, `fabric`, `neoforge`, `profiles` (default) |
| Shared library for other languages (vanilla only) | `core`, `ffi` |
| Shared library for other languages (full) | `core`, `ely`, `fabric`, `neoforge`, `profiles`, `ffi` |

## See also

- [Installation](installation.md)
- [Core API](../lib.md)
- [FFI overview](../ffi/index.md)