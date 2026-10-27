# rmlib

**Rust Minecraft Launcher library**

[GitHub](https://github.com/musdev13/rmlib) ·
[Contributing](https://github.com/musdev13/rmlib/blob/dev/CONTRIBUTING.md) ·
[License: LGPL-3.0](https://github.com/musdev13/rmlib/blob/dev/LICENSE)

---

## What is rmlib

`rmlib` is a Rust library for building Minecraft launchers. It handles the parts of a launcher that are the same regardless of UI:

- fetching and parsing the Mojang version manifest;
- downloading the client JAR, libraries, and assets, with SHA-1 verification;
- installing and managing mod loaders (Fabric, NeoForge);
- assembling the JVM command line — classpath, native libraries, game arguments, auth placeholders;
- launching the game;
- optional Ely.by integration (authentication, `authlib-injector` patch, beta skin proxy).

The crate is **modular**: `core` is always available; each additional area in `extra` is its own Cargo feature. A `cdylib` is also produced, so the library can be called from other languages through a C ABI.

## Quick start

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib" }
```

```rust
use rmlib::core::config;
use rmlib::core::version;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rmlib::core::config::init(false);

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

See [Installation](pages/getting-started/installation.md) for details, and [Features](pages/getting-started/features.md) for what each Cargo feature turns on.

## Where to go next

- **[Installation](pages/getting-started/installation.md)** — how to add `rmlib` and pick features.
- **[Features](pages/getting-started/features.md)** — `core`, `ely`, `fabric`, `neoforge`, `ffi`.
- **[Core API](pages/lib.md)** — `VERSION`, `NAME`, `available_modules()`, and other crate-root items.
- **[Core module](pages/core/config.md)** — configuration, version fetching, installation, launch.
- **[Extra modules](pages/extra/ely.md)** — Ely.by, Fabric, NeoForge.
- **[FFI](pages/ffi/index.md)** — C ABI for using `librmlib.so` from other languages.

## Requirements

- Rust `edition = "2024"` (stable toolchain).
- Java (for actually launching the game — `core::run::run_client` spawns `java`).

## License

`rmlib` is distributed under the [GNU Lesser General Public License v3.0](https://github.com/musdev13/rmlib/blob/dev/LICENSE).