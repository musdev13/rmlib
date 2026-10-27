pub fn build_maven_path(name: &str, classifier: Option<&str>) -> Option<String> {
    let name = name.strip_suffix("@jar").unwrap_or(name);
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return None;
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];

    let cls = classifier
        .map(|c| c.to_string())
        .or_else(|| parts.get(3).map(|c| c.to_string()));

    Some(match cls {
        Some(c) => format!(
            "{}/{}/{}/{}-{}-{}.jar",
            group, artifact, version, artifact, version, c
        ),
        None => format!(
            "{}/{}/{}/{}-{}.jar",
            group, artifact, version, artifact, version
        ),
    })
}
