use gpui::{
    div, prelude::*, px, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Window,
};

use crate::{canvas::LunaCanvas, theme::Theme, AppState};

use super::property::property_input;

pub const INSPECTOR_WIDTH: f32 = 200.;

pub struct Inspector {
    state: Entity<AppState>,
    canvas: Entity<LunaCanvas>,
}

impl Inspector {
    pub fn new(state: Entity<AppState>, canvas: Entity<LunaCanvas>) -> Self {
        Self { state, canvas }
    }
}

impl Render for Inspector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::new();

        let inner = div()
            .flex()
            .flex_col()
            .h_full()
            .w(px(INSPECTOR_WIDTH))
            .rounded_tr(px(15.))
            .rounded_br(px(15.))
            .child(
                div()
                    .px(px(8.))
                    .py(px(10.))
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .border_color(theme.foreground.alpha(0.06))
                    .border_b_1()
                    .child(property_input("200", "X"))
                    .child(property_input("-2300", "Y"))
                    .child(property_input("", "W"))
                    .child(property_input("-6070", "H")),
            );

        div()
            .id("titlebar")
            .absolute()
            .right_0()
            .top_0()
            .h_full()
            .w(px(INSPECTOR_WIDTH + 1.))
            .cursor_default()
            .rounded_tr(px(15.))
            .rounded_br(px(15.))
            .border_color(theme.foreground.alpha(0.06))
            .border_l_1()
            .bg(theme.background_color)
            .on_click(cx.listener(|_, _, _, cx| {
                cx.stop_propagation();
            }))
            .child(inner)
    }
}
