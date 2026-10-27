# Contributing to rmlib / Вклад в rmlib

---

## EN

Thanks for your interest in the project!

### Structure

`rmlib` is split into two independent Cargo feature groups:

- `src/core/` — **core launcher functionality** (feature `core`): version fetching, installation, launch preparation and execution, path configuration, shared types.
- `src/extra/` — **additional/optional functionality** (each area gets its own feature): mod loaders, mod managers, server managers, custom profiles, Ely.by integration, and any other nice-to-have launcher extensions.

`src/extra/` currently contains three areas:

- `extra/ely` (feature `ely`) — Ely.by auth, authlib-injector patch, beta skin proxy.
- `extra/fabric` (feature `fabric`) — Fabric Loader: version listing and installation.
- `extra/neoforge` (feature `neoforge`) — NeoForge: version listing and installation.

`default = ["core", "ely", "fabric", "neoforge"]` — all currently available features are on by default. Each `extra` area feature implies `core` (`ely = ["core"]`, `fabric = ["core"]`, `neoforge = ["core"]`).

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
├── extra/              # feature = "ely" | "fabric" | "neoforge"
│   ├── mod.rs
│   ├── ely/
│   │   ├── auth/
│   │   │   ├── dto.rs
│   │   │   ├── install_patch.rs
│   │   │   ├── login.rs
│   │   │   └── mod.rs
│   │   ├── beta_proxy_server.rs
│   │   └── mod.rs
│   ├── fabric/
│   │   ├── fetch_list.rs
│   │   ├── install.rs
│   │   └── mod.rs
│   └── neoforge/
│       ├── fetch_list.rs
│       ├── install.rs
│       └── mod.rs
└── ffi/                # feature = "ffi" — C ABI exports (see "FFI" section)
    ├── mod.rs
    ├── error.rs
    ├── exports.rs
    ├── runtime.rs
    ├── strings.rs
    └── strings.rs

tests/
├── common/mod.rs       # TempDir, HomeGuard, env_lock helpers
├── config.rs
├── ely_proxy.rs
├── ffi.rs              # #[cfg(feature = "ffi")]
├── install.rs
├── lib.rs
├── run_fs.rs
├── run_pure.rs
├── types.rs
└── version.rs

Makefile                # make test / test-ffi / lint / check / build-ffi / dist
```

`mod.rs` files are only for `mod` declarations and `pub use` re-exports. Do not put the whole implementation of a module into its `mod.rs`.

### Rules

1. **All changes must be submitted to the `dev` branch.** Open a Pull Request against `dev`, not `main`.

2. **Keep the `core` / `extra` split.**
   - Base launcher functionality → `src/core/` (in the appropriate submodule: `config/`, `run/`, `types/`, `version/`).
   - Additional/optional functionality → `src/extra/<area>/` (e.g. `extra/ely/`, `extra/fabric/`, `extra/neoforge/`, and future areas like `extra/mods/`, `extra/servers/`, `extra/profiles/`).

3. **Every `extra` area is its own Cargo feature.** Create `<area> = ["core"]` in `Cargo.toml`, gate the submodule with `#[cfg(feature = "<area>")] pub mod <area>;` in `src/extra/mod.rs`, and gate the whole `pub mod extra;` with `#[cfg(any(feature = "ely", feature = "fabric", feature = "neoforge", ...))]` in `src/lib.rs`. The `extra` crate module must not be enabled if no area feature is on.

4. **Do not make `core` depend on `extra`.** `core` must compile and work with `--no-default-features --features core`. If a piece of `core` needs optional add-on functionality, either move that piece to `extra/<area>/`, or gate it with `#[cfg(feature = "<area>")]` and document the contract.

5. **One file per logical operation.** Do not put multiple unrelated functions into one file. Follow the existing layout (e.g. `run/assemble_command.rs`, `run/build_arguments.rs`, `run/build_classpath.rs`, ...; `version/install/download_libraries.rs`, `version/install/process_assets.rs`, ...).

6. **Keep the public API stable where possible.** Breaking changes to `core` or an `extra` area should be justified in the PR description and, if non-trivial, noted in `CHANGELOG` (if it exists) or the release notes.

7. **Use `musutils` for shared helpers.** File operations, HTTP downloads, OS/arch detection, colored status output — `rmlib` uses `musutils` (v2 modules) for these. Do not re-implement them locally. If a helper is genuinely missing in `musutils`, propose adding it there first.

8. **`core::run` keeps `ely: bool` / `betafix: bool` parameters.** These are JVM argument flags (`-javaagent:authlib-injector.jar=ely.by`, `-Dhttp.proxyHost=...`), not a compile-time dependency on any `extra` feature. Do not move the flag handling out of `core::run`.

9. **`available_modules()` must reflect the actual build.** `rmlib::available_modules()` returns the top-level modules (`core`, and each enabled `extra` area) that were compiled into the current build via `#[cfg(feature = "...")]`. When you add a new `extra` area, add a corresponding `#[cfg]` branch there. Do not hardcode or filter the list in dependent crates.

10. **No unnecessary dependencies.** Prefer `std`; add crates only when they are genuinely useful. New crates should already be needed by a concrete feature.

11. **Tests are required for every change.** This is not optional:
    - **New public item** (function, method, type, enum variant, field, macro, FFI export) — add tests that cover normal behavior, boundary values, and error/edge cases where they exist.
    - **Changed behavior or signature** — update the existing tests that assert the old behavior, and add new ones for the new behavior.
    - **Bug fix** — add a regression test that fails without the fix and passes with it.
    - **New module, `extra` area, or feature** — a dedicated test file under `tests/` with a top-level `#[cfg(feature = "...")]` gate matching the existing layout.

    Place tests in `tests/` (integration tests) when exercising the public API. Use an inline `#[cfg(test)] mod tests` inside a source file only when the code under test is private (`pub(crate)` / private) and cannot be reached from an integration test.

    Tests must be deterministic, independent, safe for the filesystem (use `tests/common::TempDir`, never real user paths), and resilient to parallel execution. Tests that touch `HOME` or other environment variables must guard themselves with `tests/common::HomeGuard` / `tests/common::env_lock()`. Do not hard-code network access — for the parts that need the network (version fetching, mod loader lists, auth, etc.), do not write integration tests; cover the surrounding pure logic instead.

12. **Tests must actually be run.** Before pushing, run `make test` and `make test-ffi`. Both must pass. A PR that adds functionality with tests that were not run locally is incomplete.

13. **Every public change is reflected in documentation.** This is also required:
    - **New or changed public item** (function, type, module, FFI export) — update the corresponding documentation: doc comments on the item itself (`///`, `//!`) and, if the project ships user-facing docs, the relevant `docs/` page.
    - **Changes to features, feature dependencies, or setup** — update `Cargo.toml` docs, this file, and the feature list in the README (if present).
    - **Breaking changes** — call them out in the PR description and in `CHANGELOG` / release notes.
    - **FFI exports** — keep `rmlib.h` (if present) in sync with the exported symbols. Adding or removing an `rmlib_*` function requires a matching change in the header.

14. **FFI (C ABI).** The crate builds a `cdylib` in addition to the usual `rlib`. Public C ABI lives under `src/ffi/` behind the `ffi` feature and is exposed with `#[unsafe(no_mangle)] pub unsafe extern "C" fn rmlib_*`. Rules:
    - Every new FFI export must have a corresponding test in `tests/ffi.rs` and be listed in `tests/lib.rs`'s `available_modules`-related assertions if it changes the surface.
    - FFI exports must not panic across the boundary. Use `catch_unwind`; return `NULL` for pointers or `-1` for integers on error; store the message via `set_last_error` so callers can retrieve it with `rmlib_last_error()`.
    - Strings returned to the caller must be freed with `rmlib_string_free`.
    - Do not change an existing export's signature without bumping the version and documenting the change.

15. **Run the checks before pushing:**
    ```bash
    make fmt-check
    make clippy
    make clippy-ffi
    make test
    make test-ffi
    ```

    Or, in a single step:

    ```bash
    make check
    ```

    `make check` runs `test`, `test-ffi`, `fmt-check`, `clippy`, and `clippy-ffi`. All must be green.

16. **Shared library artifacts.** To produce `librmlib.so` (Linux) / `librmlib.dylib` (macOS) / `rmlib.dll` (Windows), run:

    ```bash
    make build-ffi      # → target/release/librmlib.so (or platform equivalent)
    make dist           # → same, plus copies artifacts into dist/
    ```

    Do not commit artifacts from `target/` or `dist/` — both are ignored by `.gitignore`.

17. By submitting a Pull Request, you agree that your code will be distributed under the **LGPL-3.0** license.

18. By submitting a Pull Request, you agree to the terms of the **Trademark Policy** (see `TRADEMARK.md`).

---

Questions and suggestions — in Issues.

---

## RU

Спасибо за интерес к проекту!

### Структура

`rmlib` разделена на две независимые категории Cargo-фич:

- `src/core/` — **базовая функциональность лаунчера** (фича `core`): получение и установка версий, подготовка и запуск игры, конфигурация путей, общие типы.
- `src/extra/` — **дополнительная/опциональная функциональность** (каждая область — отдельная фича): модлоадеры, менеджеры модов, менеджеры серверов, кастомные профили, интеграция с Ely.by и другие приятные расширения лаунчера.

Сейчас `src/extra/` содержит три области:

- `extra/ely` (фича `ely`) — авторизация Ely.by, патч authlib-injector, beta-прокси для скинов.
- `extra/fabric` (фича `fabric`) — Fabric Loader: список версий и установка.
- `extra/neoforge` (фича `neoforge`) — NeoForge: список версий и установка.

`default = ["core", "ely", "fabric", "neoforge"]` — все текущие фичи включены по умолчанию. Каждая фича-область в `extra` подразумевает `core` (`ely = ["core"]`, `fabric = ["core"]`, `neoforge = ["core"]`).

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
├── extra/              # фича = "ely" | "fabric" | "neoforge"
│   ├── mod.rs
│   ├── ely/
│   │   ├── auth/
│   │   │   ├── dto.rs
│   │   │   ├── install_patch.rs
│   │   │   ├── login.rs
│   │   │   └── mod.rs
│   │   ├── beta_proxy_server.rs
│   │   └── mod.rs
│   ├── fabric/
│   │   ├── fetch_list.rs
│   │   ├── install.rs
│   │   └── mod.rs
│   └── neoforge/
│       ├── fetch_list.rs
│       ├── install.rs
│       └── mod.rs
└── ffi/                # фича = "ffi" — экспорты C ABI (см. раздел «FFI»)
    ├── mod.rs
    ├── error.rs
    ├── exports.rs
    ├── runtime.rs
    └── strings.rs

tests/
├── common/mod.rs       # хелперы TempDir, HomeGuard, env_lock
├── config.rs
├── ely_proxy.rs
├── ffi.rs              # #[cfg(feature = "ffi")]
├── install.rs
├── lib.rs
├── run_fs.rs
├── run_pure.rs
├── types.rs
└── version.rs

Makefile                # make test / test-ffi / lint / check / build-ffi / dist
```

`mod.rs` — только для объявлений `mod` и реэкспорта через `pub use`. Не складывайте всю реализацию модуля в его `mod.rs`.

### Правила

1. **Все изменения отправляются в ветку `dev`.** Pull Request открывается против `dev`, а не `main`.

2. **Сохраняйте разделение `core` / `extra`.**
   - Базовая функциональность → `src/core/` (в соответствующий подмодуль: `config/`, `run/`, `types/`, `version/`).
   - Дополнительная/опциональная функциональность → `src/extra/<area>/` (например, `extra/ely/`, `extra/fabric/`, `extra/neoforge/` и будущие области вроде `extra/mods/`, `extra/servers/`, `extra/profiles/`).

3. **Каждая область в `extra` — отдельная Cargo-фича.** Создайте `<area> = ["core"]` в `Cargo.toml`, оберните подмодуль `#[cfg(feature = "<area>")] pub mod <area>;` в `src/extra/mod.rs`, а весь `pub mod extra;` оберните `#[cfg(any(feature = "ely", feature = "fabric", feature = "neoforge", ...))]` в `src/lib.rs`. Модуль `extra` не должен существовать, если не включена ни одна фича-область.

4. **`core` не должен зависеть от `extra`.** `core` обязан собираться и работать с `--no-default-features --features core`. Если какой-то части `core` нужна опциональная функциональность из `extra`, — либо перенесите эту часть в `extra/<area>/`, либо оберните `#[cfg(feature = "<area>")]` и задокументируйте контракт.

5. **Один файл — одна логическая операция.** Не складывайте несколько несвязанных функций в один файл. Следуйте существующей раскладке (например, `run/assemble_command.rs`, `run/build_arguments.rs`, `run/build_classpath.rs`, ...; `version/install/download_libraries.rs`, `version/install/process_assets.rs`, ...).

6. **Сохраняйте публичный API, где это возможно.** Ломающие изменения в `core` или в области `extra` должны быть обоснованы в описании PR и, если изменение нетривиальное, отражены в `CHANGELOG` (если он есть) или в release notes.

7. **Используйте `musutils` для общих хелперов.** Файловые операции, HTTP-загрузки, определение OS/arch, цветной вывод статусов — `rmlib` использует для этого `musutils` (модули v2). Не реализуйте их локально повторно. Если хелпера реально не хватает в `musutils` — предложите сначала добавить его туда.

8. **В `core::run` остаются параметры `ely: bool` / `betafix: bool`.** Это флаги JVM-аргументов (`-javaagent:authlib-injector.jar=ely.by`, `-Dhttp.proxyHost=...`), а не зависимость от какой-либо фичи `extra` на этапе компиляции. Не переносите обработку этих флагов из `core::run`.

9. **`available_modules()` должна отражать реальную сборку.** `rmlib::available_modules()` возвращает верхнеуровневые модули (`core` и каждую включённую область `extra`), вошедшие в текущую сборку через `#[cfg(feature = "...")]`. При добавлении новой области `extra` добавьте соответствующую `#[cfg]`-ветку туда же. Не хардкодьте и не фильтруйте этот список в зависимых крейтах.

10. **Никаких лишних зависимостей.** Предпочитайте `std`; добавляйте crates только при реальной пользе. Новые крейты должны быть уже нужны конкретной фиче.

11. **Тесты обязательны для любого изменения.** Это не пожелание:
    - **Новый публичный элемент** (функция, метод, тип, вариант enum'а, поле, макрос, FFI-экспорт) — добавьте тесты, покрывающие нормальное поведение, граничные значения и ошибки/крайние случаи, если они есть.
    - **Изменение поведения или сигнатуры** — обновите существующие тесты, проверяющие старое поведение, и добавьте новые под новое.
    - **Исправление бага** — добавьте регрессионный тест, который падает без исправления и проходит с ним.
    - **Новый модуль, область `extra` или фича** — отдельный файл в `tests/` с `#[cfg(feature = "...")]` в начале, по образцу существующих.

    Тесты, проверяющие публичный API, размещайте в `tests/` (интеграционные). Встроенный `#[cfg(test)] mod tests` внутри исходника — только когда тестируемый код приватный (`pub(crate)` / private) и недоступен из интеграционного теста.

    Тесты должны быть детерминированными, независимыми, безопасными для файловой системы (используйте `tests/common::TempDir`, никаких реальных пользовательских путей) и устойчивыми к параллельному запуску. Тесты, трогающие `HOME` или другие переменные окружения, должны защищаться через `tests/common::HomeGuard` / `tests/common::env_lock()`. Не хардкодьте сетевые вызовы — для частей, которым нужна сеть (получение версий, списки модлоадеров, auth и т. п.), интеграционные тесты не пишите; покрывайте окружающую чистую логику.

12. **Тесты нужно реально прогонять.** Перед пушем запустите `make test` и `make test-ffi`. Оба должны пройти. PR, добавляющий функциональность с тестами, которые не были прогнаны локально, считается незавершённым.

13. **Любое публичное изменение отражается в документации.** Это тоже обязательное требование:
    - **Новый или изменённый публичный элемент** (функция, тип, модуль, FFI-экспорт) — обновите соответствующую документацию: doc-комментарии на самом элементе (`///`, `//!`) и, если у проекта есть пользовательская документация, соответствующую страницу в `docs/`.
    - **Изменения в фичах, их зависимостях или установке** — обновите документацию в `Cargo.toml`, этот файл и список фич в README (если он есть).
    - **Ломающие изменения** — вынесите их в описание PR и в `CHANGELOG` / release notes.
    - **FFI-экспорты** — держите `rmlib.h` (если он есть) в соответствии с реальными символами. Добавление или удаление функции `rmlib_*` требует соответствующей правки в заголовке.

14. **FFI (C ABI).** Крейт собирается не только как `rlib`, но и как `cdylib`. Публичный C ABI живёт в `src/ffi/` под фичей `ffi` и экспортируется через `#[unsafe(no_mangle)] pub unsafe extern "C" fn rmlib_*`. Правила:
    - Каждый новый FFI-экспорт должен иметь тест в `tests/ffi.rs`.
    - FFI-экспорты не должны паниковать через границу. Используйте `catch_unwind`; возвращайте `NULL` для указателей или `-1` для целых при ошибке; кладите сообщение через `set_last_error`, чтобы вызывающая сторона могла забрать его через `rmlib_last_error()`.
    - Строки, возвращаемые наружу, должны освобождаться через `rmlib_string_free`.
    - Не меняйте сигнатуру существующего экспорта без бампа версии и документирования изменения.

15. **Перед пушем прогоняйте проверки:**
    ```bash
    make fmt-check
    make clippy
    make clippy-ffi
    make test
    make test-ffi
    ```

    Либо одной командой:

    ```bash
    make check
    ```

    `make check` прогоняет `test`, `test-ffi`, `fmt-check`, `clippy` и `clippy-ffi`. Всё должно быть зелёным.

16. **Артефакты shared-библиотеки.** Чтобы получить `librmlib.so` (Linux) / `librmlib.dylib` (macOS) / `rmlib.dll` (Windows):

    ```bash
    make build-ffi      # → target/release/librmlib.so (или платформенный эквивалент)
    make dist           # → то же + копия артефактов в dist/
    ```

    Не коммитьте артефакты из `target/` или `dist/` — оба каталога в `.gitignore`.

17. Отправляя Pull Request, вы соглашаетесь, что ваш код будет распространяться под лицензией **LGPL-3.0**.

18. Отправляя Pull Request, вы соглашаетесь с условиями **политики использования товарных знаков** (см. `TRADEMARK.md`).

---

Вопросы и предложения — в Issues.