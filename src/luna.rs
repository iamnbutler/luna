use gpui::{
    actions, div, hsla, prelude::*, App, Application, FocusHandle, Focusable, Hsla, IntoElement,
    Menu, MenuItem, Window, WindowOptions,
};

actions!(set_menus, [Quit]);

pub struct Theme {
    pub background_color: Hsla,
    pub text_color: Hsla,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            background_color: hsla(222.0 / 360.0, 0.12, 0.2, 1.0),
            text_color: hsla(0.0, 1.0, 1.0, 1.0),
        }
    }
}

struct Luna {
    theme: Theme,
    focus_handle: FocusHandle,
}

impl Luna {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let theme = Theme::new();

        Luna {
            focus_handle,
            theme,
        }
    }
}

impl Render for Luna {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("Luna")
            .key_context("App")
            .track_focus(&self.focus_handle(cx))
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .font_family("Berkeley Mono")
            .text_xs()
            .bg(self.theme.background_color)
            .text_color(self.theme.text_color)
    }
}

impl Focusable for Luna {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.activate(true);
        cx.on_action(quit);
        cx.set_menus(vec![Menu {
            name: "Luna".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        cx.open_window(WindowOptions::default(), |_window, cx| {
            cx.new(|cx| Luna::new(cx))
        })
        .unwrap();
    });
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}
