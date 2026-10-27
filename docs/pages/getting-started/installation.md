# Installation

`rmlib` is not published on crates.io yet. Add it as a git dependency.

## As a git dependency

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib" }
```

Pin to a specific revision or branch for reproducible builds:

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", rev = "<commit-sha>" }
```

## As a path dependency

When working on `rmlib` locally, point `Cargo.toml` at the checkout:

```toml
[dependencies]
rmlib = { path = "../rmlib" }
```

## Choosing features

The crate is split into features. `core` is the base launcher; every area under `extra/` is its own feature. `default = ["core", "ely", "fabric", "neoforge"]`.

| Feature | What it enables |
|---|---|
| `core` | Version fetching, installation, launch preparation and execution, config, shared types. |
| `ely` | Ely.by auth, `authlib-injector` patch, beta skin proxy. Implies `core`. |
| `fabric` | Fabric Loader version listing and installation. Implies `core`. |
| `neoforge` | NeoForge version listing and installation. Implies `core`. |
| `ffi` | C ABI exports and the `cdylib`. Does not imply any area feature. |

See [Features](features.md) for the full feature graph.

### Minimal setup (core only)

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

### Typical launcher (core + all mod loaders)

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core", "fabric", "neoforge"] }
```

### With Ely.by

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core", "ely"] }
```

### Building a shared library for other languages

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", features = ["ffi"] }
```

Or simply run `make build-ffi` in the crate checkout — it produces `target/release/librmlib.so` (Linux), `librmlib.dylib` (macOS) or `rmlib.dll` (Windows).

## Runtime requirements

- **Java.** `core::run::run_client` spawns `java` from `PATH`. The launcher has to use the same Java installation that is set up for the game.
- **Network access.** Version listing, downloads and (optionally) Ely.by auth require network access. Downloads go to Mojang's CDN, `libraries.minecraft.net`, `meta.fabricmc.net`, `maven.neoforged.net` and `authserver.ely.by`.
- **Configuration.** `core::config::init(false)` creates `~/.config/rml/paths.json` (Linux/macOS) or `%APPDATA%\rml\paths.json` (Windows). If the file already exists, it is kept.

## Building and testing locally

```bash
make test          # cargo test with default features
make test-ffi      # + tests/ffi.rs
make lint          # fmt-check + clippy + clippy-ffi
make check         # everything above plus docs-check
make build-ffi     # cdylib with FFI exports
make dist          # build-ffi + copy artifacts into dist/
```

`make docs` builds the MkDocs site; `make docs-serve` serves it with live reload; `make docs-check` builds in strict mode (warnings become errors).

## Toolchain

- Rust `edition = "2024"` (stable toolchain).
- The crate uses `tokio` (multi-threaded runtime), `reqwest` (with TLS), `serde` / `serde_json`, `zip`, `sha1`, `roxmltree`, and `musutils`.
- No system dependencies beyond a working Rust toolchain and `mkdocs` (only if you build the documentation).