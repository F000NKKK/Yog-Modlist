//! Mod-list UI — built with yog-ui, rendered inside a YogUIScreen.

use std::sync::Mutex;

use yog_api::GfxContext;

// ── Mod entry data ───────────────────────────────────────────────────────────

/// A single mod entry shown in the list.
#[derive(Debug, Clone)]
pub struct ModEntry {
    pub id:          String,
    pub name:        String,
    pub version:     String,
    pub authors:     String,
    pub description: String,
    pub source:      ModSource,
}

#[derive(Debug, Clone)]
pub enum ModSource {
    Yog,
    Platform,
}

// ── Placeholder data (will be replaced by `registry.mods()` in a future ABI) ──

pub fn gather_mod_entries() -> Vec<ModEntry> {
    vec![
        ModEntry {
            id: "yog-modlist".into(),
            name: "Yog Mod List".into(),
            version: "0.1.0".into(),
            authors: "F000NK".into(),
            description: "In-game mod list browser for Yog.".into(),
            source: ModSource::Yog,
        },
        ModEntry {
            id: "hexmod-yog".into(),
            name: "HexMod Yog".into(),
            version: "0.1.0".into(),
            authors: "F000NK, Yog Team".into(),
            description: "HexCasting ported to Yog loader. Programmable magic through hex patterns.".into(),
            source: ModSource::Yog,
        },
    ]
}

// ── Static UI state ───────────────────────────────────────────────────────────

/// Stored layout for hit-testing on click events.
static LAST_LAYOUT: Mutex<Option<yog_api::LayoutNode>> = Mutex::new(None);

/// The screen dimensions from the last render frame.
static LAST_SCREEN: Mutex<(f32, f32)> = Mutex::new((854.0, 480.0));

// ── Rendering ────────────────────────────────────────────────────────────────

pub fn render_mod_list(gfx: &GfxContext, entries: &[ModEntry]) {
    let d2d = gfx.draw2d();
    let (sw_i, sh_i) = gfx.screen_size();
    let sw = sw_i as f32;
    let sh = sh_i as f32;

    // Store screen size for hit-testing.
    *LAST_SCREEN.lock().unwrap() = (sw, sh);

    // ── Build a simple procedural UI (no widget tree yet — direct draw2d) ──
    let pad  = 12.0;
    let x0   = pad;
    let y0   = pad;
    let w    = sw - pad * 2.0;
    let item_h = 64.0;
    let gap    = 4.0;

    // Title bar
    d2d.rect(x0, y0, x0 + w, y0 + 28.0, 0xCC_222222);
    d2d.text("Yog Mods", x0 + 8.0, y0 + 6.0, 0xFF_FFD700, true);
    d2d.text(
        &format!("{} mod(s)", entries.len()),
        x0 + w - 80.0, y0 + 6.0, 0xAA_AAAAAA, false,
    );

    // Scroll region background
    let list_y0 = y0 + 30.0;
    let list_h  = sh - list_y0 - pad;
    d2d.rect(x0, list_y0, x0 + w, list_y0 + list_h, 0x88_111111);

    // Items
    let mut item_y = list_y0 + 2.0;
    for e in entries {
        let iy = item_y;
        // Background
        d2d.rect(x0 + 2.0, iy, x0 + w - 2.0, iy + item_h, 0x44_333333);

        let name_x  = x0 + 8.0;
        let name_y  = iy + 4.0;
        let ver_x   = name_x + 200.0;

        d2d.text(&e.name, name_x, name_y, 0xFF_FFFFFF, true);
        let ver = format!("v{}", e.version);
        d2d.text(&ver, ver_x, name_y, 0xAA_AAAAAA, false);

        // Authors
        d2d.text(
            &format!("by {}", e.authors),
            name_x, name_y + 14.0, 0xCC_888888, false,
        );

        // Source badge
        let badge = match e.source {
            ModSource::Yog     => "[Yog]",
            ModSource::Platform => "[MC]",
        };
        d2d.text(badge, x0 + w - 40.0, name_y, 0x88_44FF44, false);

        // Description (truncated to one line)
        let desc = if e.description.len() > 56 {
            format!("{}...", &e.description[..53])
        } else {
            e.description.clone()
        };
        d2d.text(&desc, name_x, name_y + 28.0, 0x99_BBBBBB, false);

        item_y += item_h + gap;
    }

    // Store a fake layout for future hit-testing support.
    *LAST_LAYOUT.lock().unwrap() = Some(yog_api::LayoutNode {
        rect: yog_api::Rect { x: x0, y: y0, w, h: sh - y0 },
        ..Default::default()
    });
}

// ── Event handling ───────────────────────────────────────────────────────────

pub fn handle_ui_event(ui_id: &str, event: &str) {
    // Future: parse "click:X:Y" events and hit-test against stored layout.
    if event == "close" {
        yog_api::info!("[yog-modlist] UI closed: {}", ui_id);
    }
}
