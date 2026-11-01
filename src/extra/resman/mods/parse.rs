use std::io::Read;
use std::path::Path;

use serde_json::Value;

use super::dto::{ModDependency, ModInfo, ModLoader};
use super::error::ModError;
use super::icon::save_icon;

const BUILTIN_FABRIC_DEPS: [&str; 4] = ["minecraft", "fabricloader", "fabric-loader", "java"];
const BUILTIN_NEOFORGE_DEPS: [&str; 7] = [
    "neoforge",
    "forge",
    "minecraft",
    "javafml",
    "lowcodefml",
    "fml",
    "mcp",
];

pub(crate) fn read_file_from_zip<R: Read + std::io::Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
) -> Option<Vec<u8>> {
    let mut f = zip.by_name(name).ok()?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).ok()?;
    Some(buf)
}

pub(crate) fn analyze_jar(path: &Path) -> Result<ModInfo, ModError> {
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)?;
    let names: Vec<String> = zip
        .file_names()
        .filter_map(|r| r.ok().map(|s| s.into_owned()))
        .collect();
    let has = |n: &str| names.iter().any(|x| x == n);

    let filename = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let enabled = !filename.ends_with(".disabled");
    let stem = filename
        .strip_suffix(".disabled")
        .unwrap_or(&filename)
        .to_string();

    let mut info = ModInfo {
        path: path.to_path_buf(),
        filename: stem.clone(),
        enabled,
        size,
        ..Default::default()
    };

    let neoforge_sentinel = ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"]
        .into_iter()
        .find(|n| has(n));
    let has_fabric = has("fabric.mod.json");

    // Detect a decoy fabric.mod.json (JourneyMap-style) up front so a real
    // NeoForge mod is not reported as "universal" by mistake.
    let fabric_is_decoy = if has_fabric {
        let bytes = read_file_from_zip(&mut zip, "fabric.mod.json")
            .ok_or_else(|| ModError::NotFound("fabric.mod.json".to_string()))?;
        let fmj: Value = serde_json::from_slice(&bytes)?;
        is_fabric_decoy(&fmj)
    } else {
        false
    };

    let fabric_valid = has_fabric && !fabric_is_decoy;

    if let Some(sentinel) = neoforge_sentinel {
        let bytes = read_file_from_zip(&mut zip, sentinel)
            .ok_or_else(|| ModError::NotFound(sentinel.to_string()))?;
        let text = String::from_utf8_lossy(&bytes);
        let toml: toml::Value = toml::from_str(&text)?;
        fill_neoforge(&mut info, &toml, &mut zip, path)?;
        if fabric_valid {
            info.loader = Some(ModLoader::Universal);
        }
        return Ok(info);
    }

    if fabric_valid {
        let bytes = read_file_from_zip(&mut zip, "fabric.mod.json")
            .ok_or_else(|| ModError::NotFound("fabric.mod.json".to_string()))?;
        let fmj: Value = serde_json::from_slice(&bytes)?;
        fill_fabric(&mut info, &fmj, &mut zip, path)?;
        return Ok(info);
    }

    if has_fabric && fabric_is_decoy {
        return Err(ModError::InvalidArgument(
            "fabric.mod.json is a decoy (wrong-loader stub)".to_string(),
        ));
    }

    Err(ModError::InvalidArgument(
        "no fabric.mod.json or NeoForge mods.toml in JAR".to_string(),
    ))
}

fn is_fabric_decoy(fmj: &Value) -> bool {
    if fmj
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.ends_with("-wrongloader"))
        .unwrap_or(false)
    {
        return true;
    }
    if fmj
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case("not a fabric mod"))
        .unwrap_or(false)
    {
        return true;
    }
    if let Some(eps) = fmj.get("entrypoints").and_then(|v| v.as_object()) {
        for (_, arr) in eps {
            if let Some(arr) = arr.as_array() {
                for ep in arr {
                    if let Some(s) = ep.as_str()
                        && s.ends_with("Error")
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn fill_fabric<R: Read + std::io::Seek>(
    info: &mut ModInfo,
    fmj: &Value,
    zip: &mut zip::ZipArchive<R>,
    _path: &Path,
) -> Result<(), ModError> {
    info.loader = Some(ModLoader::Fabric);
    info.id = fmj.get("id").and_then(|v| v.as_str()).map(str::to_string);
    info.name = fmj.get("name").and_then(|v| v.as_str()).map(str::to_string);
    info.version = fmj
        .get("version")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    info.description = fmj
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    info.license = fmj
        .get("license")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    if let Some(authors) = fmj.get("authors") {
        let mut list = Vec::new();
        if let Some(arr) = authors.as_array() {
            for a in arr {
                if let Some(s) = a.as_str() {
                    list.push(s.to_string());
                } else if let Some(name) = a.get("name").and_then(|n| n.as_str()) {
                    list.push(name.to_string());
                }
            }
        }
        info.authors = Some(list);
    }

    if let Some(mc) = fmj.get("depends").and_then(|d| d.get("minecraft")) {
        let mut vs = Vec::new();
        if let Some(s) = mc.as_str() {
            vs.push(s.to_string());
        } else if let Some(arr) = mc.as_array() {
            for v in arr {
                if let Some(s) = v.as_str() {
                    vs.push(s.to_string());
                }
            }
        }
        if !vs.is_empty() {
            info.minecraft = Some(vs);
        }
    }

    if let Some(dep) = fmj.get("depends").and_then(|d| d.as_object()) {
        for (k, v) in dep {
            if BUILTIN_FABRIC_DEPS.contains(&k.as_str()) {
                continue;
            }
            let range = match v {
                Value::String(s) => Some(s.clone()),
                Value::Array(arr) => Some(
                    arr.iter()
                        .filter_map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                _ => None,
            };
            info.dependencies.push(ModDependency {
                id: k.clone(),
                kind: "required".to_string(),
                version_range: range,
                fabric_api: k.starts_with("fabric-"),
            });
        }
    }
    if let Some(dep) = fmj.get("recommends").and_then(|d| d.as_object()) {
        for (k, v) in dep {
            if BUILTIN_FABRIC_DEPS.contains(&k.as_str()) {
                continue;
            }
            let range = match v {
                Value::String(s) => Some(s.clone()),
                Value::Array(arr) => Some(
                    arr.iter()
                        .filter_map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                _ => None,
            };
            info.dependencies.push(ModDependency {
                id: k.clone(),
                kind: "optional".to_string(),
                version_range: range,
                fabric_api: k.starts_with("fabric-"),
            });
        }
    }

    if let Some(icon) = fmj.get("icon") {
        let icon_path = match icon {
            Value::String(s) => Some(s.clone()),
            Value::Object(map) => map
                .values()
                .next()
                .and_then(|v| v.as_str())
                .map(String::from),
            _ => None,
        };
        if let Some(p) = icon_path
            && let Some(bytes) = read_file_from_zip(zip, &p)
        {
            let key = info.id.clone().unwrap_or_else(|| info.filename.clone());
            info.icon = Some(save_icon(&key, &bytes)?);
        }
    }

    if info.minecraft.is_none() {
        info.minecraft = infer_mc(info.version.as_ref(), &info.filename);
    }
    Ok(())
}

fn fill_neoforge<R: Read + std::io::Seek>(
    info: &mut ModInfo,
    toml: &toml::Value,
    zip: &mut zip::ZipArchive<R>,
    _path: &Path,
) -> Result<(), ModError> {
    info.loader = Some(ModLoader::NeoForge);
    info.license = toml
        .get("license")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let mods = toml
        .get("mods")
        .and_then(|m| m.as_array())
        .ok_or_else(|| ModError::InvalidArgument("mods.toml has no [[mods]]".to_string()))?;
    let first = mods
        .first()
        .and_then(|m| m.as_table())
        .ok_or_else(|| ModError::InvalidArgument("invalid [[mods]] entry".to_string()))?;

    let get = |k: &str| first.get(k).and_then(|v| v.as_str()).map(str::to_string);
    info.id = get("modId");
    info.name = get("displayName");
    info.version = get("version");
    info.description = get("description");
    let authors = get("authors");
    if let Some(a) = authors {
        let list: Vec<String> = a.split(',').map(|s| s.trim().to_string()).collect();
        info.authors = Some(list);
    }

    let mod_id = info.id.clone().unwrap_or_default();
    if let Some(deps) = toml
        .get("dependencies")
        .and_then(|d| d.get(&mod_id))
        .and_then(|d| d.as_array())
    {
        for dep in deps {
            let Some(table) = dep.as_table() else {
                continue;
            };
            let id = table
                .get("modId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if id.is_empty() {
                continue;
            }
            let kind = table
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("required")
                .to_string();
            let range = table
                .get("versionRange")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            if id == "minecraft" {
                if let Some(r) = range {
                    info.minecraft = Some(vec![r]);
                }
                continue;
            }
            if BUILTIN_NEOFORGE_DEPS.contains(&id.as_str()) {
                continue;
            }
            info.dependencies.push(ModDependency {
                id,
                kind,
                version_range: range,
                fabric_api: false,
            });
        }
    }

    let icon_name = first
        .get("logoFile")
        .and_then(|v| v.as_str())
        .or_else(|| first.get("iconFile").and_then(|v| v.as_str()))
        .or_else(|| first.get("bannerFile").and_then(|v| v.as_str()));
    if let Some(p) = icon_name
        && let Some(bytes) = read_file_from_zip(zip, p)
    {
        let key = info.id.clone().unwrap_or_else(|| info.filename.clone());
        info.icon = Some(save_icon(&key, &bytes)?);
    }

    if info.minecraft.is_none() {
        info.minecraft = infer_mc(info.version.as_ref(), &info.filename);
    }
    Ok(())
}

pub(crate) fn infer_mc(version: Option<&String>, filename: &str) -> Option<Vec<String>> {
    let mut candidates: Vec<&str> = Vec::new();
    if let Some(v) = version {
        candidates.push(v.as_str());
    }
    candidates.push(filename);

    let mut mark = None;
    let mut lead = None;
    for c in &candidates {
        if mark.is_none() {
            mark = extract_mc_marker(c);
        }
        if lead.is_none() {
            lead = extract_leading(c);
        }
    }
    mark.or(lead).map(|s| vec![s])
}

fn extract_mc_marker(s: &str) -> Option<String> {
    let idx = s.find("+mc")?;
    let rest = &s[idx + 3..];
    let v: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if v.is_empty() { None } else { Some(v) }
}

fn extract_leading(s: &str) -> Option<String> {
    let v: String = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if v.is_empty() || !v.contains('.') {
        return None;
    }
    Some(v)
}
