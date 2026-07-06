# Yog Mod List

An in-game mod browser for [Yog Mod Loader](https://github.com/F000NKKK/Yog-Mod-Loader).
Adds a "Yog Mods" button to the title/pause screen (Fabric) or next to the
platform mod list (Forge/NeoForge) that opens a scrollable list of every
installed mod — both `.yog` mods and, where the host exposes them, the
platform's own Java mods.

Built entirely with Yog's own client-side API (`register_menu_entry`,
`register_ui`/`on_ui_render`, `installed_mods`) — no Java code of its own, and
no registration into the host loader's own mod list.

## Features

- Lists every loaded `.yog` mod (id, name, version, authors, description from
  its `yog.toml`) plus platform mods on Fabric/Forge/NeoForge
- Click a row to expand its full description
- Mouse-wheel and draggable scrollbar for long lists

## Requirements

Built against [Yog Mod Loader](https://github.com/F000NKKK/Yog-Mod-Loader)
0.2.0+ (needs the `installed_mods` API, ABI minor 23+).

## Building

```bash
yog build
```

Produces `artifacts/yog-modlist.yog` — drop it into `<game dir>/yog-mods/`.

## License

AGPL-3.0-only — see [LICENSE](LICENSE).
