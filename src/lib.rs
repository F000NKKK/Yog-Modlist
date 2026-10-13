//! Yog Mod List — in-game mod browser.
//!
//! Adds a "Yog Mods" button to the title screen (Fabric) / mod list screen
//! (Forge, NeoForge) that opens the mod browser: `mod_list.yui` (the
//! layout, edited in Yog-IDLE's screen designer) plus its code-behind
//! `mod_list.rs`.

mod components;
mod mod_list;

// `crate::res` — typed accessors for resources/*.yres (Strings::...).
yog_api::resources!();

use res::Strings;
use yog_api::{info, Mod, Registry};

pub struct YogModList;

impl Mod for YogModList {
    fn register(registry: &mut Registry) {
        info!("[yog-modlist] initializing mod list browser...");
        mod_list::register(registry);
        // The host injects the button into the right vanilla screen; it
        // takes a translation key, so the label follows the game language.
        registry.register_menu_entry(Strings::MENU_BUTTON_KEY, mod_list::ID);
        info!("[yog-modlist] ready.");
    }
}

yog_api::export_mod!(YogModList);
