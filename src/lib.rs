//! Yog Mod List — in-game mod browser.
//!
//! Registers a "Yog Mods" button on the title screen / mod-list screen
//! (via `register_menu_entry`) and opens a yog-ui screen listing loaded mods.
//!
//! # Architecture
//! - `register_menu_entry("Yog Mods", "yog:modlist")` — adds the button on
//!   vanilla screens (TitleScreen on Fabric, ModListScreen on Forge/NeoForge).
//! - `on_ui_render("yog:modlist", ...)` — renders the mod-list UI inside a
//!   `YogUIScreen` (darkened background).
//! - `register_ui("yog:modlist", ...)` — receives click events forward from
//!   the Java-side mouse handler.

mod ui;

use yog_api::{info, Mod, Registry};

pub struct YogModList;

impl Mod for YogModList {
    fn register(registry: &mut Registry) {
        info!("[yog-modlist] initializing mod list browser...");

        // Build a static (placeholder) list for now. A future ABI minor will
        // add `registry.mods()` to query runtime metadata from the loader.
        let entries = ui::gather_mod_entries();
        let n = entries.len();

        // Register the UI tree + render + click handler.
        let ui_id = "yog:modlist";
        registry.register_ui(ui_id, move |uid, event| {
            ui::handle_ui_event(uid, event);
        });
        registry.on_ui_render(ui_id, move |gfx| {
            ui::render_mod_list(gfx, &entries);
        });

        // Register the menu button — host injects it into the right screen.
        registry.register_menu_entry("Yog Mods", ui_id);

        info!("[yog-modlist] ready. {} mods listed.", n);
    }
}

yog_api::export_mod!(YogModList);
