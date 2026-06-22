use std::path::PathBuf;
use std::rc::Rc;

mod modal;
mod settings;

use gpui::{
    Action, Anchor, App, Bounds, ClickEvent, Context, CursorStyle, Entity, IntoElement, Pixels,
    Render, SharedString, Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_component::button::{Button, ButtonVariant, ButtonVariants, DropdownButton};
use gpui_component::dialog::{Dialog, DialogHeader, DialogTitle};
use gpui_component::group_box::GroupBox;
use gpui_component::input::{Input, InputState};
use gpui_component::menu::{DropdownMenu, PopupMenu};
use gpui_component::plot::Grid;
use gpui_component::resizable::{ResizablePanel, h_resizable, resizable_panel};
use gpui_component::{
    ActiveTheme, Icon, IconName, Root, Sizable, StyledExt, Theme, ThemeConfig, ThemeMode, ThemeSet,
    WindowExt, h_flex, v_flex,
};
use gpui_component::{ThemeRegistry, TitleBar};
use hyperflash::deck::Deck;
use hyperflash::note::NoteModel;

use crate::settings::Settings;

#[derive(Clone, PartialEq, Action)]
struct SelectNoteType;

struct AppState {
    settings: Settings,
}

impl AppState {
    fn new(settings: Settings) -> Self {
        Self { settings }
    }
}

impl AppState {
    fn show_add_card(&mut self, e: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
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
                                                .on_click(|e, window, cx| {}),
                                        )
                                        .dropdown_menu(|menu, _, _| {
                                            menu.menu("Option 1", Box::new(SelectNoteType))
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

    fn settings(&mut self, e: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {}

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
                        .on_click(|event, window, cx| {
                            Theme::change(
                                if cx.theme().is_dark() {
                                    ThemeMode::Light
                                } else {
                                    ThemeMode::Dark
                                },
                                Some(window),
                                cx,
                            );
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
                                            .cursor(CursorStyle::PointingHand), //.on_click(settings_view),
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
            .unwrap();

    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(|cx: &mut App| {
            gpui_component::init(cx);
            let bounds = Bounds::centered(None, size(px(800.), px(550.0)), cx);

            cx.open_window(
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
                    apply_theme(
                        cx,
                        &settings.theme.light_theme,
                        &settings.theme.dark_theme,
                        &settings.theme.mode,
                    );

                    let view = cx.new(|_| AppState::new(settings));
                    return cx.new(|cx| Root::new(view, window, cx));
                },
            )
            .unwrap();
        })
}

pub fn apply_theme(
    cx: &mut App,
    light_mode_theme: &String,
    dark_mode_theme: &String,
    theme_mode: &ThemeMode,
) {
    let light_mode_theme = SharedString::from(light_mode_theme);
    let dark_mode_theme = SharedString::from(dark_mode_theme);
    let theme_mode = theme_mode.clone();
    // Load and watch themes from ./themes directory
    if let Err(err) = ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
        if let Some(light_mode_theme) = ThemeRegistry::global(cx)
            .themes()
            .get(&light_mode_theme)
            .cloned()
        {
            Theme::global_mut(cx).apply_config(&light_mode_theme);
        }
        if let Some(dark_mode_theme) = ThemeRegistry::global(cx)
            .themes()
            .get(&dark_mode_theme)
            .cloned()
        {
            Theme::global_mut(cx).apply_config(&dark_mode_theme);
        }
        // Theme::change must be called from within watch_dir, otherwise race condition fuckery happens.
        Theme::change(theme_mode, None, cx);
    }) {
        println!("Failed to watch themes directory: {}", err)
    }
}

fn stage_theme(cx: &mut App, theme_to_stage: &Rc<ThemeConfig>) {
    Theme::global_mut(cx).apply_config(&theme_to_stage);
}

fn theme_to_string(theme_mode: &ThemeMode) -> String {
    match theme_mode {
        ThemeMode::Light => "Light".to_owned(),
        ThemeMode::Dark => "Dark".to_owned(),
    }
}
