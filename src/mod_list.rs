//! View model of the mod browser (`mod_list.yui`). Layout, bindings,
//! scrolling, selection and keyboard handling come from the markup and
//! yog-kit; this file is the data and the logic.

yog_api::ui_include!("mod_list");

use crate::res::{Colors, Strings};
use yog_api::ModInfo;
use yog_kit::prelude::*;

/// `ModInfo::source` of a .yog mod (anything else is a platform mod).
const YOG_SOURCE: &str = "yog";

#[derive(Debug, Clone)]
pub struct ModEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub authors: String,
    pub description: String,
    pub source: ModSource,
    /// Lowercase name, id and description, `\0`-separated: what the search
    /// matches against, so a search doesn't lowercase every mod per update.
    search_key: String,
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
            Self::Yog => Colors::SOURCE_YOG,
            Self::Platform => Colors::SOURCE_PLATFORM,
        }
    }
    fn badge(self) -> String {
        match self {
            Self::Yog => Strings::badge_yog(),
            Self::Platform => Strings::badge_platform(),
        }
    }
}

impl From<ModInfo> for ModEntry {
    fn from(m: ModInfo) -> Self {
        let search_key = [&m.name, &m.id, &m.description]
            .map(|s| s.to_lowercase())
            .join("\0");
        Self {
            source: if m.source == YOG_SOURCE {
                ModSource::Yog
            } else {
                ModSource::Platform
            },
            id: m.id,
            name: m.name,
            version: m.version,
            authors: m.authors,
            description: m.description,
            search_key,
        }
    }
}

impl ModEntry {
    pub fn subtitle(&self) -> String {
        format!(
            "{}  {}",
            Strings::row_version(&self.version),
            self.source.badge()
        )
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
        self.search_key.contains(query)
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
    /// The mods `filter` shows with the current search.
    fn shown(&self, filter: Filter) -> impl Iterator<Item = &ModEntry> + '_ {
        let query = self.search.trim().to_lowercase();
        self.mods
            .iter()
            .filter(move |m| filter.allows(m) && m.matches(&query))
    }
    pub fn visible(&self) -> impl Iterator<Item = &ModEntry> + '_ {
        self.shown(self.filter)
    }
    /// The filter would show something (with the current search).
    pub fn has(&self, f: Filter) -> bool {
        self.shown(f).next().is_some()
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
        .map(ModEntry::from)
        .collect();
    mods.sort_by_cached_key(|e| (e.source != ModSource::Yog, e.name.to_lowercase()));
    mods
}

#[cfg(test)]
mod tests {
    use super::*;
    use yog_kit::testing::TestHost;

    fn entry(id: &str, source: &str) -> ModEntry {
        ModInfo {
            source: source.into(),
            id: id.into(),
            name: id.to_uppercase(),
            version: "1.0".into(),
            authors: String::new(),
            description: format!("the {id} mod"),
        }
        .into()
    }

    fn host() -> TestHost<View> {
        let mods = vec![
            entry("alpha", YOG_SOURCE),
            entry("beta", YOG_SOURCE),
            entry("fabric-api", "platform"),
        ];
        let mut h = TestHost::<View>::new(ModListVm {
            mods,
            ..Default::default()
        });
        h.size(427.0, 240.0).open();
        h
    }

    #[test]
    fn filters_search_and_selection() {
        let mut h = host();
        assert_eq!(h.text(ids::DETAIL_NAME).as_deref(), Some("ALPHA"));
        h.click_id(ids::FILTER_PLATFORM);
        assert_eq!(h.vm().filter, Filter::Platform);
        assert!(h
            .state(ids::FILTER_PLATFORM)
            .contains(yog_api::ui::StateFlags::CHECKED));
        assert_eq!(h.text(ids::DETAIL_NAME).as_deref(), Some("FABRIC-API"));

        h.click_id(ids::FILTER_ALL);
        h.key(yog_api::ui::event::keys::DOWN); // list not focused yet: no-op
        let mut row = h.path(ids::MOD_LIST);
        row.push(2); // [top spacer, alpha, beta, ...]
        h.click_path(&row);
        assert_eq!(h.vm().selected.as_deref(), Some("beta"));

        h.type_into(ids::SEARCH, "zzz");
        assert_eq!(h.vm().search, "zzz");
        assert!(!h.vm().has(Filter::Yog));
        assert!(h.widget(ids::DETAIL_NAME).is_none());
    }

    #[test]
    fn each_filter_starts_at_the_top() {
        let mut mods: Vec<ModEntry> = (0..40).map(|i| entry(&format!("yog{i:02}"), YOG_SOURCE)).collect();
        mods.extend((0..40).map(|i| entry(&format!("java{i:02}"), "platform")));
        let mut h = TestHost::<View>::new(ModListVm { mods, ..Default::default() });
        h.size(427.0, 240.0).open();
        let list = h.rect(ids::MOD_LIST).unwrap();
        // The name on the first row the list shows (after its top spacer).
        let first_row = |h: &TestHost<View>| {
            let mut name = h.path(ids::MOD_LIST);
            name.extend([1, 0]);
            h.text_at(&name)
        };
        h.move_to(list.x + 10.0, list.y + 10.0);
        h.scroll(-20.0);
        assert!(h.scroll_offset(0) > 0.0, "the list scrolled");
        h.click_id(ids::FILTER_PLATFORM);
        assert_eq!(h.scroll_offset(0), 0.0, "another filter shows its list from the top");
        assert_eq!(first_row(&h).as_deref(), Some("JAVA00"), "with its first rows, at once");
        h.move_to(list.x + 10.0, list.y + 10.0);
        h.scroll(-20.0);
        h.click_id(ids::FILTER_ALL);
        assert_eq!(h.scroll_offset(0), 0.0);
        assert_eq!(first_row(&h).as_deref(), Some("YOG00"));
    }

    #[test]
    fn search_ignores_case() {
        let mut h = host();
        h.type_into(ids::SEARCH, "FaBrIc");
        assert!(h.vm().has(Filter::Platform));
        assert!(!h.vm().has(Filter::Yog));
        assert_eq!(h.text(ids::DETAIL_NAME).as_deref(), Some("FABRIC-API"));
    }
}
