mod deckbuilding;
mod settings;

use gpui::{
    Action, App, Bounds, ClickEvent, Context, CursorStyle, IntoElement, Pixels, Render, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_component::TitleBar;
use gpui_component::button::{Button, DropdownButton};
use gpui_component::input::{Input, InputState};
use gpui_component::resizable::{h_resizable, resizable_panel};
use gpui_component::{ActiveTheme, IconName, Root, Theme, ThemeMode, WindowExt, h_flex, v_flex};

use crate::settings::Settings;

struct AppState {
    settings: Settings,
    deck: hyperflash::deck::ManagedDeck,
    basic_model: hyperflash::deck::model::Model,
}

impl AppState {
    fn new(settings: Settings) -> Self {
        Self {
            settings,
            basic_model: deckbuilding::basic_model(),
            deck: deckbuilding::open_or_init_deck("./decks/default.deck"),
        }
    }
}

impl AppState {
    fn show_add_card(&mut self, _e: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        deckbuilding::show_add_card(cx.entity().clone(), window, cx);
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
            settings::init_theme(cx);
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
