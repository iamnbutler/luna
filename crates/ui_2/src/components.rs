//! Basic UI components.

use gpui::{
    div, prelude::*, px, ClickEvent, Div, ElementId, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window,
};
use theme_2::Theme;

/// Horizontal stack layout.
pub fn h_stack() -> Div {
    div().flex().flex_row().items_center()
}

/// Vertical stack layout.
pub fn v_stack() -> Div {
    div().flex().flex_col()
}

/// A panel container with background and border.
pub fn panel(theme: &Theme) -> Div {
    div()
        .bg(theme.ui_background)
        .border_1()
        .border_color(theme.ui_border)
        .rounded(px(8.0))
        .p(px(8.0))
}

/// A simple button.
pub fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    theme: &Theme,
) -> impl IntoElement {
    let label = label.into();
    let bg = theme.ui_background;
    let border = theme.ui_border;
    let text = theme.ui_text;
    let hover_bg = theme_2::hsla(0.0, 0.0, 0.95, 1.0);

    div()
        .id(id)
        .px(px(12.0))
        .py(px(6.0))
        .bg(bg)
        .border_1()
        .border_color(border)
        .rounded(px(4.0))
        .text_color(text)
        .text_sm()
        .cursor_pointer()
        .hover(|d| d.bg(hover_bg))
        .child(label)
}

/// An icon button (small, square).
pub fn icon_button(
    id: impl Into<ElementId>,
    icon: impl Into<SharedString>,
    theme: &Theme,
) -> impl IntoElement {
    let icon = icon.into();
    let bg = theme.ui_background;
    let border = theme.ui_border;
    let text = theme.ui_text;
    let hover_bg = theme_2::hsla(0.0, 0.0, 0.95, 1.0);

    div()
        .id(id)
        .size(px(24.0))
        .flex()
        .items_center()
        .justify_center()
        .bg(bg)
        .border_1()
        .border_color(border)
        .rounded(px(4.0))
        .text_color(text)
        .text_xs()
        .cursor_pointer()
        .hover(|d| d.bg(hover_bg))
        .child(icon)
}
