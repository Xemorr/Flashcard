use std::path::PathBuf;

use facet::Facet;
use gpui::{App, SharedString};
use gpui_component::{Theme, ThemeRegistry};
use serde::{Deserialize, Serialize};

use super::settings::{RuntimeChoiceKind, Settings, runtime_choice_field};
use crate::AppState;

#[derive(Serialize, Deserialize, Default, Facet, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

runtime_choice_field!(LightThemeName, |theme| !theme.mode.is_dark());
runtime_choice_field!(DarkThemeName, |theme| theme.mode.is_dark());

#[derive(Clone, PartialEq, Serialize, Deserialize, Default, Facet)]
pub struct ThemeSettings {
    pub light_theme: LightThemeName,
    pub dark_theme: DarkThemeName,
    pub mode: ThemeMode,
    pub experimental_feature: String,
}

pub fn apply_current_theme(state: &AppState, cx: &mut App) {
    apply_theme(
        cx,
        &state.settings.theme.light_theme.0,
        &state.settings.theme.dark_theme.0,
        match state.settings.theme.mode {
            ThemeMode::Light => gpui_component::ThemeMode::Light,
            ThemeMode::Dark => gpui_component::ThemeMode::Dark,
        },
    );
}

pub fn init_theme(cx: &mut App) {
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
        let settings = toml::from_str::<Settings>(
            &std::fs::read_to_string("settings.toml").unwrap_or_default(),
        )
        .unwrap_or_else(|_| Settings::default());
        apply_theme(
            cx,
            &settings.theme.light_theme.0,
            &settings.theme.dark_theme.0,
            match settings.theme.mode {
                ThemeMode::Light => gpui_component::ThemeMode::Light,
                ThemeMode::Dark => gpui_component::ThemeMode::Dark,
            },
        );
    }) {
        println!("Failed to watch themes directory: {}", err)
    }
}

pub fn apply_theme(
    cx: &mut App,
    light_mode_theme: &String,
    dark_mode_theme: &String,
    theme_mode: gpui_component::ThemeMode,
) {
    let light_mode_theme = SharedString::from(light_mode_theme.clone());
    let dark_mode_theme = SharedString::from(dark_mode_theme.clone());

    if let Some(light_theme) = ThemeRegistry::global(cx)
        .themes()
        .get(&light_mode_theme)
        .cloned()
    {
        Theme::global_mut(cx).apply_config(&light_theme);
    }
    if let Some(dark_theme) = ThemeRegistry::global(cx)
        .themes()
        .get(&dark_mode_theme)
        .cloned()
    {
        Theme::global_mut(cx).apply_config(&dark_theme);
    }
    Theme::change(theme_mode, None, cx);
}
