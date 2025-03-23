use gpui::{prelude::FluentBuilder as _, *};

pub const TITLEBAR_HEIGHT: f32 = 24.0;

pub struct Titlebar {
    title: SharedString,
}

impl Titlebar {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let title = "Untitled".into();
        Titlebar { title }
    }
}

impl Render for Titlebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .h(px(TITLEBAR_HEIGHT))
            .border_b_1()
            .border_color(rgb(0x3F434C))
            .bg(rgb(0x2A2C31))
            .child(div().flex().items_center().h_full().px_2().child("Luna"))
    }
}
