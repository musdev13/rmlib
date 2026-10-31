use super::client::get_json;
use super::dto::{ProjectType, SearchIndex, SearchParams, SearchResponse, SideSupport};
use super::error::ModrinthError;
use super::facets::{FacetBuilder, build_query, normalize_limit};

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub query: Option<String>,
    pub project_type: Option<ProjectType>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub categories: Vec<String>,
    pub client_side: Option<SideSupport>,
    pub server_side: Option<SideSupport>,
    pub index: Option<SearchIndex>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl SearchOptions {
    pub fn to_params(&self) -> SearchParams {
        let mut facets = FacetBuilder::new();
        if let Some(t) = self.project_type {
            facets = facets.project_type(t);
        }
        for v in &self.game_versions {
            facets = facets.game_version(v);
        }
        for l in &self.loaders {
            facets = facets.loader(l);
        }
        for c in &self.categories {
            facets = facets.category(c);
        }
        if let Some(side) = self.client_side {
            facets = facets.client_side(side_as_str(side));
        }
        if let Some(side) = self.server_side {
            facets = facets.server_side(side_as_str(side));
        }

        let facets_str = if facets_is_empty(&facets) {
            None
        } else {
            Some(facets.build())
        };

        let limit = self.limit.map(normalize_limit);
        let offset = match (self.offset, self.page, limit) {
            (Some(o), _, _) => Some(o),
            (None, Some(p), Some(l)) => super::facets::page_to_offset(p, l).ok(),
            _ => None,
        };

        SearchParams {
            query: self.query.clone(),
            facets: facets_str,
            index: self.index,
            offset,
            limit,
        }
    }
}

fn side_as_str(side: SideSupport) -> &'static str {
    match side {
        SideSupport::Required => "required",
        SideSupport::Optional => "optional",
        SideSupport::Unsupported => "unsupported",
        SideSupport::Unknown => "unknown",
    }
}

fn facets_is_empty(b: &FacetBuilder) -> bool {
    b.build() == "[]"
}

pub async fn search(options: &SearchOptions) -> Result<SearchResponse, ModrinthError> {
    let params = options.to_params();
    let query = build_query(&params);
    get_json("/search", &query).await
}
