use std::path::PathBuf;
use std::rc::Rc;

mod modal;

use gpui::{
    App, Bounds, ClickEvent, Context, CursorStyle, Entity, IntoElement, Pixels, Render,
    SharedString, Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_component::button::{Button, ButtonVariant, ButtonVariants};
use gpui_component::dialog::{Dialog, DialogHeader, DialogTitle};
use gpui_component::input::{Input, InputState};
use gpui_component::{
    ActiveTheme, Icon, IconName, Root, Sizable, StyledExt, Theme, ThemeConfig, ThemeMode, ThemeSet,
    WindowExt, h_flex, v_flex,
};
use gpui_component::{ThemeRegistry, TitleBar};
use hyperflash::deck::Deck;
use hyperflash::note::NoteModel;

struct AppState {
    //deck: Deck,
}

impl AppState {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            //deck: Deck { models: vec![NoteModel {}]},
        }
    }
}

impl AppState {
    fn show_add_card(&mut self, e: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        let front = cx.new(|cx| InputState::new(window, cx));
        let back = cx.new(|cx| InputState::new(window, cx));

        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .title("Add Card")
                .backdrop_blur(Pixels::from(10.0))
                .child(
                    v_flex()
                        .gap_3()
                        .child(v_flex().gap_1().child("Front").child(Input::new(&front)))
                        .child(v_flex().gap_1().child("Back").child(Input::new(&back))),
                )
        });
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
                h_flex().child(
                    v_flex()
                        .child(
                            Button::new("home")
                                .with_variant(ButtonVariant::Primary)
                                .child(IconName::Building2)
                                .child("Home")
                                .size_full(),
                        )
                        .child(
                            Button::new("browse")
                                .with_variant(ButtonVariant::Primary)
                                .child(IconName::Search)
                                .child("Browse")
                                .size_full(),
                        )
                        .child(
                            Button::new("statistics")
                                .with_variant(ButtonVariant::Primary)
                                .child(IconName::ChartPie)
                                .child("Statistics")
                                .size_full(),
                        ),
                ),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .justify_center()
                    .items_center()
                    .gap_4()
                    .pt_4()
                    .child(
                        Button::new("Add")
                            .with_variant(ButtonVariant::Primary)
                            .child("Add")
                            .large()
                            .cursor(CursorStyle::PointingHand)
                            .on_click(cx.listener(Self::show_add_card)),
                    ),
            )
            .children(dialog_layer)
    }
}

fn main() {
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
                        &"Mellifluous Light".to_owned(),
                        &"Mellifluous Dark".to_owned(),
                        &ThemeMode::Dark,
                    );

                    let view = cx.new(|cx| AppState::new(cx));
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
