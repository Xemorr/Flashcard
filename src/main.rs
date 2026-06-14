use gpui::{
    App, Bounds, Context, IntoElement, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, size,
};
mod titlebar;
use titlebar::PlatformTitleBar;

struct AppState {
    thing: String,
}

impl AppState {
    fn new() -> Self {
        Self {
            thing: "".to_string(),
        }
    }
}

impl Render for AppState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title_bar = cx.new(|_cx| {
                    PlatformTitleBar::new("app-title-bar")
                        .child(div().text_xl().text_color(gpui::rgb(0xffffff)).child("Button 1"))
                        .child(div().text_xl().text_color(gpui::rgb(0xffffff)).child("Button 2"))
                        .child(div().text_xl().text_color(gpui::rgb(0xffffff)).child("Button 3"))
                });
        
        div()
            .size_full()
            .bg(gpui::rgb(0x1e1e1e))
            .flex()
            .flex_col()
            .child(title_bar)
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
                        div()
                            .text_xl()
                            .text_color(gpui::rgb(0xffffff))
                            .child("Decks"),
                    )
                    .child(div().text_xl().text_color(gpui::rgb(0xffffff)).child("Add"))
                    .child(
                        div()
                            .text_xl()
                            .text_color(gpui::rgb(0xffffff))
                            .child("Browse"),
                    )
                    .child(
                        div()
                            .text_xl()
                            .text_color(gpui::rgb(0xffffff))
                            .child("Statistics"),
                    )
                    .child(
                        div()
                            .text_xl()
                            .text_color(gpui::rgb(0xffffff))
                            .child("Settings"),
                    ),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        
        let bounds = Bounds::centered(None, size(px(800.), px(550.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Flash".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|_| AppState::new()),
        )
        .unwrap();
    })
}
