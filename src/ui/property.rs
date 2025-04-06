//! Property editing components for the inspector panel.
//!
//! Provides reusable UI components for displaying and editing
//! element properties, with support for mixed-value states.

use std::str::FromStr;

use gpui::{
    div, prelude::*, px, Context, Entity, Hsla, IntoElement, ParentElement, Render, Rgba,
    SharedString, Styled, Window,
};

use crate::{
    canvas::LunaCanvas,
    theme::{ActiveTheme, Theme},
    AppState,
};

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
            // First try using the built-in TryFrom<&str> implementation for Rgba
            // which handles hex formats (3, 4, 6, 8 digits with or without #)
            if let Ok(rgba) = Rgba::try_from(color_str.as_ref()) {
                let hsla_color: Hsla = rgba.into();
                return Some(hsla_color);
            }
            
            // If not a hex color, try parsing as rgb/rgba format
            if color_str.starts_with("rgb") {
                // Parse rgb(r, g, b) or rgba(r, g, b, a) format
                let stripped = color_str.trim();
                let rgba_parts: Vec<&str> = if stripped.starts_with("rgba") {
                    // Format: rgba(r, g, b, a)
                    if let Some(content) = stripped.strip_prefix("rgba(").and_then(|s| s.strip_suffix(")")) {
                        content.split(',').collect()
                    } else {
                        return None;
                    }
                } else if stripped.starts_with("rgb") {
                    // Format: rgb(r, g, b)
                    if let Some(content) = stripped.strip_prefix("rgb(").and_then(|s| s.strip_suffix(")")) {
                        content.split(',').collect()
                    } else {
                        return None;
                    }
                } else {
                    return None;
                };

                // Parse rgb values (0-255) and alpha (0.0-1.0)
                if rgba_parts.len() >= 3 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        rgba_parts[0].trim().parse::<u8>(),
                        rgba_parts[1].trim().parse::<u8>(),
                        rgba_parts[2].trim().parse::<u8>(),
                    ) {
                        let a = if rgba_parts.len() >= 4 {
                            rgba_parts[3].trim().parse::<f32>().unwrap_or(1.0)
                        } else {
                            1.0
                        };

                        let rgba = Rgba {
                            r: r as f32 / 255.0,
                            g: g as f32 / 255.0,
                            b: b as f32 / 255.0,
                            a: a.clamp(0.0, 1.0),
                        };
                        let hsla_color: Hsla = rgba.into();
                        return Some(hsla_color);
                    }
                }
            }

            // Add a hex color handler for when the # is missing
            if !color_str.starts_with('#') && !color_str.starts_with("rgb") {
                // Try prefixing with # and parsing again
                let with_hash = format!("#{}", color_str);
                if let Ok(rgba) = Rgba::try_from(with_hash.as_ref()) {
                    let hsla_color: Hsla = rgba.into();
                    return Some(hsla_color);
                }
            }
        }
        
        None
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
                    div()
                        .size(px(9.))
                        .border_color(cx.theme().tokens.inactive_border)
                        .border_1()
                        .bg(color)
                })
                .when(display_value.is_empty(), |this| {
                    this.child(div().flex_1().child(display_value))
                })
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
