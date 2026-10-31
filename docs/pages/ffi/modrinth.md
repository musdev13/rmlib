# FFI — Modrinth

Modrinth API client exports.

These exports mirror [`extra::modrinth`](../extra/modrinth.md) — see that page for the workflow, filter model, and installation rules.

All exports on this page require the `modrinth` feature **and** the `ffi` feature.

```c
#include "rmlib.h"
```

General conventions:

- Functions returning `char *` return a NUL-terminated UTF-8 string, usually JSON. Free it with `rmlib_string_free`.
- Functions returning `int32_t` return `0` on success and `-1` on error.
- On error the returning pointer is `NULL` (or the code is `-1`) and the message is available via `rmlib_last_error()`.
- Input strings must be NUL-terminated UTF-8.

## Search

```c
char *rmlib_modrinth_search(const char *params_json);
```

Runs a project search. `params_json` is a `SearchParams` object:

```json
{
  "query": "sodium",
  "facets": "[[\"project_type:mod\"],[\"categories:fabric\"]]",
  "index": "downloads",
  "offset": 0,
  "limit": 20
}
```

All fields are optional. `index` is one of `relevance`, `downloads`, `follows`, `newest`, `updated`.

### Returns

JSON `SearchResponse`: `{ "hits": [...], "offset": 0, "limit": 20, "total_hits": 1234 }`.

### Example (Python)

```python
import ctypes, json

lib.rmlib_modrinth_search.argtypes = [ctypes.c_char_p]
lib.rmlib_modrinth_search.restype = ctypes.c_void_p

params = json.dumps({"query": "sodium", "limit": 5}).encode()
ptr = lib.rmlib_modrinth_search(params)
if not ptr:
    err = lib.rmlib_last_error()
    print("search failed:", ctypes.string_at(err).decode())
    lib.rmlib_string_free(err)
else:
    resp = json.loads(ctypes.string_at(ptr).decode())
    lib.rmlib_string_free(ptr)
    for hit in resp["hits"]:
        print(hit["title"], hit["downloads"])
```

## Project

```c
char *rmlib_modrinth_project(const char *id_or_slug);
char *rmlib_modrinth_project_by_hash(const char *hash);
char *rmlib_modrinth_project_versions(const char *project_id);
char *rmlib_modrinth_dependencies(const char *project_id);
char *rmlib_modrinth_members(const char *project_id);
```

- `rmlib_modrinth_project` — project by slug or ID. Empty input is an error.
- `rmlib_modrinth_project_by_hash` — resolves a SHA-1/SHA-512 file hash to its project.
- `rmlib_modrinth_project_versions` — all versions of a project (unfiltered).
- `rmlib_modrinth_dependencies` — dependencies collected from the project's versions.
- `rmlib_modrinth_members` — team members of the project.

All return JSON and are freed with `rmlib_string_free`. Errors set `rmlib_last_error`.

## Versions

```c
char *rmlib_modrinth_version(const char *version_id);
char *rmlib_modrinth_version_by_hash(const char *hash);
```

`rmlib_modrinth_version` returns a `Version` object; `rmlib_modrinth_version_by_hash` looks up the version that contains a file with the given hash.

## Users and teams

```c
char *rmlib_modrinth_user(const char *id_or_username);
char *rmlib_modrinth_user_projects(const char *user_id);
char *rmlib_modrinth_team_members(const char *team_id);
```

Return JSON (`User`, `Project[]`, `TeamMember[]`). Empty IDs are errors.

## Tag directories

```c
char *rmlib_modrinth_categories(void);
char *rmlib_modrinth_loaders(void);
char *rmlib_modrinth_game_versions(void);
```

Return JSON arrays of `Category`, `Loader`, `GameVersion`.

## Installation

```c
char *rmlib_modrinth_install(const char *id_or_slug, const char *params_json);
char *rmlib_modrinth_install_version(const char *version_id, const char *params_json);
char *rmlib_modrinth_install_from_hash(const char *hash, const char *params_json);
char *rmlib_modrinth_install_datapack(
    const char *id_or_slug,
    const char *world_name,
    const char *game_path,
    const char *params_json
);
```

`params_json` is an `InstallParams` object:

```json
{
  "game_path": "./game",
  "install_dir": null,
  "minecraft_version": "1.21.1",
  "loader": "fabric",
  "version_type": "release",
  "install_required_dependencies": true,
  "overwrite": false
}
```

`version_type` is `release`, `beta` or `alpha`. `game_path` / `install_dir` are paths; `install_dir` wins when set.

### Returns

JSON `InstallResult`: `{ "source": "project", "project_type": "mod", "installed": [ … ] }`, where each installed file has `path`, `filename`, `bytes`, `sha1`, `sha512`, `project_id`, `version_id`.

### Errors

`NULL` + `rmlib_last_error` on:

- invalid JSON in `params_json`,
- empty identifiers,
- no compatible version,
- content type that needs an explicit `install_dir` (`modpack`, `world`),
- hash mismatch,
- filesystem errors,
- API / network errors.

`rmlib_modrinth_install_datapack` additionally requires `game_path` to contain `saves/<world_name>/`; the world directory is not created.

### Example (C)

```c
char *result = rmlib_modrinth_install("sodium", "{\"game_path\":\"./game\",\"minecraft_version\":\"1.21.1\",\"loader\":\"fabric\"}");
if (!result) {
    char *err = rmlib_last_error();
    fprintf(stderr, "install failed: %s\n", err ? err : "(no message)");
    rmlib_string_free(err);
} else {
    printf("%s\n", result);
    rmlib_string_free(result);
}
```

## See also

- [Errors and Strings](error.md) — reading `rmlib_last_error` and freeing strings
- [Modrinth](../extra/modrinth.md) — the Rust-side API