//! Annotation Object Scale dialog — add / remove the annotation scales a single
//! selected object carries a per-object representation for.
//!
//! Each drawing scale is a row; a checkmark marks the scales the object is a
//! member of. Clicking a row toggles membership: adding synthesizes a per-scale
//! context (`AcDb*ObjectContextData`) at that scale, removing drops it. Shares
//! the style / scale managers' frame so it looks consistent.

use crate::app::Message;
use crate::ui::style::style_manager::{hdivider, tb_button, BG, BORDER, DIM, LIST, TB, TEXT};
use iced::widget::{column, container, mouse_area, row, scrollable, text, Space};
use iced::{Background, Border, Color, Element, Fill, Theme};

const MEMBER_CHECK: Color = Color {
    r: 0.30,
    g: 0.82,
    b: 0.36,
    a: 1.0,
};

/// `scales` is `(name, "paper:drawing" ratio, is_member)`. Every label is cloned
/// into the widget tree, so the returned element borrows nothing from the args.
pub fn view_window(
    object_label: &str,
    scales: &[(String, String, bool)],
) -> Element<'static, Message> {
    let toolbar = container(
        row![
            text(format!("Object: {object_label}"))
                .size(11)
                .color(TEXT),
            Space::new().width(Fill),
            tb_button("Close", Message::CloseModal, true),
        ]
        .spacing(4)
        .align_y(iced::Center),
    )
    .style(|_: &Theme| container::Style {
        background: Some(Background::Color(TB)),
        ..Default::default()
    })
    .width(Fill)
    .padding([5, 8]);

    let rows: Vec<Element<'_, Message>> = scales
        .iter()
        .map(|(name, ratio, member)| {
            let check = crate::ui::icons::check_cell(*member, MEMBER_CHECK);
            let label = row![
                check,
                text(name.clone()).size(11).color(TEXT).width(Fill),
                text(ratio.clone()).size(10).color(DIM),
            ]
            .spacing(4)
            .align_y(iced::Center);
            let cell = container(label)
                .padding([4, 8])
                .width(Fill)
                .style(move |_: &Theme| container::Style {
                    text_color: Some(TEXT),
                    ..Default::default()
                });
            mouse_area(cell)
                .on_press(Message::AnnoObjectScaleToggle(name.clone()))
                .into()
        })
        .collect();

    let list = container(scrollable(column(rows).spacing(1)).height(Fill))
        .style(|_: &Theme| container::Style {
            background: Some(Background::Color(LIST)),
            border: Border {
                color: BORDER,
                width: 1.0,
                radius: 3.0.into(),
            },
            ..Default::default()
        })
        .width(Fill)
        .height(Fill)
        .padding(2);

    let body = container(
        column![
            text("Click a scale to add or remove the object's representation for it.")
                .size(10)
                .color(DIM),
            list,
        ]
        .spacing(6)
        .height(Fill),
    )
    .width(Fill)
    .height(Fill)
    .padding(12);

    container(column![toolbar, hdivider(), body])
        .style(|_: &Theme| container::Style {
            background: Some(Background::Color(BG)),
            ..Default::default()
        })
        .width(Fill)
        .height(Fill)
        .into()
}
