# Contributing to rmlib / Вклад в rmlib

---

## EN

Thanks for your interest in the project!

### Structure

`rmlib` is split into two independent Cargo features:

- `src/core/` — **core launcher functionality** (feature `core`): version fetching, installation, mod loader management, launch preparation and execution, path configuration, shared types.
- `src/extra/` — **additional/optional functionality** (each area gets its own feature): mod managers, server managers, custom profiles, Ely.by integration, and any other nice-to-have launcher extensions.

Right now `src/extra/` contains only the `ely` module (feature `ely`, depends on `core` via `ely = ["core"]`). New areas under `extra/` should follow the same pattern: a subdirectory per area, a matching Cargo feature, `#[cfg(feature = "...")]` gating in `src/extra/mod.rs` and `src/lib.rs`.

`default = ["core", "ely"]` — both currently available features are on by default.

```
src/
├── lib.rs              # feature gating + ModuleInfo + available_modules()
├── core/               # feature = "core"
│   ├── mod.rs
│   ├── config/
│   │   ├── get_assets_path.rs
│   │   ├── get_libs_path.rs
│   │   ├── get_versions_path.rs
│   │   ├── init.rs
│   │   ├── paths.rs
│   │   └── mod.rs
│   ├── modloaders/
│   │   ├── fabric/     { fetch_list.rs, install.rs, mod.rs }
│   │   ├── neoforge/   { fetch_list.rs, install.rs, mod.rs }
│   │   └── mod.rs
│   ├── run/            # assemble_command, build_arguments, build_classpath, run_client, ...
│   ├── types/
│   │   ├── maven.rs
│   │   ├── version.rs
│   │   └── mod.rs
│   ├── version/
│   │   ├── install/    # download_client_jar, download_libraries, install, process_assets, ...
│   │   ├── fetch_list.rs
│   │   ├── get_version_json.rs
│   │   ├── get_version_json_url.rs
│   │   ├── sort_by_type.rs
│   │   ├── strip_version.rs
│   │   └── mod.rs
│   └── test.rs
└── extra/              # feature = "ely" (currently the only area here)
    ├── mod.rs
    └── ely/
        ├── auth/
        │   ├── dto.rs
        │   ├── install_patch.rs
        │   ├── login.rs
        │   └── mod.rs
        ├── beta_proxy_server.rs
        └── mod.rs
```

`mod.rs` files are only for `mod` declarations and `pub use` re-exports. Do not put the whole implementation of a module into its `mod.rs`.

### Rules

1. **All changes must be submitted to the `dev` branch.** Open a Pull Request against `dev`, not `main`.

2. **Keep the `core` / `extra` split.**
   - Base launcher functionality → `src/core/` (in the appropriate submodule: `config/`, `modloaders/`, `run/`, `types/`, `version/`).
   - Additional/optional functionality → `src/extra/<area>/` (e.g. `extra/ely/`, and future areas like `extra/mods/`, `extra/servers/`, `extra/profiles/`).

3. **Every `extra` area is its own Cargo feature.** Create `<area> = ["core"]` in `Cargo.toml`, gate the submodule with `#[cfg(feature = "<area>")] pub mod <area>;` in `src/extra/mod.rs`, and gate the whole `pub mod extra;` with `#[cfg(any(feature = "ely", feature = "<area>", ...))]` in `src/lib.rs`. The `extra` crate module must not be enabled if no area feature is on.

4. **Do not make `core` depend on `extra`.** `core` must compile and work with `--no-default-features --features core`. If a piece of `core` needs optional add-on functionality, either move that piece to `extra/<area>/`, or gate it with `#[cfg(feature = "<area>")]` and document the contract.

5. **One file per logical operation.** Do not put multiple unrelated functions into one file. Follow the existing layout (e.g. `run/assemble_command.rs`, `run/build_arguments.rs`, `run/build_classpath.rs`, ...; `version/install/download_libraries.rs`, `version/install/process_assets.rs`, ...).

6. **Keep the public API stable where possible.** Breaking changes to `core` or an `extra` area should be justified in the PR description and, if non-trivial, noted in `CHANGELOG` (if it exists) or the release notes.

7. **Use `musutils` for shared helpers.** File operations, HTTP downloads, OS/arch detection, colored status output — `rmlib` uses `musutils` (v2 modules) for these. Do not re-implement them locally. If a helper is genuinely missing in `musutils`, propose adding it there first.

8. **`core::run` keeps `ely: bool` / `betafix: bool` parameters.** These are JVM argument flags (`-javaagent:authlib-injector.jar=ely.by`, `-Dhttp.proxyHost=...`), not a compile-time dependency on any `extra` feature. Do not move the flag handling out of `core::run`.

9. **`available_modules()` must reflect the actual build.** `rmlib::available_modules()` returns the top-level modules (`core`, and each enabled `extra` area) that were compiled into the current build via `#[cfg(feature = "...")]`. When you add a new `extra` area, add a corresponding `#[cfg]` branch there. Do not hardcode or filter the list in dependent crates.

10. **No unnecessary dependencies.** Prefer `std`; add crates only when they are genuinely useful. New crates should already be needed by a concrete feature.

11. **Run the checks before pushing:**
    ```bash
    cargo fmt --check
    cargo check
    cargo check --no-default-features --features core
    cargo check --no-default-features --features ely
    cargo check --no-default-features
    cargo test
    cargo clippy --all-targets
    ```

12. By submitting a Pull Request, you agree that your code will be distributed under the **LGPL-3.0** license.

13. By submitting a Pull Request, you agree to the terms of the **Trademark Policy** (see `TRADEMARK.md`).

---

Questions and suggestions — in Issues.

---

## RU

Спасибо за интерес к проекту!

### Структура

`rmlib` разделена на две независимые категории Cargo-фич:

- `src/core/` — **базовая функциональность лаунчера** (фича `core`): получение и установка версий, работа с модлоадерами, подготовка и запуск игры, конфигурация путей, общие типы.
- `src/extra/` — **дополнительная/опциональная функциональность** (каждая область — отдельная фича): менеджеры модов, менеджеры серверов, кастомные профили, интеграция с Ely.by и другие приятные расширения лаунчера.

Сейчас `src/extra/` содержит только модуль `ely` (фича `ely`, зависит от `core` через `ely = ["core"]`). Новые области в `extra/` следуют тому же принципу: подкаталог на область, соответствующая Cargo-фича, гейтинг через `#[cfg(feature = "...")]` в `src/extra/mod.rs` и `src/lib.rs`.

`default = ["core", "ely"]` — обе текущие фичи включены по умолчанию.

```
src/
├── lib.rs              # гейтинг фич + ModuleInfo + available_modules()
├── core/               # фича = "core"
│   ├── mod.rs
│   ├── config/
│   │   ├── get_assets_path.rs
│   │   ├── get_libs_path.rs
│   │   ├── get_versions_path.rs
│   │   ├── init.rs
│   │   ├── paths.rs
│   │   └── mod.rs
│   ├── modloaders/
│   │   ├── fabric/     { fetch_list.rs, install.rs, mod.rs }
│   │   ├── neoforge/   { fetch_list.rs, install.rs, mod.rs }
│   │   └── mod.rs
│   ├── run/            # assemble_command, build_arguments, build_classpath, run_client, ...
│   ├── types/
│   │   ├── maven.rs
│   │   ├── version.rs
│   │   └── mod.rs
│   ├── version/
│   │   ├── install/    # download_client_jar, download_libraries, install, process_assets, ...
│   │   ├── fetch_list.rs
│   │   ├── get_version_json.rs
│   │   ├── get_version_json_url.rs
│   │   ├── sort_by_type.rs
│   │   ├── strip_version.rs
│   │   └── mod.rs
│   └── test.rs
└── extra/              # фича = "ely" (пока единственная область)
    ├── mod.rs
    └── ely/
        ├── auth/
        │   ├── dto.rs
        │   ├── install_patch.rs
        │   ├── login.rs
        │   └── mod.rs
        ├── beta_proxy_server.rs
        └── mod.rs
```

`mod.rs` — только для объявлений `mod` и реэкспорта через `pub use`. Не складывайте всю реализацию модуля в его `mod.rs`.

### Правила

1. **Все изменения отправляются в ветку `dev`.** Pull Request открывается против `dev`, а не `main`.

2. **Сохраняйте разделение `core` / `extra`.**
   - Базовая функциональность → `src/core/` (в соответствующий подмодуль: `config/`, `modloaders/`, `run/`, `types/`, `version/`).
   - Дополнительная/опциональная функциональность → `src/extra/<area>/` (например, `extra/ely/` и будущие области вроде `extra/mods/`, `extra/servers/`, `extra/profiles/`).

3. **Каждая область в `extra` — отдельная Cargo-фича.** Создайте `<area> = ["core"]` в `Cargo.toml`, оберните подмодуль `#[cfg(feature = "<area>")] pub mod <area>;` в `src/extra/mod.rs`, а весь `pub mod extra;` оберните `#[cfg(any(feature = "ely", feature = "<area>", ...))]` в `src/lib.rs`. Модуль `extra` не должен существовать, если не включена ни одна фича-область.

4. **`core` не должен зависеть от `extra`.** `core` обязан собираться и работать с `--no-default-features --features core`. Если какой-то части `core` нужна опциональная функциональность из `extra`, — либо перенесите эту часть в `extra/<area>/`, либо оберните `#[cfg(feature = "<area>")]` и задокументируйте контракт.

5. **Один файл — одна логическая операция.** Не складывайте несколько несвязанных функций в один файл. Следуйте существующей раскладке (например, `run/assemble_command.rs`, `run/build_arguments.rs`, `run/build_classpath.rs`, ...; `version/install/download_libraries.rs`, `version/install/process_assets.rs`, ...).

6. **Сохраняйте публичный API, где это возможно.** Ломающие изменения в `core` или в области `extra` должны быть обоснованы в описании PR и, если изменение нетривиальное, отражены в `CHANGELOG` (если он есть) или в release notes.

7. **Используйте `musutils` для общих хелперов.** Файловые операции, HTTP-загрузки, определение OS/arch, цветной вывод статусов — `rmlib` использует для этого `musutils` (модули v2). Не реализуйте их локально повторно. Если хелпера реально не хватает в `musutils` — предложите сначала добавить его туда.

8. **В `core::run` остаются параметры `ely: bool` / `betafix: bool`.** Это флаги JVM-аргументов (`-javaagent:authlib-injector.jar=ely.by`, `-Dhttp.proxyHost=...`), а не зависимость от какой-либо фичи `extra` на этапе компиляции. Не переносите обработку этих флагов из `core::run`.

9. **`available_modules()` должна отражать реальную сборку.** `rmlib::available_modules()` возвращает верхнеуровневые модули (`core` и каждую включённую область `extra`), вошедшие в текущую сборку через `#[cfg(feature = "...")]`. При добавлении новой области `extra` добавьте соответствующую `#[cfg]`-ветку туда же. Не хардкодьте и не фильтруйте этот список в зависимых крейтах.

10. **Никаких лишних зависимостей.** Предпочитайте `std`; добавляйте crates только при реальной пользе. Новые крейты должны быть уже нужны конкретной фиче.

11. **Перед пушем прогоняйте проверки:**
    ```bash
    cargo fmt --check
    cargo check
    cargo check --no-default-features --features core
    cargo check --no-default-features --features ely
    cargo check --no-default-features
    cargo test
    cargo clippy --all-targets
    ```

12. Отправляя Pull Request, вы соглашаетесь, что ваш код будет распространяться под лицензией **LGPL-3.0**.

13. Отправляя Pull Request, вы соглашаетесь с условиями **политики использования товарных знаков** (см. `TRADEMARK.md`).

---

Вопросы и предложения — в Issues.