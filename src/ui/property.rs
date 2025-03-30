use gpui::{
    div, prelude::*, px, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Window,
};

use crate::{canvas::LunaCanvas, theme::Theme, AppState};

pub fn property_input(value: Option<Vec<f32>>, icon: impl Into<SharedString>) -> PropertyInput {
    PropertyInput::new(value, icon)
}

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
        let theme = Theme::new();

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
                .bg(theme.foreground.alpha(0.06))
                .text_color(theme.foreground.alpha(0.9))
                .when(no_value || mixed, |this| {
                    this.text_color(theme.foreground.alpha(0.4))
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
