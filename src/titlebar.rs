use gpui::{
    AnyElement, Context, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement,
    Render, StatefulInteractiveElement, Styled, Window, div, px,
};
use smallvec::SmallVec;
use std::mem;

pub struct PlatformTitleBar {
    id: ElementId,
    children: SmallVec<[AnyElement; 2]>,
    should_move: bool,
}

impl PlatformTitleBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            children: SmallVec::new(),
            should_move: false,
        }
    }
}

impl Render for PlatformTitleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let children = mem::take(&mut self.children);

        // We give the root container its own ID so GPUI treats the titlebar as an active input receiver
        div()
            .id("titlebar-container")
            .w_full()
            .h(px(40.0))
            .bg(gpui::rgb(0x1e1e1e))
            .flex()
            .flex_row()
            .items_center()
            .px_4()
            // --- Custom Mouse Drag Handlers ---
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.should_move = true;
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.should_move = false;
                }),
            )
            .on_mouse_down_out(cx.listener(|this, _, _, _| {
                this.should_move = false;
            }))
            .on_mouse_move(cx.listener(|this, _, window, _| {
                if this.should_move {
                    this.should_move = false;
                    window.start_window_move();
                }
            }))
            // --- LEFT SIDE BALANCE ---
            .child(div().w(px(52.0)))
            // --- CENTER SECTION ---
            .child(
                div()
                    .flex()
                    .flex_1()
                    .justify_center()
                    .items_center()
                    .gap_4()
                    .children(children),
            )
            // --- RIGHT SIDE: Traffic Light Controls ---
            .child(
                div()
                    .id("controls-wrapper")
                    .flex()
                    .flex_row()
                    .gap_2()
                    .w(px(52.0))
                    .justify_end()
                    .child(
                        // Minimize Button (Yellow)
                        div()
                            .id("win-minimize")
                            .size(px(12.0))
                            .rounded_full()
                            .bg(gpui::rgb(0xffbd2e))
                            .on_click(|_, window, _| {
                                window.minimize_window();
                            }),
                    )
                    .child(
                        // Maximize Button (Green)
                        div()
                            .id("win-maximize")
                            .size(px(12.0))
                            .rounded_full()
                            .bg(gpui::rgb(0x27c93f))
                            .on_click(|_, window, _| {
                                window.zoom_window();
                            }),
                    )
                    .child(
                        // Close Button (Red)
                        div()
                            .id("win-close")
                            .size(px(12.0))
                            .rounded_full()
                            .bg(gpui::rgb(0xff5f56))
                            .on_click(|_, _, cx| {
                                cx.quit();
                            }),
                    ),
            )
    }
}

impl ParentElement for PlatformTitleBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}
