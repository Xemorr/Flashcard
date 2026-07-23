# CLAUDE.md

## What this is

A desktop flashcard app ("Kaizen", an Anki-style app) built on **GPUI** (Zed's UI framework, via the `gpui-ce` fork) and the **gpui-component** widget library. The app itself is very early-stage: the main window currently just wires up a three-pane resizable layout (nav sidebar / deck list / card editor placeholders) plus an "Add Card" dialog and a Settings dialog. Flashcard logic will eventually use the `hyperflash` crate.

## Build/run commands

- `cargo run` — build and launch the app. Must be run from the repo root (it reads `settings.toml` and watches `./themes` using paths relative to the working directory).
- `cargo check` — fast type-check, works offline once `Cargo.lock` is resolved.
- `cargo build` — full build.
- There are no automated tests in this repo yet.
- Toolchain is pinned via `rust-toolchain.toml` to `nightly` with `rustfmt`, `clippy`, `miri`, `rust-analyzer`, `rust-src` components — cargo will auto-select this toolchain.

## Architecture

### Module wiring — check `main.rs` before trusting a file exists

`src/main.rs` declares `mod modal; mod settings;`. `src/settings.rs` is the live, reflection-driven settings implementation — an earlier hand-written version was superseded in place during a refactor (see git history: "missed settings" → "Working settings, not created via reflection" → "Experimental meta programming settings menu"). There is no separate `settings2.rs`; if you see that name mentioned anywhere (old notes, stale docs), treat it as referring to what is now `src/settings.rs`. `src/modal.rs` is currently an empty stub. Always grep `main.rs`'s `mod` list before assuming a `src/*.rs` file is live.

### Settings: reflection-driven UI (`src/settings.rs`)

This is the most structurally interesting part of the codebase. Instead of hand-writing a settings dialog field-by-field, `settings.rs` uses the `facet`/`facet-reflect` crates to walk the `Settings` struct's shape at runtime and generate UI automatically:

- `Settings` derives `Facet` (in addition to `Serialize`/`Deserialize` for TOML round-tripping to `settings.toml`).
- `build_input_states` walks the struct via `Peek`, recursing into nested structs and creating a `gpui_component::input::InputState` for every string-typed leaf field, keyed by a dotted `field_path` (e.g. `"theme.light_theme"`).
- `render_shape` does a second walk over the same shape to lay out the actual dialog UI: plain string fields render as `Input`, enum fields render as a `DropdownButton` whose menu items dispatch a generic `SelectEnumVariant { field_path, variant_name, variant_index }` action.
- **Runtime-populated dropdowns** (e.g. `light_theme`/`dark_theme`): a compile-time `Facet` enum can only offer choices baked in by `#[derive(Facet)]`, but the set of loaded themes isn't known until `ThemeRegistry` finishes loading `./themes/*.json`. These fields use `String`-backed newtypes (`LightThemeName`, `DarkThemeName`) that implement `RuntimeChoiceKind`, declared via the `runtime_choice_field!` macro. Each one's `options(cx)` filters `ThemeRegistry::global(cx).sorted_themes()` by the theme's actual `mode` (light/dark, read from the theme JSON — not string-matched off the name). `render_shape`/`build_input_states` recognize these wrapper types via `peek.get::<T>()` and render them as a `DropdownButton` instead of an `Input`, following the same type-drives-the-widget pattern as compile-time enums. Writes go through a matching `SelectRuntimeChoice` action / `update_runtime_choice_by_path`. `#[serde(transparent)]` keeps the `settings.toml` representation a bare string. To add another runtime-populated field: add a `runtime_choice_field!` invocation, add it to `is_runtime_choice`/`runtime_choice_options`, and add a branch in `update_runtime_choice_by_path`.
- Settings updates flow: input change → `cx.subscribe` handler on that field's `InputState` → `Settings::update_field_by_path` (or `update_enum_by_path`/`update_runtime_choice_by_path` for enum/runtime-choice fields) mutates both a local `Rc<RefCell<Settings>>` (used for redrawing the currently-open dialog) and the real `AppState.settings` (via the passed-in `Entity<AppState>`) → `save_to_disk()` (rewrites `settings.toml`) → `apply_current_theme` re-applies the theme live.
- **Important limitation**: `update_field_by_path`/`update_enum_by_path`/`update_runtime_choice_by_path` are still manually matched string comparisons (`if path == "theme.light_theme"`) — the reflection only generates the *UI*, not the write-back path. Adding a new settings field means adding it to the `Settings`/`ThemeSettings` struct **and** adding a matching branch in the relevant update function(s).
- Dialogs in gpui-component can't safely borrow/update the owning `Entity` from inside their own render closure (it's already mid-update). The workaround used throughout `settings.rs` is to mirror mutable state into an `Rc<Cell<...>>`/`Rc<RefCell<...>>` that the dialog closure reads for rendering, while actions update both that shared cell and the real entity via `entity.update(cx, ...)`.

### Theming

- Runtime themes are JSON files in `./themes/` (e.g. `ayu.json`, `molokai.json`), loaded via `gpui_component::ThemeRegistry::watch_dir` in `init_theme` (`main.rs`) — editing a theme JSON while the app runs hot-reloads it. Each theme JSON can declare multiple named theme entries (e.g. `ayu.json` has both "Ayu Light" and "Ayu Dark"), each tagged with its own `mode`.
- `settings.toml` stores which theme name to use for light mode vs. dark mode (`theme.light_theme` / `theme.dark_theme`, backed by the `LightThemeName`/`DarkThemeName` runtime-choice types described above) plus the active `theme.mode` (Light/Dark enum).
- `apply_theme` (`main.rs`) is the single place that pushes `(light_theme, dark_theme, mode)` into `gpui_component::Theme::global_mut`/`Theme::change`. Both the initial load (`init_theme`) and every settings-dialog edit call back into this same function so theme state never drifts from `settings.toml`.

### Dependencies of note

- `gpui` / `gpui_platform` come from `gpui-ce/gpui-ce` (a community fork of Zed's GPUI), not upstream `zed-industries/zed` — there's a `[patch]` block in `Cargo.toml` redirecting any transitive `zed-industries/zed` gpui reference to the same fork. Keep this patch in sync if bumping the gpui-ce rev.
- `gpui-component` / `gpui-component-assets` are pinned to a specific rev (`277f220`) — bump both together.
- `facet` / `facet-reflect` / `facet-toml` are the reflection stack backing `settings.rs`. Note that actual TOML load/save (`main.rs`, `Settings::save_to_disk`) goes through the plain `toml` + `serde` crates, not `facet-toml` — `facet`/`facet-reflect` are only used for the runtime struct-walking that drives the settings UI.
- `hyperflash` is the flashcard-logic dependency; not yet wired into any code path.
