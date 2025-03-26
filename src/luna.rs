use gpui::{
    actions, div, hsla, linear_color_stop, linear_gradient, point, prelude::*, px, App,
    Application, FocusHandle, Focusable, Hsla, IntoElement, Menu, MenuItem, TitlebarOptions,
    Window, WindowBackgroundAppearance, WindowOptions,
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

#[derive(IntoElement)]
struct Titlebar {}

impl RenderOnce for Titlebar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id("titlebar")
            .h(px(30.))
            .w_full()
            .border_b_1()
            .border_color(gpui::white().alpha(0.08))
            .bg(linear_gradient(
                180.,
                linear_color_stop(gpui::white().alpha(0.02), 0.0),
                linear_color_stop(gpui::transparent_white(), 1.0),
            ))
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
            .border_1()
            .border_color(gpui::white().alpha(0.08))
            .child(Titlebar {})
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

        cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("Luna".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(8.0), px(8.0))),
                }),
                window_background: WindowBackgroundAppearance::Transparent,
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| Luna::new(cx)),
        )
        .unwrap();
    });
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}
