use gpui::{
    App, AppContext, ClickEvent, Context, IntoElement, ParentElement, Pixels, Render, Styled,
    Window,
};
use gpui_component::{
    ThemeMode, WindowExt,
    button::{Button, DropdownButton},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub theme: ThemeSettings,
}

impl Settings {
    pub fn save_to_disk(&self, cx: &mut App) {
        let toml_string = toml::to_string_pretty(self).unwrap_or_default();

        cx.background_spawn(async move {
            let _ = std::fs::write("settings.toml", toml_string);
        })
        .detach();
    }
}

pub trait EditableSetting {
    fn render_setting(
        &self,
        label: &'static str,
        window: &mut Window,
        cx: &mut App,
        settings: &mut Settings,
        on_change: impl Fn(&mut Settings, &mut App) + 'static,
    ) -> impl IntoElement;
}
/*
impl EditableSetting for String {
    fn render_setting(
        &self,
        label: &'static str,
        window: &mut Window,
        cx: &mut App,
        settings: &mut Settings,
        on_change: impl Fn(&mut Settings, &mut App) + 'static,
    ) -> impl IntoElement {

        let input_state = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.insert(self.clone(), window, cx);
            state
        });

        // Whenever the user types, update the struct and save
        let on_change = std::sync::Arc::new(on_change);

        let input_state_clone = input_state.clone();
        cx.subscribe(
            &input_state_clone,
            |_input_state_clone, _event: &InputEvent, cx| {
                let read = input_state.read(cx);
                read.value();
                on_change(settings, cx);
                settings.save_to_disk(cx);
            },
        )
        .detach();

        v_flex()
            .gap_1()
            .child(label)
            .child(Input::new(&input_state_clone))
    }
}
*/

#[derive(Serialize, Deserialize)]
pub struct ThemeSettings {
    pub light_theme: String,
    pub dark_theme: String,
    pub mode: ThemeMode,
}
/*
pub fn settings_view(e: &ClickEvent, window: &mut Window, cx: &mut App) {
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
*/
