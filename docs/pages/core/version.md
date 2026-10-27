# core::version

Mojang version manifest: fetching it, fetching per-version JSON, and shaping the manifest entries for display.

The manifest is a JSON document served by Mojang at:

```text
https://launchermeta.mojang.com/mc/game/version_manifest.json
```

Its `versions` array has entries like:

```json
{
  "id": "1.20.1",
  "type": "release",
  "url": "https://piston-meta.mojang.com/v1/packages/.../1.20.1.json",
  "time": "2023-06-12T13:25:51+00:00",
  "releaseTime": "2023-06-12T13:25:51+00:00",
  "sha1": "...",
  "complianceLevel": 1
}
```

The functions in this module operate on `serde_json::Value` entries from that array. They are pure except for `fetch_list`, `get_version_json` and `get_version_json_url`, which hit the network.

## Enable

```toml
[dependencies]
rmlib = { git = "https://github.com/musdev13/rmlib", default-features = false, features = ["core"] }
```

```rust
use rmlib::core::version::{fetch_list, get_version_json, get_version_json_url, sort_by_type, strip_version};
```

## `fetch_list`

```rust
pub async fn fetch_list() -> Vec<Value>
```

Fetches the Mojang version manifest and returns the `versions` array.

- Network error → **panics** with `Status::Err: Failed to fetch manifest string`.
- Invalid JSON from Mojang → **panics** with `Status::Err: Failed to parse manifest XML/JSON`.
- Manifest without a `versions` array → **panics** with `Status::Err: Failed to find 'versions' array in manifest`.

Returns a `Vec<serde_json::Value>` where each entry is one version object. Order is Mojang's order (newest first). Do not rely on the exact order — sort with `sort_by_type` or a custom comparator.

```rust
use rmlib::core::version::fetch_list;

let versions = fetch_list().await;
println!("{} versions", versions.len());
```

This is an `async fn`. Call it from a `tokio` runtime.

## `get_version_json_url`

```rust
pub async fn get_version_json_url(version_id: String) -> String
```

Looks up the URL of a specific version's JSON document by its `id`.

- Fetches the manifest via `fetch_list`.
- Finds the entry whose `id` equals `version_id`.
- Returns that entry's `url` field.

Panics if:

- the manifest cannot be fetched (as `fetch_list`);
- no entry has the requested `id` — `Status::Err: version with id '<id>' not found`;
- the entry has no `url` — `Status::Err: url field is missing in the version element`;
- `url` is not a string — `Status::Err: url is not a valid string`.

```rust
let url = rmlib::core::version::get_version_json_url("1.20.1".into()).await;
println!("{url}");
```

## `get_version_json`

```rust
pub async fn get_version_json(version_id: String) -> String
```

Combines `get_version_json_url` and `get_json_str`: looks up the URL and fetches the JSON, returning it as a pretty-printed string.

Panics on any error (see `get_version_json_url`, plus `Status::Err: failed to get version json string` if the HTTP fetch fails).

```rust
let json = rmlib::core::version::get_version_json("1.20.1".into()).await;
let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
```

The result is the JSON document that `version::install::install` writes to `versions/<id>/<id>.json`.

## `sort_by_type`

```rust
pub fn sort_by_type(versions: &[Value], version_type: VersionType) -> Vec<Value>
```

Filters manifest entries by `type`.

- Keeps entries whose `type` string equals `version_type.as_str()`.
- Drops entries with no `type`, with a `type` that does not match, or that are not JSON objects.
- Preserves the input order of the kept entries.

```rust
use rmlib::core::types::VersionType;
use rmlib::core::version::sort_by_type;

let releases = sort_by_type(&manifest_entries, VersionType::Release);
let snapshots = sort_by_type(&manifest_entries, VersionType::Snapshot);
let betas = sort_by_type(&manifest_entries, VersionType::OldBeta);
let alphas = sort_by_type(&manifest_entries, VersionType::OldAlpha);
```

`sort_by_type` does **not** sort — it only filters. The name is historical.

Edge cases:

- Empty input → empty output.
- Entries with no `type` are dropped (they are not treated as any specific type).
- Entries with an unknown `type` (e.g. `"weird"`) are dropped for all four `VersionType` variants.

## `strip_version`

```rust
pub fn strip_version(
    versions: &[Value],
    keep_id: bool,
    keep_release_time: bool,
    keep_time: bool,
    keep_type: bool,
    keep_url: bool,
) -> Vec<Value>
```

Returns a copy of `versions` where each entry contains **only** the fields selected by the `keep_*` flags.

| Flag | Field kept |
|---|---|
| `keep_id` | `id` |
| `keep_release_time` | `releaseTime` |
| `keep_time` | `time` |
| `keep_type` | `type` |
| `keep_url` | `url` |

Any other field (`sha1`, `complianceLevel`, …) is always dropped.

Output shape:

- One output entry per input entry, in order.
- Each output entry is a JSON object with the selected fields that were present in the input.
- If none of the selected fields are present, the output entry is `{}`.
- If the input entry is not an object (e.g. `"scalar"`, `42`, `null`), it still produces one `{}` in the output (so the length is preserved).

```rust
use rmlib::core::version::strip_version;

// Only the "id" field.
let ids = strip_version(&manifest_entries, true, false, false, false, false);

// Everything except sha1 and complianceLevel.
let full = strip_version(&manifest_entries, true, true, true, true, true);

// Nothing — a vector of empty objects.
let nothing = strip_version(&manifest_entries, false, false, false, false, false);
```

This is useful for CLI output: `--json` mode typically wants a small, stable shape; `--plain` mode may want everything.

## Manifest field assumptions

The functions in this module assume the standard Mojang manifest shape. A minimal entry looks like:

```json
{
  "id": "1.20.1",
  "type": "release",
  "url": "https://...",
  "time": "2023-06-12T13:25:51+00:00",
  "releaseTime": "2023-06-12T13:25:51+00:00"
}
```

If Mojang ever adds fields, `strip_version` will still produce a stable shape (it only ever keeps the five listed fields). If Mojang removes a field, `strip_version` simply won't include it in the output.

## See also

- [Version installation](version-install.md) — `install` and its helper functions
- [Types](types.md) — `VersionType`, `build_maven_path`