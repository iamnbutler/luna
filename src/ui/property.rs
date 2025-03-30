use gpui::{
    div, prelude::*, px, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Window,
};

use crate::{canvas::LunaCanvas, theme::Theme, AppState};

pub fn property_input(
    value: impl Into<SharedString>,
    icon: impl Into<SharedString>,
) -> PropertyInput {
    PropertyInput::new(value, icon)
}

#[derive(IntoElement)]
pub struct PropertyInput {
    value: SharedString,
    icon: SharedString,
}

impl PropertyInput {
    pub fn new(value: impl Into<SharedString>, icon: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            icon: icon.into(),
        }
    }
}

impl RenderOnce for PropertyInput {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = Theme::new();
        let no_value = self.value.is_empty();

        div().flex().flex_row().child(
            div()
                .flex()
                .items_center()
                .flex_none()
                .pl(px(6.))
                .pr(px(4.))
                .w(px(64.))
                .rounded(px(4.))
                .bg(theme.foreground.alpha(0.06))
                .text_color(theme.foreground.alpha(0.9))
                .when(no_value, |this| {
                    this.text_color(theme.foreground.alpha(0.4))
                })
                .text_size(px(11.))
                .child(div().flex_1().child(self.value))
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
