//! Mod-list UI — procedural draw2d rendering inside a YogUIScreen.
//!
//! Interaction model: the host forwards raw mouse events as strings
//! (`click:X:Y`, `drag:X:Y`, `release:X:Y`, `scroll:DY`); we hit-test against
//! rectangles remembered from the previous frame.

use std::collections::HashSet;
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

// ── Real mod data via the loader (ABI minor 23) ──────────────────────────────

/// Fetched once, on the first render — by then every mod has finished loading,
/// so the list is complete (querying inside `register()` would miss mods that
/// load after this one).
static ENTRIES: std::sync::OnceLock<Vec<ModEntry>> = std::sync::OnceLock::new();

pub fn mod_entries() -> &'static [ModEntry] {
    ENTRIES.get_or_init(|| {
        yog_api::installed_mods()
            .into_iter()
            .map(|m| ModEntry {
                source: if m.source == "yog" { ModSource::Yog } else { ModSource::Platform },
                id: m.id,
                name: m.name,
                version: m.version,
                authors: m.authors,
                description: m.description,
            })
            .collect()
    })
}

// ── UI state ─────────────────────────────────────────────────────────────────

/// Current scroll offset in GUI pixels (0 = top). Updated by wheel/drag
/// events, clamped in render where the content height is known.
static SCROLL: Mutex<f32> = Mutex::new(0.0);

/// Indices of entries with the description card expanded.
static EXPANDED: Mutex<Option<HashSet<usize>>> = Mutex::new(None);

/// Screen-space row rectangles from the last frame: (entry index, y0, y1).
static ROWS: Mutex<Vec<(usize, f32, f32)>> = Mutex::new(Vec::new());

/// Scrollbar geometry from the last frame:
/// (track_x0, track_y0, track_x1, track_y1, knob_y0, knob_y1).
static SCROLLBAR: Mutex<Option<(f32, f32, f32, f32, f32, f32)>> = Mutex::new(None);

/// List metrics from the last frame: (list_y0, list_h, max_scroll, knob_h).
static METRICS: Mutex<(f32, f32, f32, f32)> = Mutex::new((0.0, 0.0, 0.0, 16.0));

/// Active scrollbar drag: grab offset (cursor y − knob y0).
static DRAGGING: Mutex<Option<f32>> = Mutex::new(None);

// ── Text metrics ─────────────────────────────────────────────────────────────

/// Approximate MC font advance in GUI pixels (5 px glyph + 1 px spacing).
const CHAR_W: f32 = 6.0;
const LINE_H: f32 = 12.0;

/// Greedy word wrap into lines of at most `max_chars` characters.
fn wrap(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let need = if line.is_empty() { word.chars().count() }
                   else { line.chars().count() + 1 + word.chars().count() };
        if need > max_chars && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() { line.push(' '); }
        // A single word longer than the line gets hard-split.
        if word.chars().count() > max_chars {
            let mut w = word;
            while w.chars().count() > max_chars {
                let cut: String = w.chars().take(max_chars).collect();
                lines.push(cut.clone());
                w = &w[cut.len()..];
            }
            line.push_str(w);
        } else {
            line.push_str(word);
        }
    }
    if !line.is_empty() { lines.push(line); }
    if lines.is_empty() { lines.push(String::new()); }
    lines
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() > max_chars {
        format!("{}...", text.chars().take(max_chars.saturating_sub(3)).collect::<String>())
    } else {
        text.to_string()
    }
}

// ── Layout constants ─────────────────────────────────────────────────────────

const PAD: f32 = 12.0;
const HEADER_H: f32 = 28.0;
const ROW_H: f32 = 46.0;      // collapsed: name+version / authors / one desc line
const ROW_GAP: f32 = 4.0;

/// Width available for description text inside a row.
fn desc_chars(row_w: f32) -> usize {
    (((row_w - 16.0) / CHAR_W) as usize).max(20)
}

/// Height of one entry row given its expansion state and row width.
fn row_height(e: &ModEntry, expanded: bool, row_w: f32) -> f32 {
    if !expanded {
        return ROW_H;
    }
    let lines = wrap(&e.description, desc_chars(row_w)).len() as f32;
    // name line + authors line + wrapped description + id/source footer + padding
    4.0 + LINE_H * 2.0 + lines * LINE_H + LINE_H + 6.0
}

// ── Rendering ────────────────────────────────────────────────────────────────

pub fn render_mod_list(gfx: &GfxContext, entries: &[ModEntry]) {
    let d2d = gfx.draw2d();
    let (sw_i, sh_i) = gfx.screen_size();
    let sw = sw_i as f32;
    let sh = sh_i as f32;

    let x0 = PAD;
    let y0 = PAD;
    let w  = sw - PAD * 2.0;
    let row_w = w - 10.0; // leave room for the scrollbar

    let list_y0 = y0 + HEADER_H + 2.0;
    let list_h  = sh - list_y0 - PAD;

    let expanded = {
        let guard = EXPANDED.lock().unwrap();
        guard.clone().unwrap_or_default()
    };

    // Content height with per-row (expansion-dependent) heights.
    let heights: Vec<f32> = entries.iter().enumerate()
        .map(|(i, e)| row_height(e, expanded.contains(&i), row_w))
        .collect();
    let content_h: f32 = heights.iter().map(|h| h + ROW_GAP).sum();
    let max_scroll = (content_h - list_h).max(0.0);
    let scroll = {
        let mut s = SCROLL.lock().unwrap();
        *s = s.clamp(0.0, max_scroll);
        *s
    };

    // List background
    d2d.rect(x0, list_y0, x0 + w, list_y0 + list_h, 0x88_111111);

    // Rows
    let mut rows = Vec::with_capacity(entries.len());
    let mut item_y = list_y0 + 2.0 - scroll;
    for (i, e) in entries.iter().enumerate() {
        let h = heights[i];
        let iy = item_y;
        item_y += h + ROW_GAP;
        rows.push((i, iy, iy + h));
        // Cull rows fully outside the list region (no scissor in draw2d —
        // partially visible top rows are covered by the header redraw below).
        if iy + h < list_y0 || iy > list_y0 + list_h {
            continue;
        }
        let is_open = expanded.contains(&i);
        // Clamp the plate to the list region (no scissor in draw2d).
        d2d.rect(x0 + 2.0, iy.max(list_y0), x0 + row_w, (iy + h).min(list_y0 + list_h),
                 if is_open { 0x66_2a2a3a } else { 0x44_333333 });

        // Text goes through the MC pipeline and is drawn AFTER all rects
        // (two-pass overlay rendering) — the header redraw can't cover it, so
        // clip every text line to the list viewport manually.
        let text_visible = |ty: f32| ty >= list_y0 - 1.0 && ty + 10.0 <= list_y0 + list_h;

        let name_x = x0 + 8.0;
        let name_y = iy + 4.0;

        if text_visible(name_y) {
            // ▸ / ▾ expansion marker + name + version
            d2d.text(if is_open { "v" } else { ">" }, name_x, name_y, 0xFF_FFD700, false);
            d2d.text(&e.name, name_x + 12.0, name_y, 0xFF_FFFFFF, true);
            let ver_x = name_x + 12.0 + (e.name.chars().count() as f32 + 1.0) * CHAR_W;
            d2d.text(&format!("v{}", e.version), ver_x, name_y, 0xAA_AAAAAA, false);
            let badge = match e.source {
                ModSource::Yog      => "[Yog]",
                ModSource::Platform => "[MC]",
            };
            d2d.text(badge, x0 + row_w - 38.0, name_y, 0x88_44FF44, false);
        }

        // Authors
        if text_visible(name_y + LINE_H) {
            let by = if e.authors.is_empty() { "by unknown".to_string() }
                     else { format!("by {}", e.authors) };
            d2d.text(&truncate_chars(&by, desc_chars(row_w)), name_x, name_y + LINE_H, 0xCC_888888, false);
        }

        // Description: one truncated line collapsed, full wrap expanded.
        let desc_y = name_y + LINE_H * 2.0;
        if is_open {
            let mut ly = desc_y;
            for line in wrap(&e.description, desc_chars(row_w)) {
                if text_visible(ly) {
                    d2d.text(&line, name_x, ly, 0xDD_CCCCCC, false);
                }
                ly += LINE_H;
            }
            // Footer: id, source detail
            if text_visible(ly + 2.0) {
                let src = match e.source { ModSource::Yog => "yog mod", ModSource::Platform => "platform mod" };
                d2d.text(&format!("id: {}  ({})", e.id, src), name_x, ly + 2.0, 0x99_777799, false);
            }
        } else if text_visible(desc_y) {
            d2d.text(
                &truncate_chars(&e.description, desc_chars(row_w)),
                name_x, desc_y, 0x99_BBBBBB, false,
            );
        }
    }
    *ROWS.lock().unwrap() = rows;

    // Header redraw — scrolled rows slide under it instead of over it.
    d2d.rect(x0, y0, x0 + w, y0 + HEADER_H, 0xFF_222222);
    d2d.text("Yog Mods", x0 + 8.0, y0 + 6.0, 0xFF_FFD700, true);
    d2d.text(
        &format!("{} mod(s)", entries.len()),
        x0 + w - 80.0, y0 + 6.0, 0xAA_AAAAAA, false,
    );

    // Scrollbar (draggable)
    if max_scroll > 0.0 {
        let track_x0 = x0 + w - 6.0;
        let track_x1 = x0 + w;
        d2d.rect(track_x0, list_y0, track_x1, list_y0 + list_h, 0x44_000000);
        let knob_h = (list_h * (list_h / content_h)).max(16.0);
        let knob_y = list_y0 + (list_h - knob_h) * (scroll / max_scroll);
        let dragging = DRAGGING.lock().unwrap().is_some();
        d2d.rect(track_x0, knob_y, track_x1, knob_y + knob_h,
                 if dragging { 0xFF_BBBBBB } else { 0xAA_888888 });
        *SCROLLBAR.lock().unwrap() =
            Some((track_x0, list_y0, track_x1, list_y0 + list_h, knob_y, knob_y + knob_h));
        *METRICS.lock().unwrap() = (list_y0, list_h, max_scroll, knob_h);
    } else {
        *SCROLLBAR.lock().unwrap() = None;
        *METRICS.lock().unwrap() = (list_y0, list_h, 0.0, 16.0);
    }
}

// ── Event handling ───────────────────────────────────────────────────────────

/// Map a cursor y on the track to a scroll offset while dragging.
fn drag_to_scroll(my: f32, grab: f32) {
    let (list_y0, list_h, max_scroll, knob_h) = *METRICS.lock().unwrap();
    if max_scroll <= 0.0 { return; }
    let denom = (list_h - knob_h).max(1.0);
    let t = ((my - grab - list_y0) / denom).clamp(0.0, 1.0);
    *SCROLL.lock().unwrap() = t * max_scroll;
}

pub fn handle_ui_event(ui_id: &str, event: &str) {
    // Wheel: roughly one collapsed row per notch. Positive dy = wheel up.
    if let Some(dy) = event.strip_prefix("scroll:").and_then(|v| v.parse::<f32>().ok()) {
        let mut s = SCROLL.lock().unwrap();
        *s = (*s - dy * (ROW_H + ROW_GAP)).max(0.0); // upper clamp happens in render
        return;
    }

    let parse_xy = |rest: &str| -> Option<(f32, f32)> {
        let mut it = rest.split(':');
        Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
    };

    if let Some((mx, my)) = event.strip_prefix("click:").and_then(|r| parse_xy(r)) {
        // Scrollbar first: grab the knob, or jump-scroll on the track.
        if let Some((tx0, ty0, tx1, ty1, ky0, ky1)) = *SCROLLBAR.lock().unwrap() {
            if mx >= tx0 && mx <= tx1 && my >= ty0 && my <= ty1 {
                if my >= ky0 && my <= ky1 {
                    *DRAGGING.lock().unwrap() = Some(my - ky0);
                } else {
                    // Center the knob on the click, then keep dragging from there.
                    let knob_h = ky1 - ky0;
                    drag_to_scroll(my, knob_h / 2.0);
                    *DRAGGING.lock().unwrap() = Some(knob_h / 2.0);
                }
                return;
            }
        }
        // Row hit → toggle the description card.
        let rows = ROWS.lock().unwrap();
        if let Some(&(idx, _, _)) = rows.iter().find(|&&(_, ry0, ry1)| my >= ry0 && my <= ry1) {
            drop(rows);
            let mut guard = EXPANDED.lock().unwrap();
            let set = guard.get_or_insert_with(HashSet::new);
            if !set.remove(&idx) {
                set.insert(idx);
            }
        }
        return;
    }

    if let Some((_, my)) = event.strip_prefix("drag:").and_then(|r| parse_xy(r)) {
        let grab = *DRAGGING.lock().unwrap();
        if let Some(grab) = grab {
            drag_to_scroll(my, grab);
        }
        return;
    }

    if event.starts_with("release:") {
        *DRAGGING.lock().unwrap() = None;
        return;
    }

    if event == "close" {
        yog_api::info!("[yog-modlist] UI closed: {}", ui_id);
    }
}
