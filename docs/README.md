# rmlib

**Rust Minecraft Launcher library**

[![License: LGPL-3.0](https://img.shields.io/badge/license-LGPL--3.0-blue.svg)](https://github.com/musdev13/rmlib/blob/dev/LICENSE)

`rmlib` is a Rust library for building Minecraft launchers. It handles the platform-independent parts of a launcher: version fetching, installation, mod loader management, launch preparation and execution, and optional Ely.by integration.

The crate is **modular** — `core` is always available; each additional area under `extra` is its own Cargo feature. The crate also builds a `cdylib` (`librmlib.so` / `librmlib.dylib` / `rmlib.dll`) so the same logic can be driven from any language that can call a C ABI.

## Features

- **`core`** — version fetching, installation (client JAR, libraries, assets, natives), path configuration, command assembly and game launch.
- **`ely`** — Ely.by authentication (`login`), `authlib-injector` patch installation (`install_patch`), and a local beta skin proxy (`start_beta_proxy`).
- **`fabric`** — Fabric Loader version listing and installation.
- **`neoforge`** — NeoForge version listing and installation.
- **`ffi`** — C ABI exports (`rmlib_*`) and a `cdylib` for use from Python, C/C++, Go, Node.js, and any other language with C FFI support.

`default = ["core", "ely", "fabric", "neoforge"]`.

## Installation

`rmlib` is not published on crates.io yet. Add it as a git dependency:

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib" }
```

To enable only some areas:

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core", "fabric"] }
```

See [Features](pages/getting-started/features.md) in the documentation for the full feature graph.

## Quick start

```rust
use rmlib::core::config;
use rmlib::core::version;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    config::init(false);

    let versions = version::fetch_list().await;
    println!("{} versions available", versions.len());

    version::install::install(
        Some("1.20.1".into()),
        None,
        None, None, None,
        true, true, true, true,
        150, 350,
    ).await;

    Ok(())
}
```

Launching the game:

```rust
use rmlib::core::config;
use rmlib::core::run::run_client;

let versions = config::get_versions_path();
let assets = config::get_assets_path();
let libs = config::get_libs_path();
let natives = format!("{libs}/natives");
let game = "~/.minecraft";

run_client(
    "1.20.1",
    None, None,
    "4G",
    "Player",
    "00000000-0000-0000-0000-000000000000",
    "xuid",
    "token",
    false, false,
    &versions, &assets, &libs, &natives, game, game,
    &[], &[],
).await;
```

## Building a shared library

The crate produces a `cdylib` in addition to the usual `rlib`. To build it with all FFI exports:

```bash
make build-ffi          # → target/release/librmlib.so (or platform equivalent)
make dist               # → same, plus copies artifacts into dist/
```

Calling it from Python:

```python
import ctypes

lib = ctypes.CDLL("./target/release/librmlib.so")
lib.rmlib_version.restype = ctypes.c_void_p
lib.rmlib_string_free.argtypes = [ctypes.c_void_p]

ptr = lib.rmlib_version()
print(ctypes.string_at(ptr).decode())  # "0.10.0"
lib.rmlib_string_free(ptr)
```

Full C ABI reference: [FFI documentation](pages/ffi/index.md).

## Documentation

Full documentation lives in the [`docs/`](docs/) directory of this repository:

- [Installation](docs/pages/getting-started/installation.md)
- [Features](docs/pages/getting-started/features.md)
- [Core API](docs/pages/lib.md)
- [Core modules](docs/pages/core/config.md)
- [Extra modules](docs/pages/extra/ely.md)
- [FFI](docs/pages/ffi/index.md)

The same content is served as a static site when the MkDocs site is built from `mkdocs.yml`.

## Building and testing

A convenience `Makefile` is provided:

```bash
make test          # cargo test with default features
make test-ffi      # cargo test --features ffi (needed for tests/ffi.rs)
make lint          # fmt-check + clippy + clippy-ffi
make check         # test + test-ffi + lint + docs-check
make build-ffi     # cdylib with FFI exports
make dist          # build-ffi + copy into dist/
make docs          # build the MkDocs site
make docs-serve    # live preview
make docs-check    # mkdocs build --strict
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the crate layout, feature conventions, testing requirements, and the checklist before submitting a PR. Pull requests target the `dev` branch.

## License

`rmlib` is distributed under the [GNU Lesser General Public License v3.0](LICENSE). By submitting a PR you agree to the license and to the terms of the [Trademark Policy](TRADEMARK.md).