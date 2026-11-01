use serde::{Deserialize, Serialize};

use super::error::RpError;

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
    pub fn parse(s: &str) -> Result<Self, RpError> {
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
                        return Err(RpError::InvalidArgument(
                            "priority source \"modrinth\" requires the rmlib `modrinth` feature"
                                .to_string(),
                        ));
                    }
                    Source::Modrinth
                }
                other => {
                    return Err(RpError::InvalidArgument(format!(
                        "unknown priority source {other:?}"
                    )));
                }
            };
            if !sources.contains(&src) {
                sources.push(src);
            }
        }
        if sources.is_empty() {
            return Err(RpError::InvalidArgument(
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
    Description,
    PackFormat,
    SupportedFormats,
    Icon,
    Path,
    Enabled,
    Filename,
    Size,
}

impl Field {
    pub fn parse(s: &str) -> Result<Self, RpError> {
        match s.trim() {
            "id" => Ok(Field::Id),
            "description" => Ok(Field::Description),
            "pack_format" => Ok(Field::PackFormat),
            "supported_formats" => Ok(Field::SupportedFormats),
            "icon" => Ok(Field::Icon),
            "path" => Ok(Field::Path),
            "enabled" => Ok(Field::Enabled),
            "filename" => Ok(Field::Filename),
            "size" => Ok(Field::Size),
            other => Err(RpError::InvalidArgument(format!("unknown field {other:?}"))),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Field::Id => "id",
            Field::Description => "description",
            Field::PackFormat => "pack_format",
            Field::SupportedFormats => "supported_formats",
            Field::Icon => "icon",
            Field::Path => "path",
            Field::Enabled => "enabled",
            Field::Filename => "filename",
            Field::Size => "size",
        }
    }
}

pub fn parse_fields(s: &str) -> Result<Vec<Field>, RpError> {
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
