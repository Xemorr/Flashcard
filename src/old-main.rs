use gpui::{
    div, prelude::*, px, rgb, size, App, Bounds, Context, IntoElement, Render,
    WindowBounds, WindowOptions, MouseButton, Window,
};
use gpui::WindowDecorations::{Client, Server};

#[derive(Clone, Copy, PartialEq)]
enum ReviewState {
    Question,
    Answer,
}

struct AnkiMockup {
    state: ReviewState,
    question: String,
    answer: String,
    deck_name: String,
    cards_remaining: (usize, usize, usize),
}

impl AnkiMockup {
    fn new() -> Self {
        Self {
            state: ReviewState::Question,
            deck_name: "Rust :: GPUI Framework".to_string(),
            question: "What is GPUI's element assembly layout model heavily inspired by?".to_string(),
            answer: "Tailwind CSS. It utilizes composable utility methods like .flex(), .bg(), and .text_color() directly mapped to GPU primitives.".to_string(),
            cards_remaining: (12, 4, 85),
        }
    }

    fn show_answer(&mut self, _cx: &mut Context<Self>) {
        self.state = ReviewState::Answer;
    }

    fn rate_card(&mut self, _cx: &mut Context<Self>) {
        self.state = ReviewState::Question;
    }
}

impl Render for AnkiMockup {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (new_cnt, learn_cnt, due_cnt) = self.cards_remaining;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e24))
            .text_color(rgb(0xe0e0e0))
            // --- TOP NAV BAR ---
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .px_4()
                    .py_2()
                    .bg(rgb(0x16161a))
                    .border_b_1()
                    .border_color(rgb(0x2a2a35))
                    .child(div().font_weight(gpui::FontWeight::BOLD).child(self.deck_name.clone()))
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .text_sm()
                            .child(div().text_color(rgb(0x4fa6ed)).child(format!("New: {}", new_cnt)))
                            .child(div().text_color(rgb(0xde6262)).child(format!("Learn: {}", learn_cnt)))
                            .child(div().text_color(rgb(0x52c41a)).child(format!("Due: {}", due_cnt)))
                    )
            )
            // --- MAIN CARD PANEL ---
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_grow() // FIXED: Pass 1.0 weight parameter
                    .justify_center()
                    .items_center()
                    .p_8()
                    .child(
                        div()
                            .w_full()
                            .max_w(px(600.0)) // FIXED: Explicit width constraint definition
                            .text_center()
                            .text_2xl()
                            .p_4()
                            .child(self.question.clone())
                    )
                    .child(
                        div()
                            .w_full()
                            .max_w(px(600.0)) // FIXED: Explicit width constraint definition
                            .flex()
                            .flex_col()
                            .items_center()
                            .child(if self.state == ReviewState::Answer {
                                div()
                                    .w_full()
                                    .my_6()
                                    .border_t_1()
                                    .border_color(rgb(0x3a3a4a))
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            })
                            .child(if self.state == ReviewState::Answer {
                                div()
                                    .text_center()
                                    .text_xl()
                                    .text_color(rgb(0xb5e2b5))
                                    .px_4()
                                    .child(self.answer.clone())
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            })
                    )
            )
            // --- BOTTOM ACTIONS BAR ---
            .child(
                div()
                    .flex()
                    .justify_center()
                    .items_center()
                    .p_6()
                    .bg(rgb(0x16161a))
                    .border_t_1()
                    .border_color(rgb(0x2a2a35))
                    .child(match self.state {
                        ReviewState::Question => {
                            div()
                                .px_6()
                                .py_2()
                                .bg(rgb(0x31313f))
                                .border_1()
                                .border_color(rgb(0x4a4a5a))
                                .rounded_md()
                                // FIXED: Captures complete 4-argument signature context loop
                                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| this.show_answer(cx)))
                                .child("Show Answer")
                                .into_any_element()
                        }
                        ReviewState::Answer => {
                            div()
                                .flex()
                                .gap_3()
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .bg(rgb(0x5c2525))
                                        .rounded_md()
                                        // FIXED: Captures complete 4-argument signature context loop
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| this.rate_card(cx)))
                                        .child(div().text_center().text_sm().child("Again\n<1m").text_color(rgb(0xff8585)))
                                )
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .bg(rgb(0x5c4025))
                                        .rounded_md()
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| this.rate_card(cx)))
                                        .child(div().text_center().text_sm().child("Hard\n6m").text_color(rgb(0xffd585)))
                                )
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .bg(rgb(0x254a25))
                                        .rounded_md()
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| this.rate_card(cx)))
                                        .child(div().text_center().text_sm().child("Good\n10m").text_color(rgb(0x85ff85)))
                                )
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .bg(rgb(0x253b5c))
                                        .rounded_md()
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| this.rate_card(cx)))
                                        .child(div().text_center().text_sm().child("Easy\n4d").text_color(rgb(0x85d5ff)))
                                )
                                .into_any_element()
                        }
                    })
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
                    title: Some("Anki GPUI Mockup".into()),
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_decorations: Some(
                    Client
                ),
                ..Default::default()
            },
            |_, cx| cx.new(|_| AnkiMockup::new()),
        )
            .unwrap();
    });
}