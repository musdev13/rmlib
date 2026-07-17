# Contributing to rml / Вклад в rml

---

## EN

Thanks for your interest in the project!

### Structure

- `src/core/` — **core launcher functionality**: game launch, version fetching from official sources, authentication, etc.
- `src/extra/` — **additional features**: mod managers, server managers, custom profiles, and other nice-to-have launcher extensions.

### Rules

1. Keep the existing modular structure.
2. New **core** features → `src/core/` (appropriate submodule: `config/`, `types/`, `version/`, etc.).
3. New **optional/extended** features → `src/extra/` (create a new module if needed, e.g., `extra/mods/`, `extra/servers/`).
4. By submitting a Pull Request, you agree that your code will be distributed under the **LGPL-3.0** license.
5. By submitting a Pull Request, you agree to the terms of the **Trademark Policy** (see `TRADEMARK.md`).

---

Questions and suggestions — in Issues.

---

## RU

Спасибо за интерес к проекту!

### Структура

- `src/core/` — **базовая функциональность лаунчера**: запуск игры, получение версий с официальных источников, авторизация и т.д.
- `src/extra/` — **дополнительные возможности**: менеджеры модов, менеджеры серверов, кастомные профили и другие приятные расширения для лаунчера.

### Правила

1. Сохраняйте существующую модульную структуру.
2. Новые **базовые** функции → `src/core/` (в соответствующий подмодуль: `config/`, `types/`, `version/` и т.д.).
3. Новые **дополнительные/расширенные** функции → `src/extra/` (при необходимости создавайте новый модуль, например `extra/mods/`, `extra/servers/`).
4. Отправляя Pull Request, вы соглашаетесь, что ваш код будет распространяться под лицензией **LGPL-3.0**.
5. Отправляя Pull Request, вы соглашаетесь с условиями **политики использования товарных знаков** (см. `TRADEMARK.md`).

---

Вопросы и предложения — в Issues.
