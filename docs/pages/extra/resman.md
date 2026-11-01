# extra::resman

Resource manager for locally installed Minecraft content.

`resman` itself is a thin namespace: it contains no types or functions of its own. Everything is provided by its submodules, each behind its own feature:

| Submodule | Feature | Purpose |
|---|---|---|
| `resman::mods` | `resman-mods` | Scan, inspect and toggle mods. |
| `resman::resourcepacks` | `resman-resourcepacks` | Scan and inspect resource packs. |

Both features imply `resman`, which implies `core`. The `resman` module exists only when at least one of them is enabled.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["resman", "resman-mods"] }
# or
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["resman", "resman-resourcepacks"] }
# or both
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["resman", "resman-mods", "resman-resourcepacks"] }
```

```rust
use rmlib::extra::resman::mods;
use rmlib::extra::resman::resourcepacks;
```

## Layout

```text
src/extra/resman/
├── mod.rs                 # gate only: pub mod mods / pub mod resourcepacks
├── mods/                  # feature = "resman-mods"
│   ├── dto.rs             # ModInfo, ModDependency, ModLoader
│   ├── error.rs           # ModError
│   ├── fields.rs          # field selection
│   ├── format.rs          # json / table / plain rendering
│   ├── icon.rs            # icon extraction to the temp dir
│   ├── info.rs            # info_from_file, info_by_id
│   ├── info_priority.rs   # info_priority (async, priority + fallback)
│   ├── parse.rs           # JAR parsing (NeoForge → Fabric)
│   ├── scan.rs            # scan, scan_dir
│   ├── scan_priority.rs   # scan_priority (async)
│   ├── source.rs          # Source, Priority, FallbackPolicy, Field
│   └── toggle.rs          # enable / disable / toggle / is_enabled
└── resourcepacks/         # feature = "resman-resourcepacks"
    ├── dto.rs             # ResourcepackInfo
    ├── error.rs           # RpError
    ├── fields.rs
    ├── format.rs
    ├── icon.rs
    ├── info.rs
    ├── info_priority.rs
    ├── parse.rs           # pack.mcmeta parsing, pack_format → MC
    ├── scan.rs
    ├── scan_priority.rs
    └── source.rs
```

The two submodules are independent: they have their own `Source`, `Priority`, `FallbackPolicy`, `Field` and error types.

## Common behaviour

Both submodules:

- Discover content from a game directory (`<game>/mods/`, `<game>/resourcepacks/`) or from an explicit directory.
- Treat `<name>.jar.disabled` / `<name>.zip.disabled` as disabled content.
- Extract icons into the OS temp directory: `std::env::temp_dir()/rmlib_icons/<key>.png`.
- Support a `Priority` list of information sources (`local`, `modrinth`) and a `FallbackPolicy` used when no source resolves a file.
- Offer field selection and three output formats (`json`, `table`, `plain`) for both listing and single-file info.

See the submodule pages for the full API:

- [resman::mods](resman-mods.md)
- [resman::resourcepacks](resman-resourcepacks.md)

## See also

- [Config](../core/config.md) — `game_path` and other paths.
- [FFI — resman::mods](../ffi/resman-mods.md)
- [FFI — resman::resourcepacks](../ffi/resman-resourcepacks.md)