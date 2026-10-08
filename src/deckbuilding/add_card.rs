use std::{borrow::Cow};

use gpui::{Action, App, AppContext, ClickEvent, Context, Entity, ParentElement, Pixels, Styled, Window};
use gpui_component::{
    WindowExt, button::{Button, ButtonVariants, DropdownButton}, h_flex, input::{Input, InputState}, notification::Notification, v_flex
};
use hyperflash::{deck::note::{Field, Note}, error::DeckError};

use crate::AppState;

pub fn show_add_card(entity: Entity<AppState>, window: &mut Window, cx: &mut App) {
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
                                            .menu("Basic (and reversed)", Box::new(SelectNoteType))
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
            .footer(
                Button::new("add").label("Add").primary().on_click({
                    let front = front.clone();
                    let back = back.clone();
                    let tags = tags.clone();
                    let entity = entity.clone();
                    move |_, window, cx| {
                        let front_text = front.read(cx).value().to_string();
                        let back_text = back.read(cx).value().to_string();
                        let tag_list: Vec<String> = tags
                            .read(cx)
                            .value()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();

                        let result: Result<(), DeckError> = entity.update(cx, |state, _cx| {
                            let note = Note {
                                model: Cow::Borrowed(&state.basic_model),
                                tags: tag_list,
                                fields: vec![
                                    Field { name: "Front".into(), content: &front_text },
                                    Field { name: "Back".into(), content: &back_text }
                                ],
                                references: vec![]
                            };
                            state.deck.add_card(&note)
                        });

                        match result {
                            Ok(()) => window.push_notification(Notification::success("Card added"), cx),
                            Err(err) => window.push_notification(Notification::error(err.to_string()), cx),
                        }
                        window.close_dialog(cx);
                    }
                })
            )
    });
}

#[derive(Clone, PartialEq, Action, serde::Deserialize)]
pub struct SelectNoteType;
