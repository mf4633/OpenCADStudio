//! Attribute editor dialog — edit the attribute values of a single block
//! reference (INSERT). Opened by double-clicking a block that carries
//! attributes, or by the ATTEDIT command with such a block selected.
//!
//! Each attribute is one row: its tag (read-only) and an editable value box.
//! OK writes every value back to the block; Cancel (the ✕ or button) discards.
//! The heavy lifting — applying edits, undo, repaint — lives in the update
//! handler (`Message::AttrEditorOk`); this module is pure layout.

use crate::app::Message;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Background, Border, Color, Element, Length, Theme};

const BG: Color = Color { r: 0.15, g: 0.15, b: 0.15, a: 1.0 };
const WHITE: Color = Color { r: 0.92, g: 0.92, b: 0.92, a: 1.0 };
const DIM: Color = Color { r: 0.55, g: 0.55, b: 0.55, a: 1.0 };
const ACCENT: Color = Color { r: 0.30, g: 0.62, b: 0.95, a: 1.0 };
const FIELD_BG: Color = Color { r: 0.10, g: 0.10, b: 0.10, a: 1.0 };
const BORDER: Color = Color { r: 0.32, g: 0.32, b: 0.32, a: 1.0 };

fn field_style(_t: &Theme, _s: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(FIELD_BG),
        border: Border { color: BORDER, width: 1.0, radius: 3.0.into() },
        icon: WHITE,
        placeholder: DIM,
        value: WHITE,
        selection: ACCENT,
    }
}

/// Build the attribute editor dialog body. `block` is the reference's block
/// name (shown as a subtitle); `fields` are the `(tag, value)` pairs in
/// attribute order — the row index is the routing key back to
/// `Message::AttrEditorInput`.
pub fn view_window<'a>(block: &'a str, fields: &'a [(String, String)]) -> Element<'a, Message> {
    let mut list = column![].spacing(6);
    for (idx, (tag, value)) in fields.iter().enumerate() {
        let value_box = text_input("", value)
            .on_input(move |v| Message::AttrEditorInput { idx, value: v })
            .on_submit(Message::AttrEditorOk)
            .style(field_style)
            .size(13)
            .padding([4, 6])
            .width(Length::Fill);

        let attr_row = row![
            container(text(tag.as_str()).size(13).color(WHITE))
                .width(170)
                .padding([4, 6]),
            value_box,
        ]
        .spacing(8)
        .align_y(iced::Center);
        list = list.push(attr_row);
    }

    let body: Element<'_, Message> = if fields.is_empty() {
        text("This block has no attributes.")
            .size(13)
            .color(DIM)
            .into()
    } else {
        scrollable(list).height(Length::Fill).into()
    };

    let ok = button(text("OK").size(13).color(WHITE))
        .padding([5, 22])
        .on_press(Message::AttrEditorOk)
        .style(|_: &Theme, status| {
            let bg = if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                Color { r: 0.32, g: 0.55, b: 0.85, a: 1.0 }
            } else {
                ACCENT
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: WHITE,
                border: Border { radius: 4.0.into(), ..Default::default() },
                ..Default::default()
            }
        });

    let cancel = button(text("Cancel").size(13).color(WHITE))
        .padding([5, 18])
        .on_press(Message::CloseModal)
        .style(|_: &Theme, status| {
            let bg = if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                Color { r: 0.28, g: 0.28, b: 0.28, a: 1.0 }
            } else {
                Color { r: 0.20, g: 0.20, b: 0.20, a: 1.0 }
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: WHITE,
                border: Border { color: BORDER, width: 1.0, radius: 4.0.into() },
                ..Default::default()
            }
        });

    let header = column![
        text("Edit Attributes").size(18).color(WHITE),
        text(format!("Block: {block}")).size(12).color(DIM),
    ]
    .spacing(2);

    container(
        column![
            header,
            Space::new().height(10),
            body,
            Space::new().height(12),
            row![
                Space::new().width(Length::Fill),
                cancel,
                Space::new().width(8),
                ok
            ]
            .align_y(iced::Center),
        ]
        .padding(4),
    )
    .style(|_: &Theme| container::Style {
        background: Some(Background::Color(BG)),
        ..Default::default()
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(14)
    .into()
}
