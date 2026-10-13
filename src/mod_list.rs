//! Code-behind of the mod browser screen (`mod_list.yui`).
//!
//! The markup holds the static layout — header, search box, filter buttons,
//! the list and details panes, the Done button; `yog build` generates
//! `build()`/`render_with()`/`click()`/`ids`/`events` from it into
//! `.yog-build/generated/mod_list.g.rs`, pulled in by `ui_include!` below.
//! This file has the logic: the data, the state, the list rows (added at
//! runtime, like WinForms code adding controls to a panel) and the event
//! handlers.

yog_api::ui_include!("mod_list");

use std::sync::{Mutex, MutexGuard, OnceLock};

use yog_api::ui::{widget, FlexDir, ScrollState, TextBoxEvent, TextBoxState, Units, Widget};

// ── Data ─────────────────────────────────────────────────────────────────────

/// One mod shown in the list.
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

/// Every installed mod, Yog mods first, then by name. Fetched on the first
/// render — by then every mod has finished loading (querying inside
/// `register()` would miss mods that load after this one).
fn mod_entries() -> &'static [ModEntry] {
    static ENTRIES: OnceLock<Vec<ModEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        let mut entries: Vec<ModEntry> = yog_api::installed_mods()
            .into_iter()
            .map(|m| ModEntry {
                source: if m.source == "yog" {
                    ModSource::Yog
                } else {
                    ModSource::Platform
                },
                id: m.id,
                name: m.name,
                version: m.version,
                authors: m.authors,
                description: m.description,
            })
            .collect();
        entries.sort_by_key(|e| (e.source != ModSource::Yog, e.name.to_lowercase()));
        entries
    })
}

// ── State ────────────────────────────────────────────────────────────────────

/// Which mods the list shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Yog,
    Platform,
}

impl Filter {
    fn matches(self, e: &ModEntry) -> bool {
        match self {
            Filter::All => true,
            Filter::Yog => e.source == ModSource::Yog,
            Filter::Platform => e.source == ModSource::Platform,
        }
    }
}

/// Whether `e` matches the search text (name, id or description; any case).
fn matches_search(e: &ModEntry, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || [&e.name, &e.id, &e.description]
            .iter()
            .any(|field| field.to_lowercase().contains(&query))
}

struct State {
    filter: Filter,
    search: TextBoxState,
    /// Id of the selected mod; `None` = the first one shown.
    selected: Option<String>,
    list: ScrollState,
    details: ScrollState,
}

static STATE: Mutex<State> = Mutex::new(State {
    filter: Filter::All,
    search: TextBoxState::new(),
    selected: None,
    list: ScrollState::new(),
    details: ScrollState::new(),
});

fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// The mods passing `filter` and the search text `query`, with their index
/// in [`mod_entries`].
fn visible_with(filter: Filter, query: &str) -> impl Iterator<Item = (usize, &'static ModEntry)> + '_ {
    mod_entries()
        .iter()
        .enumerate()
        .filter(move |(_, e)| filter.matches(e) && matches_search(e, query))
}

/// The mods the list shows.
fn visible(st: &State) -> impl Iterator<Item = (usize, &'static ModEntry)> + '_ {
    visible_with(st.filter, st.search.text())
}

/// The selected mod, or the first one shown.
fn selection(st: &State) -> Option<(usize, &'static ModEntry)> {
    match &st.selected {
        Some(id) => visible(st).find(|(_, e)| &e.id == id),
        None => visible(st).next(),
    }
}

// ── Rendering ────────────────────────────────────────────────────────────────

/// Event raised by a list row: `select:<index into mod_entries()>`.
const SELECT: &str = "select:";
/// Wheel scroll per notch: about one row.
const WHEEL_STEP: f32 = 26.0;

const ROW_BG: u32 = 0x44_333333;
const ROW_SELECTED_BG: u32 = 0x88_3A3A5A;
const FILTER_BG: u32 = 0xFF_333333;
const FILTER_ACTIVE_BG: u32 = 0xFF_555555;

fn source_label(source: ModSource) -> &'static str {
    match source {
        ModSource::Yog => "Yog mod",
        ModSource::Platform => "Platform mod",
    }
}

/// One list row: name, then version and source.
fn row(index: usize, e: &ModEntry, selected: bool) -> Widget {
    let badge = match e.source {
        ModSource::Yog => "[Yog]",
        ModSource::Platform => "[MC]",
    };
    widget::panel(FlexDir::Column)
        .w(Units::Percent(100.0))
        .padding(3.0, 4.0, 3.0, 4.0)
        .gap(1.0)
        .bg(if selected { ROW_SELECTED_BG } else { ROW_BG })
        .on_click(format!("{SELECT}{index}"))
        .child(
            widget::label(&e.name)
                .w(Units::Percent(100.0))
                .color(if selected { 0xFF_FFD700 } else { 0xFF_FFFFFF })
                .ellipsis(),
        )
        .child(
            widget::label(format!("v{}  {badge}", e.version))
                .w(Units::Percent(100.0))
                .color(0xAA_AAAAAA)
                .shadow(false)
                .ellipsis(),
        )
}

/// Sets a label's text (and color, if given).
fn set(root: &mut Widget, id: &str, text: &str, color: Option<u32>) {
    if let Some(w) = root.find_mut(id) {
        w.set_text(text);
        if let Some(c) = color {
            w.style.color = c;
        }
    }
}

/// Fills the designed tree in with the current state.
fn populate(root: &mut Widget) {
    let mut st = state();
    if let Some(search) = root.find_mut(ids::SEARCH) {
        st.search.apply(search);
    }
    let total = mod_entries().len();
    let shown = visible(&st).count();
    let count = if shown == total {
        format!("{total} mods")
    } else {
        format!("{shown} of {total} mods")
    };
    set(root, ids::MOD_COUNT, &count, None);

    for (id, filter) in [
        (ids::FILTER_ALL, Filter::All),
        (ids::FILTER_YOG, Filter::Yog),
        (ids::FILTER_PLATFORM, Filter::Platform),
    ] {
        if let Some(b) = root.find_mut(id) {
            let any = visible_with(filter, st.search.text()).next().is_some();
            b.style.bg = if st.filter == filter {
                FILTER_ACTIVE_BG
            } else {
                FILTER_BG
            };
            b.style.color = if any { 0xFF_FFFFFF } else { 0xFF_777777 };
            b.enabled = any;
        }
    }

    let current = selection(&st);
    if let Some(list) = root.find_mut(ids::MOD_LIST) {
        list.style.scroll_y = st.list.offset;
        let selected = current.map(|(i, _)| i);
        list.children = visible(&st)
            .map(|(i, e)| row(i, e, Some(i) == selected))
            .collect();
        if list.children.is_empty() {
            let empty = if st.search.text().trim().is_empty() {
                "No mods"
            } else {
                "No mods match the search"
            };
            list.children
                .push(widget::label(empty).color(0xFF_888888).shadow(false));
        }
    }

    if let Some(details) = root.find_mut(ids::DETAILS) {
        details.style.scroll_y = st.details.offset;
    }
    match current {
        Some((_, e)) => {
            set(root, ids::DETAIL_NAME, &e.name, None);
            set(
                root,
                ids::DETAIL_VERSION,
                &format!("Version {}", e.version),
                None,
            );
            let source_color = match e.source {
                ModSource::Yog => 0xFF_88FF88,
                ModSource::Platform => 0xFF_88BBFF,
            };
            set(
                root,
                ids::DETAIL_SOURCE,
                source_label(e.source),
                Some(source_color),
            );
            let by = if e.authors.is_empty() {
                "by unknown".to_string()
            } else {
                format!("by {}", e.authors)
            };
            set(root, ids::DETAIL_AUTHORS, &by, None);
            set(root, ids::DETAIL_ID, &format!("id: {}", e.id), None);
            let desc = if e.description.is_empty() {
                "No description."
            } else {
                e.description.as_str()
            };
            set(root, ids::DETAIL_DESCRIPTION, desc, None);
        }
        None => {
            set(root, ids::DETAIL_NAME, "No mod selected", None);
            for id in [
                ids::DETAIL_VERSION,
                ids::DETAIL_SOURCE,
                ids::DETAIL_AUTHORS,
                ids::DETAIL_ID,
                ids::DETAIL_DESCRIPTION,
            ] {
                set(root, id, "", None);
            }
        }
    }
}

// ── Events ───────────────────────────────────────────────────────────────────

/// Wheel and scrollbar events for the two scroll views, against the layout
/// the user saw. The wheel scrolls the list (it carries no cursor position).
fn scroll_event(event: &str) -> bool {
    with_last(|ui| {
        let mut st = state();
        let st = &mut *st;
        let list = ui.find_node(ids::MOD_LIST);
        let details = ui.find_node(ids::DETAILS);
        list.is_some_and(|n| st.list.handle(event, n, WHEEL_STEP))
            || (!event.starts_with("scroll:")
                && details.is_some_and(|n| st.details.handle(event, n, WHEEL_STEP)))
    })
    .unwrap_or(false)
}

/// After the filter or the search text changed: back to the list's top,
/// keeping the selection if it's still shown.
fn list_changed(st: &mut State) {
    st.list.reset();
    if selection(st).is_none() {
        st.selected = None;
        st.details.reset();
    }
}

fn set_filter(filter: Filter) {
    let mut st = state();
    if st.filter != filter {
        st.filter = filter;
        list_changed(&mut st);
    }
}

/// The search box's share of an event; `true` when it used it (typing,
/// caret keys, Esc unfocusing the box, ...), which keeps the key from the
/// game — Esc then doesn't close the screen.
fn search_event(event: &str) -> bool {
    let mut st = state();
    let st = &mut *st;
    let result = text_box(ids::SEARCH, &mut st.search, event);
    if result == TextBoxEvent::Changed {
        list_changed(st);
    }
    result.consumed()
}

fn select(index: usize) {
    let Some(e) = mod_entries().get(index) else {
        return;
    };
    let mut st = state();
    if st.selected.as_deref() != Some(e.id.as_str()) {
        st.selected = Some(e.id.clone());
        st.details.reset();
    }
}

/// Handles one UI event; `true` = used (see [`search_event`]).
fn on_event(event: &str) -> bool {
    if search_event(event) || scroll_event(event) {
        return true;
    }
    let Some(clicked) = click(event) else {
        return false;
    };
    match clicked.as_str() {
        events::DONE => close(),
        events::FILTER_ALL => set_filter(Filter::All),
        events::FILTER_YOG => set_filter(Filter::Yog),
        events::FILTER_PLATFORM => set_filter(Filter::Platform),
        other => {
            if let Some(i) = other.strip_prefix(SELECT).and_then(|i| i.parse().ok()) {
                select(i);
            }
        }
    }
    true
}

/// Registers the screen; open it with [`open`] or a menu entry on [`ID`].
pub fn register(registry: &mut yog_api::Registry) {
    registry.on_ui_render(ID, |gfx| render_with(gfx, populate));
    registry.register_ui(ID, |_ui_id, event| on_event(event));
}
