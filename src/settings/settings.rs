use std::{cell::{Cell, RefCell}, rc::Rc};

use facet::{Facet, Type, UserType, PrimitiveType};
use facet_reflect::Peek;
use gpui::{
    Action, App, AppContext, Entity, ParentElement, Styled, Window, px,
};
use gpui_component::{
    button::{Button, ButtonVariants, DropdownButton},
    input::{Input, InputEvent, InputState},
    h_flex, v_flex, ActiveTheme, Selectable, WindowExt,
};
use serde::{Deserialize, Serialize};

use super::theme::{DarkThemeName, LightThemeName, ThemeMode, ThemeSettings, apply_current_theme};
use crate::AppState;

#[derive(Facet, Default, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub theme: ThemeSettings,
    #[serde(default)]
    pub scheduler: SchedulerSettings,
}

impl Settings {
    pub fn save_to_disk(&self) {
        let toml_string = toml::to_string_pretty(self).unwrap_or_default();
        let _ = std::fs::write("settings.toml", toml_string);
    }

    fn update_field_by_path(&mut self, path: &str, value: &str) {
        if path == "theme.experimental_feature" {
            self.theme.experimental_feature = value.to_string();
        }
    }

    fn update_enum_by_path(&mut self, path: &str, variant_index: usize) {
        if path == "theme.mode" {
            self.theme.mode = if variant_index == 0 {
                ThemeMode::Light
            } else {
                ThemeMode::Dark
            };
        }
    }

    fn update_runtime_choice_by_path(&mut self, path: &str, value: &str) {
        if path == "theme.light_theme" {
            self.theme.light_theme = LightThemeName(value.to_string());
        } else if path == "theme.dark_theme" {
            self.theme.dark_theme = DarkThemeName(value.to_string());
        }
    }
}

/// A field whose valid choices aren't known at compile time (e.g. they depend on
/// what got hot-loaded into `ThemeRegistry`). Implementors pair a `String`-backed
/// wrapper type with the logic to compute its option list, so the *type* of a
/// `Settings` field is enough to drive the dropdown UI — the same way a compile-time
/// `Facet` enum's variants drive its dropdown, just resolved at render time instead
/// of derive time.
pub trait RuntimeChoiceKind: 'static {
    fn options(cx: &App) -> Vec<String>;
}

/// Declares a `String`-backed newtype whose choices come from the themes loaded into
/// `ThemeRegistry` at runtime, filtered by `$filter`. Add a new one of these (and a
/// matching arm in `Settings::update_runtime_choice_by_path`) to add another
/// runtime-populated dropdown field.
macro_rules! runtime_choice_field {
    ($name:ident, |$theme:ident| $filter:expr) => {
        #[derive(Clone, PartialEq, Default, Serialize, Deserialize, Facet)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl RuntimeChoiceKind for $name {
            fn options(cx: &App) -> Vec<String> {
                ThemeRegistry::global(cx)
                    .sorted_themes()
                    .into_iter()
                    .filter(|$theme| $filter)
                    .map(|theme| theme.name.to_string())
                    .collect()
            }
        }
    };
}
pub(crate) use runtime_choice_field;

/// Placeholder for scheduler settings; empty for now, exists only so "Scheduler"
/// shows up as its own sidebar tab in the settings dialog.
#[derive(Clone, PartialEq, Serialize, Deserialize, Default, Facet)]
pub struct SchedulerSettings {}

#[derive(Clone, serde::Deserialize, PartialEq)]
pub struct SelectEnumVariant {
    pub field_path: String,
    pub variant_name: String,
    pub variant_index: usize,
}

impl Action for SelectEnumVariant {
    fn boxed_clone(&self) -> Box<dyn Action> { Box::new(self.clone()) }
    fn partial_eq(&self, other: &dyn Action) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |a| a == self)
    }
    fn name(&self) -> &'static str { "SelectEnumVariant" }
    fn name_for_type() -> &'static str { "SelectEnumVariant" }
    fn build(value: gpui::private::serde_json::Value) -> gpui::Result<Box<dyn Action>> {
        Ok(Box::new(gpui::private::serde_json::from_value::<Self>(value)?))
    }
}

#[derive(Clone, serde::Deserialize, PartialEq)]
pub struct SelectRuntimeChoice {
    pub field_path: String,
    pub value: String,
}

impl Action for SelectRuntimeChoice {
    fn boxed_clone(&self) -> Box<dyn Action> { Box::new(self.clone()) }
    fn partial_eq(&self, other: &dyn Action) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |a| a == self)
    }
    fn name(&self) -> &'static str { "SelectRuntimeChoice" }
    fn name_for_type() -> &'static str { "SelectRuntimeChoice" }
    fn build(value: gpui::private::serde_json::Value) -> gpui::Result<Box<dyn Action>> {
        Ok(Box::new(gpui::private::serde_json::from_value::<Self>(value)?))
    }
}

struct SettingsUiState {
    input_states: Vec<(String, Entity<InputState>)>,
}

pub fn settings_view(settings: Settings, entity: Entity<AppState>, window: &mut Window, cx: &mut App) {
    let mut ui_state = SettingsUiState {
        input_states: Vec::new(),
    };

    // Find all string fields in Settings and create InputStates
    build_input_states(Peek::new(&settings), &mut ui_state, window, cx, "");

    let settings_for_render = Rc::new(RefCell::new(settings.clone()));
    let selected_tab = Rc::new(Cell::new(0usize));

    for (path, input_state) in &ui_state.input_states {
        let path = path.clone();
        let entity_clone = entity.clone();
        let settings_for_update = settings_for_render.clone();
        cx.subscribe(input_state, move |state, _: &InputEvent, cx| {
            let value = state.read(cx).value().to_string();
            settings_for_update.borrow_mut().update_field_by_path(&path, &value);
            entity_clone.update(cx, |this, cx| {
                this.settings.update_field_by_path(&path, &value);
                this.settings.save_to_disk();
                apply_current_theme(this, cx);
            });
        }).detach();
    }

    let entity_for_dialog = entity.clone();
    let selected_tab_for_dialog = selected_tab.clone();

    window.open_dialog(cx, move |dialog, _window, cx| {
        let entity_for_action = entity_for_dialog.clone();
        let settings_for_action = settings_for_render.clone();

        cx.on_action(move |action: &SelectEnumVariant, cx| {
            settings_for_action.borrow_mut().update_enum_by_path(&action.field_path, action.variant_index);
            entity_for_action.update(cx, |state, cx| {
                state.settings.update_enum_by_path(&action.field_path, action.variant_index);
                state.settings.save_to_disk();
                apply_current_theme(state, cx);
                cx.notify();
            });
        });

        let entity_for_runtime_choice = entity_for_dialog.clone();
        let settings_for_runtime_choice = settings_for_render.clone();

        cx.on_action(move |action: &SelectRuntimeChoice, cx| {
            settings_for_runtime_choice.borrow_mut().update_runtime_choice_by_path(&action.field_path, &action.value);
            entity_for_runtime_choice.update(cx, |state, cx| {
                state.settings.update_runtime_choice_by_path(&action.field_path, &action.value);
                state.settings.save_to_disk();
                apply_current_theme(state, cx);
                cx.notify();
            });
        });

        let current_settings = settings_for_render.borrow();

        // Each top-level field of `Settings` (e.g. `theme`) becomes a sidebar tab, so
        // fields render under their section without a repeated "theme." prefix.
        let mut tabs: Vec<(String, Peek)> = Vec::new();
        let root_peek = Peek::new(&*current_settings);
        if let Type::User(UserType::Struct(s_ty)) = root_peek.shape().ty {
            if let Ok(s) = root_peek.into_struct() {
                for (i, field) in s_ty.fields.iter().enumerate() {
                    if let Ok(field_peek) = s.field(i) {
                        tabs.push((field.name.to_string(), field_peek));
                    }
                }
            }
        }

        let selected_index = selected_tab_for_dialog.get().min(tabs.len().saturating_sub(1));

        let mut sidebar = v_flex()
            .gap_1()
            .p_2()
            .w(px(140.))
            .border_r_1()
            .border_color(cx.theme().border);

        for (i, (name, _)) in tabs.iter().enumerate() {
            let tab_entity = entity_for_dialog.clone();
            let tab_state = selected_tab_for_dialog.clone();
            sidebar = sidebar.child(
                Button::new(("settings-tab", i))
                    .label(capitalize(name))
                    .ghost()
                    .selected(i == selected_index)
                    .on_click(move |_, _window, cx| {
                        tab_state.set(i);
                        tab_entity.update(cx, |_, cx| cx.notify());
                    }),
            );
        }

        let mut content = v_flex().gap_3().p_4().flex_1();
        if let Some((tab_name, field_peek)) = tabs.into_iter().nth(selected_index) {
            content = render_shape(field_peek, &ui_state, content, &tab_name, cx);
        }

        dialog
            .title("App Settings")
            .child(h_flex().items_start().child(sidebar).child(content))
    });
}

/// True if `peek` is one of the `RuntimeChoiceKind` wrapper types (e.g. `LightThemeName`),
/// which render as a dropdown fed by runtime state rather than a free-text `Input`.
fn is_runtime_choice(peek: &Peek) -> bool {
    peek.get::<LightThemeName>().is_ok() || peek.get::<DarkThemeName>().is_ok()
}

/// If `peek` is one of the `RuntimeChoiceKind` wrapper types, returns its current value
/// plus the options computed from current runtime state (e.g. loaded themes).
fn runtime_choice_options(peek: &Peek, cx: &App) -> Option<(String, Vec<String>)> {
    if let Ok(v) = peek.get::<LightThemeName>() {
        return Some((v.0.clone(), LightThemeName::options(cx)));
    }
    if let Ok(v) = peek.get::<DarkThemeName>() {
        return Some((v.0.clone(), DarkThemeName::options(cx)));
    }
    None
}

fn build_input_states(peek: Peek, ui_state: &mut SettingsUiState, window: &mut Window, cx: &mut App, path: &str) {
    if is_runtime_choice(&peek) {
        return;
    }
    match peek.shape().ty {
        Type::User(UserType::Struct(s_ty)) => {
            if let Ok(s) = peek.into_struct() {
                for (i, field) in s_ty.fields.iter().enumerate() {
                    let field_name = field.name;
                    let field_path = if path.is_empty() { field_name.to_string() } else { format!("{}.{}", path, field_name) };
                    if let Ok(field_peek) = s.field(i) {
                        build_input_states(field_peek, ui_state, window, cx, &field_path);
                    }
                }
            }
        }
        Type::Primitive(PrimitiveType::Textual(_)) | Type::User(UserType::Opaque) => {
            if let Ok(val_str) = peek.get::<String>() {
                let val_str = val_str.clone();
                let input_state = cx.new(|cx| {
                    let mut state = InputState::new(window, cx);
                    state.insert(val_str, window, cx);
                    state
                });
                ui_state.input_states.push((path.to_string(), input_state));
            }
        }
        _ => {}
    }
}

/// The dialog groups fields into sidebar tabs keyed by their top-level field name (see
/// `settings_view`), so labels only need to show a field's own name, not its full
/// dotted `field_path` (which is still used, unabridged, for the update actions).
fn display_label(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or(path)
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn render_shape(peek: Peek, ui_state: &SettingsUiState, mut container: gpui::Div, path: &str, cx: &App) -> gpui::Div {
    if let Some((current, options)) = runtime_choice_options(&peek, cx) {
        let field_path = path.to_string();

        return container.child(
            DropdownButton::new(path.to_string())
                .button(Button::new(path.to_string()).label(format!("{}: {}", display_label(path), current)))
                .dropdown_menu(move |menu, _, _| {
                    let mut menu = menu;
                    for name in &options {
                        let field_path = field_path.clone();
                        let value = name.clone();
                        menu = menu.menu(name.clone(), Box::new(SelectRuntimeChoice { field_path, value }));
                    }
                    menu
                }),
        );
    }

    match peek.shape().ty {
        Type::User(UserType::Struct(s_ty)) => {
            if let Ok(s) = peek.into_struct() {
                for (i, field) in s_ty.fields.iter().enumerate() {
                    let field_name = field.name;
                    let field_path = if path.is_empty() { field_name.to_string() } else { format!("{}.{}", path, field_name) };
                    if let Ok(field_peek) = s.field(i) {
                        container = render_shape(field_peek, ui_state, container, &field_path, cx);
                    }
                }
            }
        }
        Type::Primitive(PrimitiveType::Textual(_)) | Type::User(UserType::Opaque) => {
            if let Some((_, input_state)) = ui_state.input_states.iter().find(|(p, _)| p == path) {
                container = container.child(v_flex().gap_1().child(display_label(path).to_string()).child(Input::new(input_state)));
            }
        }
        Type::User(UserType::Enum(e_ty)) => {
            if let Ok(e) = peek.into_enum() {
                if let Ok(current_variant_index) = e.variant_index() {
                    let current_variant_name = e.variant_name(current_variant_index).unwrap_or("Unknown");

                    let field_path = path.to_string();
                    let variants: Vec<(String, usize)> = e_ty.variants.iter().enumerate().map(|(i, v)| (v.name.to_string(), i)).collect();

                    container = container.child(
                        DropdownButton::new(path.to_string())
                            .button(Button::new(path.to_string()).label(format!("{}: {}", display_label(path), current_variant_name)))
                            .dropdown_menu(move |menu, _, _| {
                                let mut menu = menu;
                                for (name, index) in &variants {
                                    let field_path = field_path.clone();
                                    let variant_name = name.clone();
                                    let variant_index = *index;
                                    menu = menu.menu(name.clone(), Box::new(SelectEnumVariant {
                                        field_path: field_path.clone(),
                                        variant_name: variant_name.clone(),
                                        variant_index,
                                    }));
                                }
                                menu
                            }),
                    );
                }
            }
        }
        _ => {}
    }
    container
}
