//! Property editing components for the inspector panel.
//!
//! Provides reusable UI components for displaying and editing
//! element properties, with support for mixed-value states.

use std::str::FromStr;

use gpui::{
    div, prelude::*, px, Context, Entity, Hsla, IntoElement, ParentElement, Render, Rgba,
    SharedString, Styled, Window,
};

use canvas::{AppState, LunaCanvas};
use theme::{ActiveTheme, Theme};

/// Creates a new property input field with the given value and icon
pub fn float_input(value: Option<Vec<f32>>, icon: impl Into<SharedString>) -> PropertyInput {
    PropertyInput::new(value, icon)
}

/// Input field for numeric property values with support for mixed states
///
/// Displays property values with appropriate formatting based on selection state:
/// - No value: Empty field
/// - Single value: Shows the exact value
/// - Multiple different values: Shows "Mixed"
#[derive(IntoElement)]
pub struct PropertyInput {
    value: Option<Vec<f32>>,
    icon: SharedString,
}

impl PropertyInput {
    pub fn new(value: Option<Vec<f32>>, icon: impl Into<SharedString>) -> Self {
        Self {
            value,
            icon: icon.into(),
        }
    }
}

impl RenderOnce for PropertyInput {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = Theme::default();

        // Convert Option<Vec<f32>> to display string
        let display_value = match &self.value {
            None => SharedString::from(""),
            Some(values) if values.is_empty() => SharedString::from("Mixed"),
            Some(values) if values.len() == 1 => SharedString::from(format!("{}", values[0])),
            Some(_) => SharedString::from("Mixed"),
        };

        let no_value = display_value.is_empty();
        let mixed = display_value == "Mixed";

        div().flex().flex_row().child(
            div()
                .flex()
                .items_center()
                .flex_none()
                .pl(px(6.))
                .pr(px(4.))
                .w(px(84.))
                .rounded(px(4.))
                .bg(theme.tokens.surface0)
                .text_color(theme.tokens.text)
                .when(no_value || mixed, |this| {
                    this.text_color(theme.tokens.text.alpha(0.5))
                })
                .text_size(px(11.))
                .child(div().flex_1().child(display_value))
                .child(
                    div()
                        .flex()
                        .justify_center()
                        .flex_none()
                        .overflow_hidden()
                        .w(px(11.))
                        .h_full()
                        .child(self.icon),
                ),
        )
    }
}

#[derive(IntoElement)]
pub struct ColorInput {
    value: Option<SharedString>,
    opacity: Option<f32>,
    icon: SharedString,
}

impl ColorInput {
    pub fn new(value: Option<SharedString>, icon: SharedString) -> Self {
        Self {
            value,
            opacity: None,
            icon,
        }
    }

    pub fn parse_color(&self) -> Option<Hsla> {
        if let Some(color_str) = &self.value {
            luna_core::color::parse_color(color_str)
        } else {
            None
        }
    }
}

impl RenderOnce for ColorInput {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();

        let parsed_color = self.parse_color();
        let mut display_value = String::new();
        if let Some(color) = parsed_color {
            display_value = color.to_string();
        };

        div().flex().flex_row().child(
            div()
                .flex()
                .items_center()
                .flex_none()
                .pl(px(6.))
                .pr(px(4.))
                .w_full()
                .rounded(px(4.))
                .bg(theme.tokens.surface0)
                .text_color(theme.tokens.text)
                .text_size(px(11.))
                .when_some(parsed_color, |this, color| {
                    this.child(
                        div()
                            .size(px(9.))
                            .border_color(cx.theme().tokens.inactive_border)
                            .border_1()
                            .rounded(px(2.))
                            .bg(color),
                    )
                })
                .when(!display_value.is_empty(), |this| {
                    this.child(div().flex_1().ml(px(4.)).child(display_value))
                })
                .child(div().flex_1())
                .child(
                    div()
                        .flex()
                        .justify_center()
                        .flex_none()
                        .overflow_hidden()
                        .w(px(11.))
                        .h_full()
                        .child(self.icon),
                ),
        )
    }
}
