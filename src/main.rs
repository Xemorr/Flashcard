use std::path::PathBuf;

mod modal;
mod settings;

use gpui::{
    Action, App, Bounds, ClickEvent, Context, CursorStyle, IntoElement, Pixels, Render,
    SharedString, Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_component::button::{Button, DropdownButton};
use gpui_component::input::{Input, InputState};
use gpui_component::resizable::{h_resizable, resizable_panel};
use gpui_component::{ActiveTheme, IconName, Root, Theme, ThemeMode, WindowExt, h_flex, v_flex};
use gpui_component::{ThemeRegistry, TitleBar};

use crate::settings::Settings;

#[derive(Clone, PartialEq, Action, serde::Deserialize)]
pub struct SelectNoteType;

struct AppState {
    settings: Settings,
}

impl AppState {
    fn new(settings: Settings) -> Self {
        Self { settings }
    }
}

impl AppState {
    fn show_add_card(&mut self, _e: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let front = cx.new(|cx| InputState::new(window, cx));
        let back = cx.new(|cx| InputState::new(window, cx));
        let tags = cx.new(|cx| InputState::new(window, cx));

        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .title("Add Card")
                .backdrop_blur(Pixels::from(10.0))
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            h_flex()
                                .gap_1()
                                .justify_around()
                                .child(
                                    DropdownButton::new("Note Type")
                                        .button(
                                            Button::new("Note Type")
                                                .label("Note Type")
                                                .on_click(|_event, _window, _cx| {}),
                                        )
                                        .dropdown_menu(|menu, _, _| {
                                            menu.menu("Basic", Box::new(SelectNoteType))
                                                .menu(
                                                    "Basic (and reversed)",
                                                    Box::new(SelectNoteType),
                                                )
                                                .menu("Gap Fill (Cloze)", Box::new(SelectNoteType))
                                        }),
                                )
                                .child(
                                    DropdownButton::new("Deck")
                                        .button(Button::new("Deck").label("Deck"))
                                        .dropdown_menu(|menu, _, _| {
                                            menu.menu("Option 1", Box::new(SelectNoteType))
                                        }),
                                ),
                        )
                        .child(v_flex().gap_1().child("Front").child(Input::new(&front)))
                        .child(v_flex().gap_1().child("Back").child(Input::new(&back)))
                        .child(
                            v_flex()
                                .gap_1()
                                .child("Tags (comma-separated)")
                                .child(Input::new(&tags)),
                        ),
                )
        });
    }

    fn settings(&mut self, _e: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        settings::settings_view(self.settings.clone(), cx.entity().clone(), window, cx);
    }

    fn titlebar(cx: &mut Context<Self>) -> TitleBar {
        TitleBar::new().child(
            h_flex()
                .gap_1()
                .child(
                    div()
                        .id("title")
                        .child("Kaizen")
                        .text_color(cx.theme().foreground),
                )
                .child(
                    div()
                        .id("theme-toggle")
                        .child(if cx.theme().is_dark() {
                            IconName::Moon
                        } else {
                            IconName::Sun
                        })
                        .text_color(cx.theme().foreground)
                        .on_click(|_event, window, cx| {
                            let mode = if cx.theme().is_dark() {
                                ThemeMode::Light
                            } else {
                                ThemeMode::Dark
                            };
                            Theme::change(mode, Some(window), cx);
                        }),
                ),
        )
    }
}

impl Render for AppState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dialog_layer = Root::render_dialog_layer(window, cx);

        div()
            .size_full()
            .bg(cx.theme().background)
            .flex()
            .flex_col()
            .child(Self::titlebar(cx))
            .child(
                h_resizable("panels")
                    .child(
                        resizable_panel()
                            .min_size(Pixels::from(100.0))
                            .size(Pixels::from(200.0))
                            .child(
                                v_flex()
                                    .size_full()
                                    .items_stretch()
                                    .px_4()
                                    .py_4()
                                    .gap_3()
                                    .child(
                                        Button::new("home")
                                            .child(IconName::Building2)
                                            .label("Home")
                                            .cursor(CursorStyle::PointingHand),
                                    )
                                    .child(
                                        Button::new("browse")
                                            .child(IconName::Search)
                                            .label("Browse")
                                            .cursor(CursorStyle::PointingHand),
                                    )
                                    .child(
                                        Button::new("statistics")
                                            .child(IconName::ChartPie)
                                            .label("Statistics")
                                            .cursor(CursorStyle::PointingHand),
                                    )
                                    .child(
                                        Button::new("settings")
                                            .child(IconName::Settings)
                                            .label("Settings")
                                            .cursor(CursorStyle::PointingHand)
                                            .on_click(cx.listener(Self::settings)),
                                    ),
                            ),
                    )
                    .child(
                        resizable_panel()
                            .size(Pixels::from(400.0))
                            .min_size(Pixels::from(200.0))
                            .child(div().rounded_b_2xl().bg(cx.theme().sidebar_accent)),
                    )
                    .child(
                        resizable_panel().child(
                            v_flex()
                                .size_full()
                                .items_stretch()
                                .px_4()
                                .py_4()
                                .gap_3()
                                .child(
                                    Button::new("Add")
                                        .label("Add Card")
                                        .cursor(CursorStyle::PointingHand)
                                        .on_click(cx.listener(Self::show_add_card)),
                                ),
                        ),
                    ),
            )
            .children(dialog_layer)
    }
}

fn main() {
    let settings =
        toml::from_str::<Settings>(&std::fs::read_to_string("settings.toml").unwrap_or_default())
            .unwrap_or_else(|_| Settings::default());

    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(move |cx: &mut App| {
            gpui_component::init(cx);
            init_theme(cx);
            let bounds = Bounds::centered(None, size(px(800.), px(550.0)), cx);

            let _ = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitleBar::title_bar_options()),
                    is_resizable: true,
                    is_movable: true,
                    is_minimizable: true,
                    window_decorations: Some(gpui::WindowDecorations::Server),
                    ..Default::default()
                },
                |window, cx| {
                    let _app_state = cx.new(|_| AppState::new(settings.clone()));

                    return cx.new(|cx| Root::new(_app_state, window, cx));
                },
            );
        })
}

pub fn init_theme(cx: &mut App) {
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
        let settings = toml::from_str::<Settings>(
            &std::fs::read_to_string("settings.toml").unwrap_or_default(),
        )
        .unwrap_or_else(|_| Settings::default());
        apply_theme(
            cx,
            &settings.theme.light_theme,
            &settings.theme.dark_theme,
            match settings.theme.mode {
                settings::ThemeMode::Light => ThemeMode::Light,
                settings::ThemeMode::Dark => ThemeMode::Dark,
            },
        );
    }) {
        println!("Failed to watch themes directory: {}", err)
    }
}

#[derive(Clone, Action, serde::Deserialize, PartialEq)]
pub struct ThemeUpdated;

pub fn apply_theme(
    cx: &mut App,
    light_mode_theme: &String,
    dark_mode_theme: &String,
    theme_mode: ThemeMode,
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
