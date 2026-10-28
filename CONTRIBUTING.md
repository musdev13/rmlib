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
│   ├── run/
│   ├── types/
│   ├── version/
│   └── test.rs
├── extra/              # feature = "ely" | "fabric" | "neoforge"
│   ├── mod.rs
│   ├── ely/
│   ├── fabric/
│   └── neoforge/
└── ffi/                # feature = "ffi" — C ABI exports (see "FFI" section)
    ├── mod.rs
    ├── error.rs
    ├── exports.rs
    ├── runtime.rs
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

docs/                   # MkDocs site sources (see "Documentation" section)
├── index.md            # site landing page
├── README.md           # GitHub-facing overview (excluded from the site)
├── assets/extra.css
└── pages/
    ├── getting-started/
    ├── lib.md
    ├── core/
    ├── extra/
    └── ffi/

mkdocs.yml              # MkDocs configuration and navigation
Makefile                # make test / lint / check / docs-check / build-ffi / dist
```

`mod.rs` files are only for `mod` declarations and `pub use` re-exports. Do not put the whole implementation of a module into its `mod.rs`.

### Rules

1. **All changes must be submitted to the `dev` branch.** Open a Pull Request against `dev`, not `main`.

2. **Keep the `core` / `extra` split.**
   - Base launcher functionality → `src/core/` (in the appropriate submodule: `config/`, `run/`, `types/`, `version/`).
   - Additional/optional functionality → `src/extra/<area>/` (e.g. `extra/ely/`, `extra/fabric/`, `extra/neoforge/`, and future areas like `extra/mods/`, `extra/servers/`, `extra/profiles/`).

3. **Every `extra` area is its own Cargo feature.** Create `<area> = ["core"]` in `Cargo.toml`, gate the submodule with `#[cfg(feature = "<area>")] pub mod <area>;` in `src/extra/mod.rs`, and gate the whole `pub mod extra;` with `#[cfg(any(feature = "ely", feature = "fabric", feature = "neoforge", ...))]` in `src/lib.rs`. The `extra` crate module must not be enabled if no area feature is on.

4. **Do not make `core` depend on `extra`.** `core` must compile and work with `--no-default-features --features core`. If a piece of `core` needs optional add-on functionality, either move that piece to `extra/<area>/`, or gate it with `#[cfg(feature = "<area>")]` and document the contract.

5. **One file per logical operation.** Do not put multiple unrelated functions into one file. Follow the existing layout.

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
    - **New or changed public item** (function, type, module, FFI export) — update:
      - the corresponding page under `docs/pages/` — see the [Documentation](#documentation) section below for which page covers what.
    - **Changes to features, feature dependencies, or setup** — update `Cargo.toml`, this file, `README.md`, and `docs/pages/getting-started/installation.md` / `features.md`.
    - **Breaking changes** — call them out in the PR description and in `CHANGELOG` / release notes.
    - **FFI exports** — keep `rmlib.h` (if present) in sync with the exported symbols. Adding or removing an `rmlib_*` function requires a matching change in the header **and** in `docs/pages/ffi/`.

    Where to put documentation for a change:

    | Change | Docs page |
    |---|---|
    | Crate-root items (`VERSION`, `NAME`, `available_modules`, …) | `docs/pages/lib.md` |
    | `core::config` | `docs/pages/core/config.md` |
    | `core::types` | `docs/pages/core/types.md` |
    | `core::version` | `docs/pages/core/version.md` |
    | `core::version::install` | `docs/pages/core/version-install.md` |
    | `core::run` (paths, environment) | `docs/pages/core/run/paths.md` |
    | `core::run` (classpath) | `docs/pages/core/run/classpath.md` |
    | `core::run` (arguments, rules) | `docs/pages/core/run/arguments.md` |
    | `core::run` (command, launch) | `docs/pages/core/run/command.md` |
    | `core::run::dto` | `docs/pages/core/run/dto.md` |
    | `extra::ely` | `docs/pages/extra/ely.md` |
    | `extra::fabric` | `docs/pages/extra/fabric.md` |
    | `extra::neoforge` | `docs/pages/extra/neoforge.md` |
    | FFI overview / conventions | `docs/pages/ffi/index.md` |
    | FFI exports by area | `docs/pages/ffi/{lib,error,config,version,install,run,ely}.md` |

    A new `extra` area also needs: a new page under `docs/pages/extra/`, a new entry in the table in `docs/pages/extra/` (see `index`-like grouping on the relevant page), the new feature listed in `docs/pages/getting-started/features.md`, and a nav entry in `mkdocs.yml`.

14. **FFI (C ABI).** The crate builds a `cdylib` in addition to the usual `rlib`. Public C ABI lives under `src/ffi/` behind the `ffi` feature and is exposed with `#[unsafe(no_mangle)] pub unsafe extern "C" fn rmlib_*`. Rules:
    - Every new FFI export must have a corresponding test in `tests/ffi.rs`.
    - FFI exports must not panic across the boundary. Use `catch_unwind`; return `NULL` for pointers or `-1` for integers on error; store the message via `set_last_error` so callers can retrieve it with `rmlib_last_error()`.
    - Strings returned to the caller must be freed with `rmlib_string_free`.
    - Do not change an existing export's signature without bumping the version and documenting the change.
    - Every new export must be documented on the matching page under `docs/pages/ffi/`, with the C signature, parameters, return value, error conditions, and (where useful) a short Python / C example.

### Documentation

The user-facing documentation is a MkDocs site (Material theme) sourced from `docs/` and configured by `mkdocs.yml`. It is part of the repository — treat it as code.

- **`docs/index.md`** — the site landing page.
- **`docs/README.md`** — the GitHub-facing overview. It is excluded from the built site via `exclude_docs` in `mkdocs.yml`.
- **`docs/pages/`** — one page per module / area / generation, matching the crate layout.

Navigation is defined in the `nav:` section of `mkdocs.yml`. When you add a page, add it to `nav:` as well — otherwise it exists in `docs/` but is unreachable from the site.

`rmlib.h` and `docs/pages/ffi/` must stay in sync with `src/ffi/exports.rs`. The exports are the source of truth; the docs follow.

### Running the checks

Before pushing, run:

```bash
make fmt-check
make clippy
make clippy-ffi
make test
make test-ffi
make docs-check
```

Or, in a single step:

```bash
make check
```

`make check` runs `test`, `test-ffi`, `fmt-check`, `clippy`, `clippy-ffi`, and `docs-check`. All must be green.

`make docs-check` runs `mkdocs build --strict`. Strict mode turns broken links, missing nav targets, and any other warnings into errors — do not push a documentation change that does not pass `make docs-check`.

To preview the docs locally while editing:

```bash
make docs-serve
```

### Building the documentation

The MkDocs binary is expected in `venv/bin/mkdocs` by default. If you have MkDocs installed globally (or under a different path), override it:

```bash
make docs MKDOCS=mkdocs
make docs-check MKDOCS=mkdocs
```

The dependency list for MkDocs is in `requirements-docs.txt` (or `req.txt` — whichever the project ships). Install it with:

```bash
python3 -m venv venv
venv/bin/pip install -r req.txt
```

`make docs` builds the site into `site/`. `site/` and `venv/` are ignored by `.gitignore` — do not commit them.

### Shared library artifacts

To produce `librmlib.so` (Linux) / `librmlib.dylib` (macOS) / `rmlib.dll` (Windows), run:

```bash
make build-ffi      # → target/release/librmlib.so (or platform equivalent)
make dist           # → same, plus copies artifacts into dist/
```

Do not commit artifacts from `target/`, `dist/`, `site/`, or `venv/` — all of them are ignored by `.gitignore`.

### License

By submitting a Pull Request, you agree that your code will be distributed under the **LGPL-3.0** license

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
│   ├── run/
│   ├── types/
│   ├── version/
│   └── test.rs
├── extra/              # фича = "ely" | "fabric" | "neoforge"
│   ├── mod.rs
│   ├── ely/
│   ├── fabric/
│   └── neoforge/
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

docs/                   # исходники сайта MkDocs (см. раздел «Документация»)
├── index.md            # главная страница сайта
├── README.md           # страница для GitHub (исключена из сайта)
├── assets/extra.css
└── pages/
    ├── getting-started/
    ├── lib.md
    ├── core/
    ├── extra/
    └── ffi/

mkdocs.yml              # конфигурация MkDocs и навигация
Makefile                # make test / lint / check / docs-check / build-ffi / dist
```

`mod.rs` — только для объявлений `mod` и реэкспорта через `pub use`. Не складывайте всю реализацию модуля в его `mod.rs`.

### Правила

1. **Все изменения отправляются в ветку `dev`.** Pull Request открывается против `dev`, а не `main`.

2. **Сохраняйте разделение `core` / `extra`.**
   - Базовая функциональность → `src/core/` (в соответствующий подмодуль: `config/`, `run/`, `types/`, `version/`).
   - Дополнительная/опциональная функциональность → `src/extra/<area>/` (например, `extra/ely/`, `extra/fabric/`, `extra/neoforge/` и будущие области вроде `extra/mods/`, `extra/servers/`, `extra/profiles/`).

3. **Каждая область в `extra` — отдельная Cargo-фича.** Создайте `<area> = ["core"]` в `Cargo.toml`, оберните подмодуль `#[cfg(feature = "<area>")] pub mod <area>;` в `src/extra/mod.rs`, а весь `pub mod extra;` оберните `#[cfg(any(feature = "ely", feature = "fabric", feature = "neoforge", ...))]` в `src/lib.rs`. Модуль `extra` не должен существовать, если не включена ни одна фича-область.

4. **`core` не должен зависеть от `extra`.** `core` обязан собираться и работать с `--no-default-features --features core`. Если какой-то части `core` нужна опциональная функциональность из `extra`, — либо перенесите эту часть в `extra/<area>/`, либо оберните `#[cfg(feature = "<area>")]` и задокументируйте контракт.

5. **Один файл — одна логическая операция.** Не складывайте несколько несвязанных функций в один файл. Следуйте существующей раскладке.

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
    - **Новый или изменённый публичный элемент** (функция, тип, модуль, FFI-экспорт) — обновите соответствующую страницу в `docs/pages/` — см. таблицу ниже.
    - **Изменения в фичах, их зависимостях или установке** — обновите `Cargo.toml`, этот файл, `README.md` и `docs/pages/getting-started/installation.md` / `features.md`.
    - **Ломающие изменения** — вынесите их в описание PR и в `CHANGELOG` / release notes.
    - **FFI-экспорты** — держите `rmlib.h` (если он есть) в соответствии с реальными символами. Добавление или удаление функции `rmlib_*` требует правки в заголовке **и** в `docs/pages/ffi/`.

    Куда писать документацию для изменения:

    | Изменение | Страница документации |
    |---|---|
    | Элементы на корне крейта (`VERSION`, `NAME`, `available_modules`, …) | `docs/pages/lib.md` |
    | `core::config` | `docs/pages/core/config.md` |
    | `core::types` | `docs/pages/core/types.md` |
    | `core::version` | `docs/pages/core/version.md` |
    | `core::version::install` | `docs/pages/core/version-install.md` |
    | `core::run` (пути, окружение) | `docs/pages/core/run/paths.md` |
    | `core::run` (classpath) | `docs/pages/core/run/classpath.md` |
    | `core::run` (аргументы, rules) | `docs/pages/core/run/arguments.md` |
    | `core::run` (команда, запуск) | `docs/pages/core/run/command.md` |
    | `core::run::dto` | `docs/pages/core/run/dto.md` |
    | `extra::ely` | `docs/pages/extra/ely.md` |
    | `extra::fabric` | `docs/pages/extra/fabric.md` |
    | `extra::neoforge` | `docs/pages/extra/neoforge.md` |
    | Обзор FFI / конвенции | `docs/pages/ffi/index.md` |
    | FFI-экспорты по областям | `docs/pages/ffi/{lib,error,config,version,install,run,ely}.md` |

    Новая область `extra` также требует: новую страницу в `docs/pages/extra/`, запись в списке областей на соответствующих страницах, новую фичу в `docs/pages/getting-started/features.md` и запись в `nav:` файла `mkdocs.yml`.

14. **FFI (C ABI).** Крейт собирается не только как `rlib`, но и как `cdylib`. Публичный C ABI живёт в `src/ffi/` под фичей `ffi` и экспортируется через `#[unsafe(no_mangle)] pub unsafe extern "C" fn rmlib_*`. Правила:
    - Каждый новый FFI-экспорт должен иметь тест в `tests/ffi.rs`.
    - FFI-экспорты не должны паниковать через границу. Используйте `catch_unwind`; возвращайте `NULL` для указателей или `-1` для целых при ошибке; кладите сообщение через `set_last_error`, чтобы вызывающая сторона могла забрать его через `rmlib_last_error()`.
    - Строки, возвращаемые наружу, должны освобождаться через `rmlib_string_free`.
    - Не меняйте сигнатуру существующего экспорта без бампа версии и документирования изменения.
    - Каждый новый экспорт должен быть описан на соответствующей странице в `docs/pages/ffi/`: C-сигнатура, параметры, возврат, условия ошибок и, где полезно, короткий пример на Python / C.

### Документация

Пользовательская документация — это сайт на MkDocs (тема Material), исходники которого лежат в `docs/`, а конфигурация — в `mkdocs.yml`. Документация — часть репозитория, относитесь к ней как к коду.

- **`docs/index.md`** — главная страница сайта.
- **`docs/README.md`** — страница для GitHub. Она исключена из собранного сайта через `exclude_docs` в `mkdocs.yml`.
- **`docs/pages/`** — по одной странице на модуль / область / поколение, в соответствии со структурой крейта.

Навигация задаётся в секции `nav:` файла `mkdocs.yml`. Добавляя страницу, добавьте её и в `nav:` — иначе файл существует в `docs/`, но недостижим с сайта.

`rmlib.h` и `docs/pages/ffi/` должны оставаться в соответствии с `src/ffi/exports.rs`. Источник истины — экспорты, документация следует за ними.

### Прогон проверок

Перед пушем запустите:

```bash
make fmt-check
make clippy
make clippy-ffi
make test
make test-ffi
make docs-check
```

Либо одной командой:

```bash
make check
```

`make check` прогоняет `test`, `test-ffi`, `fmt-check`, `clippy`, `clippy-ffi` и `docs-check`. Всё должно быть зелёным.

`make docs-check` запускает `mkdocs build --strict`. В строгом режиме битые ссылки, отсутствующие цели навигации и любые другие предупреждения становятся ошибками — не пушьте изменение документации, если `make docs-check` не проходит.

Чтобы посмотреть документацию локально во время правок:

```bash
make docs-serve
```

### Сборка документации

По умолчанию ожидается, что бинарь MkDocs лежит в `venv/bin/mkdocs`. Если MkDocs установлен глобально (или в другом месте), переопределите путь:

```bash
make docs MKDOCS=mkdocs
make docs-check MKDOCS=mkdocs
```

Список зависимостей для MkDocs лежит в `requirements-docs.txt` (или `req.txt` — как в проекте). Установка:

```bash
python3 -m venv venv
venv/bin/pip install -r req.txt
```

`make docs` собирает сайт в `site/`. `site/` и `venv/` в `.gitignore` — не коммитьте их.

### Артефакты shared-библиотеки

Чтобы получить `librmlib.so` (Linux) / `librmlib.dylib` (macOS) / `rmlib.dll` (Windows):

```bash
make build-ffi      # → target/release/librmlib.so (или платформенный эквивалент)
make dist           # → то же + копия артефактов в dist/
```

Не коммитьте артефакты из `target/`, `dist/`, `site/` или `venv/` — все они в `.gitignore`.

### Лицензия

Отправляя Pull Request, вы соглашаетесь, что ваш код будет распространяться под лицензией **LGPL-3.0**

---

Вопросы и предложения — в Issues.