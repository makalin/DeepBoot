use crate::models::{StartupEntry, StartupSource};

#[derive(Debug, Clone)]
pub struct Filter {
    pub search_term: Option<String>,
    pub source_filter: Option<Vec<StartupSource>>,
    pub enabled_only: Option<bool>,
    pub disabled_only: Option<bool>,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            search_term: None,
            source_filter: None,
            enabled_only: None,
            disabled_only: None,
        }
    }
}

impl Filter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_search(mut self, term: String) -> Self {
        self.search_term = Some(term.to_lowercase());
        self
    }

    pub fn with_source(mut self, sources: Vec<StartupSource>) -> Self {
        self.source_filter = Some(sources);
        self
    }

    pub fn enabled_only(mut self) -> Self {
        self.enabled_only = Some(true);
        self.disabled_only = None;
        self
    }

    pub fn disabled_only(mut self) -> Self {
        self.disabled_only = Some(true);
        self.enabled_only = None;
        self
    }

    pub fn apply(&self, entries: &[StartupEntry]) -> Vec<StartupEntry> {
        entries
            .iter()
            .filter(|entry| {
                // Search term filter
                if let Some(ref term) = self.search_term {
                    let name_match = entry.name.to_lowercase().contains(term);
                    let command_match = entry.command.to_lowercase().contains(term);
                    let desc_match = entry
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(term))
                        .unwrap_or(false);
                    if !name_match && !command_match && !desc_match {
                        return false;
                    }
                }

                // Source filter
                if let Some(ref sources) = self.source_filter {
                    if !sources.contains(&entry.source) {
                        return false;
                    }
                }

                // Enabled/Disabled filter
                if let Some(true) = self.enabled_only {
                    if !entry.enabled {
                        return false;
                    }
                }

                if let Some(true) = self.disabled_only {
                    if entry.enabled {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) {
        self.search_term = None;
        self.source_filter = None;
        self.enabled_only = None;
        self.disabled_only = None;
    }
}

pub fn sort_entries(entries: &mut [StartupEntry], sort_by: SortBy) {
    match sort_by {
        SortBy::Name => {
            entries.sort_by(|a, b| a.name.cmp(&b.name));
        }
        SortBy::Source => {
            entries.sort_by(|a, b| a.source.to_string().cmp(&b.source.to_string()));
        }
        SortBy::Status => {
            entries.sort_by(|a, b| b.enabled.cmp(&a.enabled)); // Enabled first
        }
        SortBy::Command => {
            entries.sort_by(|a, b| a.command.cmp(&b.command));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Name,
    Source,
    Status,
    Command,
}

