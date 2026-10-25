<div align="center">

<a id="top"></a>

# 📋 Yog Mod List

### An in-game mod browser for the Yog mod loader

[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.2.0-informational.svg)](yog.toml)
[![yog-kit 0.15](https://img.shields.io/badge/yog--kit-0.15-orange.svg)](https://crates.io/crates/yog-kit)
[![Minecraft](https://img.shields.io/badge/Minecraft-1.20.1%20%7C%201.21.1%20%7C%2026.2-62B47A.svg)](https://github.com/F000NKKK/Yog-Mod-Loader)
[![Loaders](https://img.shields.io/badge/loaders-Fabric%20%7C%20NeoForge%20%7C%20Forge-8A2BE2.svg)](https://github.com/F000NKKK/Yog-Mod-Loader)

🇺🇸 **English** | 🇷🇺 [Русский](README.ru.md)

[Overview](#-overview) • [Features](#-features) • [How It's Built](#-how-its-built) • [Building](#-building) • [License](#-license)

</div>

---

## 📖 Overview

**Yog Mod List** adds a **Yog Mods** button to the game: on the title screen
with Fabric, on the mod list screen with Forge and NeoForge. It opens a
browser of every installed mod: `.yog` mods and, where the loader exposes
them, the platform's own Java mods.

It is a pure [Yog](https://github.com/F000NKKK/Yog-Mod-Loader) mod: Rust and
`.yui` markup, no Java of its own. It is also the reference example of a
view-model screen built with `yog-kit`.

<p align="center"><img src="docs/images/mod-list.png" alt="The mod browser as Yog-IDLE's screen designer draws it: search box, All/Yog/Platform filters, the list and the details pane" width="760"></p>

## ✨ Features

- 🔎 **Search** by name, id or description as you type
- 🏷️ **Filters**: All, Yog, Platform (a filter with nothing to show is disabled)
- 📜 **List** with Yog mods first, then by name; mouse wheel, scrollbar and keyboard
- 📝 **Details pane**: name, version, source, authors, id and full description
- 🌍 **English and Russian** UI that follows the game language

## 🧩 How It's Built

| File | What it is |
|---|---|
| [`src/lib.rs`](src/lib.rs) | Registers the screen and the menu button (`register_menu_entry`) |
| [`src/mod_list.yui`](src/mod_list.yui) | The screen layout: a `Grid` with the header, search, filters, `ListView` and details pane |
| [`src/mod_list.rs`](src/mod_list.rs) | `ModListVm`, the view model: loads `installed_mods()` on open, filtering, search, selection, plus a headless `TestHost` test |
| [`resources/Strings.yres`](resources/Strings.yres), [`Strings.ru_ru.yres`](resources/Strings.ru_ru.yres) | Texts in English and Russian; the mod name and description in `yog.toml` point here too (`@Strings.mod_name`) |
| [`resources/Colors.yres`](resources/Colors.yres) | The source colors of the details pane (`Colors::SOURCE_YOG`, `Colors::SOURCE_PLATFORM`) |

Bindings in the markup call straight into the view model:

```xml
<TextBox text="{x:Bind search, Mode=TwoWay}" placeholder="@Strings.search.placeholder" />
<ToggleButton text="@Strings.filter.yog"
              checked="{x:Bind filter == Filter::Yog}"
              enabled="{x:Bind has(Filter::Yog)}"
              command="{x:Bind set_filter(Filter::Yog)}" />
<ListView items="{x:Bind visible()}" selected="{x:Bind selected_id(), BindBack=select}" />
```

All files open in [Yog-IDLE](https://github.com/F000NKKK/Yog-IDLE): the
screen and components in the designer, the `.yres` files in the resource
table. The formats are described in Yog-IDLE's docs:
[`.yui`](https://github.com/F000NKKK/Yog-IDLE/blob/master/docs/yui.md),
[view models](https://github.com/F000NKKK/Yog-IDLE/blob/master/docs/mvvm.md),
[`.yres`](https://github.com/F000NKKK/Yog-IDLE/blob/master/docs/yres.md).

## 🔨 Building

You need the `yog` CLI (`cargo install yog-cli`).

```bash
yog build              # -> bin/yog-modlist.yog
```

Put the `.yog` into `<game dir>/yog-mods/` next to the Yog loader jar.

To try it in a dev game, clone
[Yog-Mod-Loader](https://github.com/F000NKKK/Yog-Mod-Loader) next to this
repo and run it on one of the loader's targets (they come from the loader SDK and
`~/.yog/local.toml`, not from the mod):

```bash
yog run fabric 1.21.1  # every loader and version: yog run --list
```

The view-model test runs without the game: `cargo test` in `obj/`
after a build.

## 📄 License

**AGPL-3.0-only**, see [LICENSE](LICENSE).

---

<div align="center">

**[⬆ Back to Top](#top)**

Part of the [Yog](https://github.com/F000NKKK/Yog-Mod-Loader) project

</div>
