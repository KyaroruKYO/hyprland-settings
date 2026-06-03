use crate::ui::model::{UiProjection, UiSetting};

#[derive(Debug, Clone)]
pub struct SearchProjection {
    pub query: String,
    pub is_searching: bool,
    pub title: String,
    pub results: Vec<SearchResult>,
    pub empty_title: Option<String>,
    pub empty_detail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub setting: UiSetting,
    pub rank: Option<SearchRank>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SearchRank {
    ExactKey,
    PrefixKey,
    Label,
    Context,
    Description,
    Metadata,
}

struct SearchIndexEntry {
    tab_order: usize,
    row_id: String,
    official_setting: String,
    label: String,
    context: String,
    description: String,
    searchable_text: String,
}

pub fn search_projection(
    projection: &UiProjection,
    selected_tab_id: &str,
    query: &str,
) -> SearchProjection {
    let query = query.trim();
    if query.is_empty() {
        return selected_tab_projection(projection, selected_tab_id);
    }

    let query_key = canonical(query);
    let terms = search_terms(query);
    let mut matches: Vec<_> = projection
        .settings
        .iter()
        .enumerate()
        .filter_map(|(source_index, setting)| {
            let entry = SearchIndexEntry::new(projection, setting);
            entry.matches(&terms).then(|| SearchResultWithSort {
                result: SearchResult {
                    setting: setting.clone(),
                    rank: Some(entry.rank(&query_key, &terms)),
                },
                tab_order: entry.tab_order,
                row_order: setting.row_order,
                source_index,
            })
        })
        .collect();

    matches.sort_by_key(|entry| {
        (
            entry.result.rank.unwrap_or(SearchRank::Metadata),
            entry.tab_order,
            entry.row_order,
            entry.source_index,
        )
    });

    let results: Vec<_> = matches.into_iter().map(|entry| entry.result).collect();
    let empty = results.is_empty();

    SearchProjection {
        query: query.to_string(),
        is_searching: true,
        title: format!("Search results for \"{}\" · {} rows", query, results.len()),
        results,
        empty_title: empty.then(|| "No matching settings found.".to_string()),
        empty_detail: empty.then(|| "Search uses export metadata only.".to_string()),
    }
}

fn selected_tab_projection(projection: &UiProjection, selected_tab_id: &str) -> SearchProjection {
    let tab = projection
        .tabs
        .iter()
        .find(|tab| tab.id == selected_tab_id)
        .or_else(|| projection.tabs.first());

    let Some(tab) = tab else {
        return SearchProjection {
            query: String::new(),
            is_searching: false,
            title: "No tab selected".to_string(),
            results: Vec::new(),
            empty_title: Some("No scalar settings are exported for this tab yet.".to_string()),
            empty_detail: None,
        };
    };

    let settings = projection.settings_for_tab(&tab.id);
    let empty = settings.is_empty();
    SearchProjection {
        query: String::new(),
        is_searching: false,
        title: format!("{} · {} rows", tab.label, tab.row_count),
        results: settings
            .into_iter()
            .map(|setting| SearchResult {
                setting,
                rank: None,
            })
            .collect(),
        empty_title: empty.then(|| "No scalar settings are exported for this tab yet.".to_string()),
        empty_detail: None,
    }
}

struct SearchResultWithSort {
    result: SearchResult,
    tab_order: usize,
    row_order: usize,
    source_index: usize,
}

impl SearchIndexEntry {
    fn new(projection: &UiProjection, setting: &UiSetting) -> Self {
        let row_id = canonical(&setting.row_id);
        let official_setting = canonical(&setting.official_setting);
        let label = searchable(&setting.label);
        let context = searchable(&format!(
            "{} {} {}",
            setting.tab_id, setting.tab_label, setting.subsection
        ));
        let description = searchable(&setting.description);
        let metadata = searchable(&format!(
            "{} {} {} {} {} {}",
            setting.read_support,
            setting.write_support,
            setting.risk_class,
            setting.preview_status,
            if setting.report_only {
                "report-only"
            } else {
                ""
            },
            if setting.is_write_candidate {
                "write-candidate"
            } else {
                ""
            }
        ));
        let searchable_text = [
            searchable(&setting.row_id),
            searchable(&setting.official_setting),
            label.clone(),
            context.clone(),
            description.clone(),
            metadata.clone(),
        ]
        .join(" ");

        Self {
            tab_order: projection.tab_order_for(&setting.tab_id),
            row_id,
            official_setting,
            label,
            context,
            description,
            searchable_text,
        }
    }

    fn matches(&self, terms: &[String]) -> bool {
        terms
            .iter()
            .all(|term| self.searchable_text.contains(term.as_str()))
    }

    fn rank(&self, query: &str, terms: &[String]) -> SearchRank {
        if self.row_id == query || self.official_setting == query {
            SearchRank::ExactKey
        } else if self.row_id.starts_with(query) || self.official_setting.starts_with(query) {
            SearchRank::PrefixKey
        } else if field_matches(&self.label, terms) {
            SearchRank::Label
        } else if field_matches(&self.context, terms) {
            SearchRank::Context
        } else if field_matches(&self.description, terms) {
            SearchRank::Description
        } else {
            SearchRank::Metadata
        }
    }
}

fn field_matches(text: &str, terms: &[String]) -> bool {
    terms.iter().all(|term| text.contains(term.as_str()))
}

fn search_terms(query: &str) -> Vec<String> {
    searchable(query)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

fn canonical(text: &str) -> String {
    text.trim().to_lowercase()
}

fn searchable(text: &str) -> String {
    let lowered = canonical(text);
    let separated: String = lowered
        .chars()
        .map(|ch| match ch {
            '.' | ':' | '_' | '-' | '/' => ' ',
            _ => ch,
        })
        .collect();
    format!("{} {}", lowered, separated)
}
