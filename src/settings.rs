use std::{cell::Cell, rc::Rc};

use facet::Facet;
use gpui::{
    Action, App, AppContext, Entity, ParentElement, Styled, Window
};
use gpui_component::{
    button::{Button, DropdownButton},
    input::{Input, InputEvent, InputState},
    v_flex, WindowExt,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Facet, Default, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub theme: ThemeSettings,
}

impl Settings {
    pub fn save_to_disk(&self) {
        let toml_string = toml::to_string_pretty(self).unwrap_or_default();
        let _ = std::fs::write("settings.toml", toml_string);
    }
}

#[derive(Serialize, Deserialize, Default, Facet, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Default, Facet)]
pub struct ThemeSettings {
    pub light_theme: String,
    pub dark_theme: String,
    pub mode: ThemeMode,
}


#[derive(Clone, Action, serde::Deserialize, PartialEq)]
pub struct SelectLightTheme;

#[derive(Clone, Action, serde::Deserialize, PartialEq)]
pub struct SelectDarkTheme;

pub fn settings_view(settings: Settings, entity: Entity<AppState>, window: &mut Window, cx: &mut App) {
    let initial_settings = settings.clone();
    let light_theme_input = cx.new(|cx| {
        let mut state = InputState::new(window, cx);
        state.insert(initial_settings.theme.light_theme.clone(), window, cx);
        state
    });

    let dark_theme_input = cx.new(|cx| {
        let mut state = InputState::new(window, cx);
        state.insert(initial_settings.theme.dark_theme.clone(), window, cx);
        state
    });

    let entity_clone = entity.clone();
    cx.subscribe(&light_theme_input, move |state, _event: &InputEvent, cx| {
        let value = state.read(cx).value().to_string();
        entity_clone.update(cx, |this, cx| {
            this.settings.theme.light_theme = value;
            this.settings.save_to_disk();
            crate::apply_theme(
                cx,
                &this.settings.theme.light_theme,
                &this.settings.theme.dark_theme,
                match this.settings.theme.mode {
                    ThemeMode::Light => gpui_component::ThemeMode::Light,
                    ThemeMode::Dark => gpui_component::ThemeMode::Dark,
                },
            );
        });
    })
    .detach();

    let entity_clone = entity.clone();
    cx.subscribe(&dark_theme_input, move |state, _event: &InputEvent, cx| {
        let value = state.read(cx).value().to_string();
        entity_clone.update(cx, |this, cx| {
            this.settings.theme.dark_theme = value;
            this.settings.save_to_disk();
            crate::apply_theme(
                cx,
                &this.settings.theme.light_theme,
                &this.settings.theme.dark_theme,
                match this.settings.theme.mode {
                    ThemeMode::Light => gpui_component::ThemeMode::Light,
                    ThemeMode::Dark => gpui_component::ThemeMode::Dark,
                },
            );
        });
    })
    .detach();

    let entity_for_dialog = entity.clone();
    let initial_mode = settings.theme.mode;
    let mode_cell = Rc::new(Cell::new(initial_mode));

    window.open_dialog(cx, move |dialog, _window, cx| {
        // We can't safely read or update AppState here because this closure is called
        // during the render pass of AppState, which already holds an update lease.
        
        // Instead of reading the mode from the entity each time, we can use a local
        // variable in the closure that we update when actions occur.
        
        // We use a RefCell-like approach but shared via Rc to allow the actions to update it
        // and the render pass to read it.
        let mode_for_render = mode_cell.clone();

        let entity_clone = entity_for_dialog.clone();
        let mode_for_light = mode_cell.clone();
        cx.on_action(move |_: &SelectLightTheme, cx| {
            mode_for_light.set(ThemeMode::Light);
            entity_clone.update(cx, |state, cx| {
                state.settings.theme.mode = ThemeMode::Light;
                state.settings.save_to_disk();
                crate::apply_theme(
                    cx,
                    &state.settings.theme.light_theme,
                    &state.settings.theme.dark_theme,
                    gpui_component::ThemeMode::Light,
                );
                cx.notify();
            });
        });

        let entity_clone = entity_for_dialog.clone();
        let mode_for_dark = mode_cell.clone();
        cx.on_action(move |_: &SelectDarkTheme, cx| {
            mode_for_dark.set(ThemeMode::Dark);
            entity_clone.update(cx, |state, cx| {
                state.settings.theme.mode = ThemeMode::Dark;
                state.settings.save_to_disk();
                crate::apply_theme(
                    cx,
                    &state.settings.theme.light_theme,
                    &state.settings.theme.dark_theme,
                    gpui_component::ThemeMode::Dark,
                );
                cx.notify();
            });
        });

        let current_mode = mode_for_render.get();
        let current_variant_name = match current_mode {
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
        };

        dialog.title("App Settings").child(
            v_flex()
                .gap_3()
                .p_4()
                .child(
                    v_flex()
                        .gap_1()
                        .child("light_theme")
                        .child(Input::new(&light_theme_input)),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child("dark_theme")
                        .child(Input::new(&dark_theme_input)),
                )
                .child(
                    DropdownButton::new("mode")
                        .button(
                            Button::new("mode").label(format!("mode: {}", current_variant_name)),
                        )
                        .dropdown_menu({
                            move |menu, _, _| {
                                menu.menu("Light", Box::new(SelectLightTheme))
                                    .menu("Dark", Box::new(SelectDarkTheme))
                            }
                        }),
                ),
        )
    });
}

