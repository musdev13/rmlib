use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    Mod,
    Modpack,
    Resourcepack,
    Shader,
    Datapack,
    World,
    Plugin,
}

impl ProjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Mod => "mod",
            ProjectType::Modpack => "modpack",
            ProjectType::Resourcepack => "resourcepack",
            ProjectType::Shader => "shader",
            ProjectType::Datapack => "datapack",
            ProjectType::World => "world",
            ProjectType::Plugin => "plugin",
        }
    }

    pub fn as_dir(&self) -> Option<&'static str> {
        match self {
            ProjectType::Mod => Some("mods"),
            ProjectType::Modpack => None,
            ProjectType::Resourcepack => Some("resourcepacks"),
            ProjectType::Shader => Some("shaderpacks"),
            ProjectType::Datapack => Some("datapacks"),
            ProjectType::World => None,
            ProjectType::Plugin => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchIndex {
    Relevance,
    Downloads,
    Follows,
    Newest,
    Updated,
}

impl SearchIndex {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchIndex::Relevance => "relevance",
            SearchIndex::Downloads => "downloads",
            SearchIndex::Follows => "follows",
            SearchIndex::Newest => "newest",
            SearchIndex::Updated => "updated",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideSupport {
    Required,
    Optional,
    Unsupported,
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchParams {
    pub query: Option<String>,
    pub facets: Option<String>,
    pub index: Option<SearchIndex>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchHit {
    pub project_id: String,
    pub project_type: String,
    pub slug: String,
    pub author: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    #[serde(default)]
    pub display_categories: Vec<String>,
    pub versions: Vec<String>,
    pub downloads: u64,
    pub follows: u64,
    #[serde(default)]
    pub icon_url: Option<String>,
    pub date_created: String,
    pub date_modified: String,
    #[serde(default)]
    pub latest_version: Option<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub client_side: Option<SideSupport>,
    #[serde(default)]
    pub server_side: Option<SideSupport>,
    #[serde(default)]
    pub gallery: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u64,
}
