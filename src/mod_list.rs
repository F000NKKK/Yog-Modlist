//! View model of the mod browser (`mod_list.yui`). Layout, bindings,
//! scrolling, selection and keyboard handling come from the markup and
//! yog-kit; this file is the data and the logic.

yog_api::ui_include!("mod_list");

use crate::res::Strings;
use yog_kit::prelude::*;

#[derive(Debug, Clone)]
pub struct ModEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub authors: String,
    pub description: String,
    pub source: ModSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModSource {
    Yog,
    Platform,
}

impl ModSource {
    pub fn label(self) -> String {
        match self {
            Self::Yog => Strings::source_yog(),
            Self::Platform => Strings::source_platform(),
        }
    }
    pub fn color(self) -> u32 {
        match self {
            Self::Yog => 0xFF_88FF88,
            Self::Platform => 0xFF_88BBFF,
        }
    }
    fn badge(self) -> String {
        match self {
            Self::Yog => Strings::badge_yog(),
            Self::Platform => Strings::badge_platform(),
        }
    }
}

impl ModEntry {
    pub fn subtitle(&self) -> String {
        format!("{}  {}", Strings::row_version(&self.version), self.source.badge())
    }
    pub fn authors_text(&self) -> String {
        if self.authors.is_empty() {
            Strings::details_authors_unknown()
        } else {
            Strings::details_authors(&self.authors)
        }
    }
    pub fn description_text(&self) -> String {
        if self.description.is_empty() {
            Strings::details_no_description()
        } else {
            self.description.clone()
        }
    }
    /// Name, id or description contains `query` (already lowercase).
    fn matches(&self, query: &str) -> bool {
        query.is_empty()
            || [&self.name, &self.id, &self.description]
                .iter()
                .any(|s| s.to_lowercase().contains(query))
    }
}

/// Which mods the list shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Filter {
    #[default]
    All,
    Yog,
    Platform,
}

impl Filter {
    fn allows(self, e: &ModEntry) -> bool {
        match self {
            Filter::All => true,
            Filter::Yog => e.source == ModSource::Yog,
            Filter::Platform => e.source == ModSource::Platform,
        }
    }
}

#[derive(Default)]
pub struct ModListVm {
    mods: Vec<ModEntry>,
    filter: Filter,
    search: String,
    /// Chosen mod; `None` = the first one shown.
    selected: Option<String>,
}

impl ViewModel for ModListVm {
    /// Every mod has loaded by the time the screen first opens (querying in
    /// `register()` would miss mods that load after this one).
    fn on_open(&mut self, _cx: &mut Cx<'_, Self>) {
        if self.mods.is_empty() {
            self.mods = load_mods();
        }
    }
}

impl ModListVm {
    pub fn visible(&self) -> impl Iterator<Item = &ModEntry> + '_ {
        let query = self.search.trim().to_lowercase();
        self.mods
            .iter()
            .filter(move |m| self.filter.allows(m) && m.matches(&query))
    }
    /// The filter would show something (with the current search).
    pub fn has(&self, f: Filter) -> bool {
        let query = self.search.trim().to_lowercase();
        self.mods.iter().any(|m| f.allows(m) && m.matches(&query))
    }
    pub fn count_text(&self) -> String {
        let (shown, total) = (self.visible().count(), self.mods.len());
        if shown == total {
            Strings::screen_count(total)
        } else {
            Strings::screen_count_filtered(shown, total)
        }
    }
    pub fn empty_text(&self) -> String {
        if self.search.trim().is_empty() {
            Strings::list_empty()
        } else {
            Strings::list_no_matches()
        }
    }
    pub fn selected(&self) -> Option<&ModEntry> {
        match &self.selected {
            Some(id) => self.visible().find(|m| &m.id == id),
            None => self.visible().next(),
        }
    }
    pub fn selected_id(&self) -> Option<&str> {
        self.selected().map(|m| m.id.as_str())
    }
    /// The list scrolls back to the top when this changes.
    pub fn list_scope(&self) -> (Filter, &str) {
        (self.filter, self.search.as_str())
    }

    pub fn select(&mut self, id: String) {
        self.selected = Some(id);
    }
    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }
}

/// Every installed mod, Yog mods first, then by name.
fn load_mods() -> Vec<ModEntry> {
    let mut mods: Vec<ModEntry> = yog_api::installed_mods()
        .into_iter()
        .map(|m| ModEntry {
            source: if m.source == "yog" { ModSource::Yog } else { ModSource::Platform },
            id: m.id,
            name: m.name,
            version: m.version,
            authors: m.authors,
            description: m.description,
        })
        .collect();
    mods.sort_by_key(|e| (e.source != ModSource::Yog, e.name.to_lowercase()));
    mods
}
