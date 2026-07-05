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

        // Register the UI tree + render + click handler. The actual mod list
        // is queried from the loader lazily on first render (see ui.rs) so it
        // includes mods that load after this one.
        let ui_id = "yog:modlist";
        registry.register_ui(ui_id, move |uid, event| {
            ui::handle_ui_event(uid, event);
        });
        registry.on_ui_render(ui_id, move |gfx| {
            ui::render_mod_list(gfx, ui::mod_entries());
        });

        // Register the menu button — host injects it into the right screen.
        registry.register_menu_entry("Yog Mods", ui_id);

        info!("[yog-modlist] ready.");
    }
}

yog_api::export_mod!(YogModList);
