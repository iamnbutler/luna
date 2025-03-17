use canvas::{Canvas, LunaElement};
use gpui::*;

struct Luna {
    canvas: Entity<Canvas>,
}

impl Render for Luna {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x2e7d32))
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(self.canvas.clone())
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            let canvas = cx.new(|_cx| Canvas::default());
            let element_1 = LunaElement::default();
            let element_2 = LunaElement::default();

            canvas.update(cx, |canvas, _| {
                canvas.add_element(element_1, point(px(0.), px(0.)));
                canvas.add_element(element_2, point(px(300.), px(300.)));
            });

            cx.new(|_cx| Luna { canvas })
        })
        .unwrap();
    });
}
