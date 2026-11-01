use serde::{Deserialize, Serialize};

use super::error::ModError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Local,
    Modrinth,
}

impl Source {
    pub fn as_str(&self) -> &'static str {
        match self {
            Source::Local => "local",
            Source::Modrinth => "modrinth",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Priority {
    sources: Vec<Source>,
}

impl Default for Priority {
    fn default() -> Self {
        Self {
            sources: vec![Source::Local],
        }
    }
}

impl Priority {
    pub fn parse(s: &str) -> Result<Self, ModError> {
        let mut sources = Vec::new();
        for part in s.split(',') {
            let p = part.trim();
            if p.is_empty() {
                continue;
            }
            let src = match p {
                "local" => Source::Local,
                "modrinth" => {
                    if !cfg!(feature = "modrinth") {
                        return Err(ModError::InvalidArgument(
                            "priority source \"modrinth\" requires the rmlib `modrinth` feature"
                                .to_string(),
                        ));
                    }
                    Source::Modrinth
                }
                other => {
                    return Err(ModError::InvalidArgument(format!(
                        "unknown priority source {other:?}"
                    )));
                }
            };
            if !sources.contains(&src) {
                sources.push(src);
            }
        }
        if sources.is_empty() {
            return Err(ModError::InvalidArgument(
                "priority must contain at least one source".to_string(),
            ));
        }
        Ok(Self { sources })
    }

    pub fn sources(&self) -> &[Source] {
        &self.sources
    }

    pub fn includes(&self, s: Source) -> bool {
        self.sources.contains(&s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    Id,
    Name,
    Version,
    Description,
    Authors,
    License,
    Minecraft,
    Dependencies,
    Icon,
    Path,
    Enabled,
    Loader,
    Filename,
    Size,
    FabricApi,
}

impl Field {
    pub fn parse(s: &str) -> Result<Self, ModError> {
        match s.trim() {
            "id" => Ok(Field::Id),
            "name" => Ok(Field::Name),
            "version" => Ok(Field::Version),
            "description" => Ok(Field::Description),
            "authors" => Ok(Field::Authors),
            "license" => Ok(Field::License),
            "minecraft" => Ok(Field::Minecraft),
            "dependencies" => Ok(Field::Dependencies),
            "icon" => Ok(Field::Icon),
            "path" => Ok(Field::Path),
            "enabled" => Ok(Field::Enabled),
            "loader" => Ok(Field::Loader),
            "filename" => Ok(Field::Filename),
            "size" => Ok(Field::Size),
            "fabric_api" => Ok(Field::FabricApi),
            other => Err(ModError::InvalidArgument(format!(
                "unknown field {other:?}"
            ))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Field::Id => "id",
            Field::Name => "name",
            Field::Version => "version",
            Field::Description => "description",
            Field::Authors => "authors",
            Field::License => "license",
            Field::Minecraft => "minecraft",
            Field::Dependencies => "dependencies",
            Field::Icon => "icon",
            Field::Path => "path",
            Field::Enabled => "enabled",
            Field::Loader => "loader",
            Field::Filename => "filename",
            Field::Size => "size",
            Field::FabricApi => "fabric_api",
        }
    }
}

pub fn parse_fields(s: &str) -> Result<Vec<Field>, ModError> {
    let mut out = Vec::new();
    for part in s.split(',') {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        out.push(Field::parse(p)?);
    }
    Ok(out)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FallbackPolicy {
    #[default]
    Placeholder,
    Skip,
    Fields(Vec<Field>),
}
