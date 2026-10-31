use super::dto::{ProjectType, SearchIndex, SearchParams};

#[derive(Debug, Clone, Default)]
pub struct FacetBuilder {
    groups: Vec<Vec<String>>,
}

impl FacetBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn project_type(mut self, t: ProjectType) -> Self {
        self.add("project_type", t.as_str());
        self
    }

    pub fn game_version(mut self, v: &str) -> Self {
        self.add("versions", v);
        self
    }

    pub fn loader(mut self, l: &str) -> Self {
        self.add("categories", l);
        self
    }

    pub fn category(mut self, c: &str) -> Self {
        self.add("categories", c);
        self
    }

    pub fn client_side(mut self, side: &str) -> Self {
        self.add("client_side", side);
        self
    }

    pub fn server_side(mut self, side: &str) -> Self {
        self.add("server_side", side);
        self
    }

    fn add(&mut self, key: &str, value: &str) {
        let facet = format!("\"{key}:{value}\"");
        if let Some(group) = self.groups.iter_mut().find(|g| {
            g.first()
                .map(|f| f.starts_with(&format!("\"{key}:")))
                .unwrap_or(false)
        }) {
            group.push(facet);
        } else {
            self.groups.push(vec![facet]);
        }
    }

    pub fn build(&self) -> String {
        let parts: Vec<String> = self
            .groups
            .iter()
            .map(|g| format!("[{}]", g.join(",")))
            .collect();
        format!("[{}]", parts.join(","))
    }
}

pub fn build_query(params: &SearchParams) -> Vec<(&'static str, String)> {
    let mut q: Vec<(&'static str, String)> = Vec::new();
    if let Some(query) = &params.query
        && !query.is_empty()
    {
        q.push(("query", query.clone()));
    }
    if let Some(facets) = &params.facets
        && !facets.is_empty()
    {
        q.push(("facets", facets.clone()));
    }
    if let Some(index) = params.index {
        q.push(("index", index.as_str().to_string()));
    }
    if let Some(offset) = params.offset {
        q.push(("offset", offset.to_string()));
    }
    if let Some(limit) = params.limit {
        q.push(("limit", limit.to_string()));
    }
    q
}

pub fn page_to_offset(page: u32, page_size: u32) -> Result<u32, super::error::ModrinthError> {
    if page == 0 || page_size == 0 {
        return Err(super::error::ModrinthError::InvalidArgument(
            "page and page_size must be >= 1".to_string(),
        ));
    }
    page.checked_sub(1)
        .and_then(|p| p.checked_mul(page_size))
        .ok_or_else(|| {
            super::error::ModrinthError::InvalidArgument("pagination overflow".to_string())
        })
}

pub fn normalize_limit(limit: u32) -> u32 {
    limit.clamp(1, 100)
}

pub fn search_index_from_str(s: &str) -> Option<SearchIndex> {
    match s {
        "relevance" => Some(SearchIndex::Relevance),
        "downloads" => Some(SearchIndex::Downloads),
        "follows" => Some(SearchIndex::Follows),
        "newest" => Some(SearchIndex::Newest),
        "updated" => Some(SearchIndex::Updated),
        _ => None,
    }
}
