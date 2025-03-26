use gpui::{
    actions, div, hsla, point, prelude::*, px, App, Application, BoxShadow, FocusHandle, Focusable,
    Global, Hsla, IntoElement, Keystroke, Menu, MenuItem, Modifiers, TitlebarOptions, Window,
    WindowBackgroundAppearance, WindowOptions,
};

actions!(luna, [Quit, ToggleUI]);

pub struct Theme {
    pub background_color: Hsla,
    pub canvas_color: Hsla,
    pub text_color: Hsla,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            background_color: hsla(222.0 / 360.0, 0.12, 0.2, 1.0),
            canvas_color: hsla(224.0 / 360., 0.12, 0.19, 1.0),
            text_color: hsla(0.0, 1.0, 1.0, 1.0),
        }
    }
}

impl Theme {
    pub fn get_global(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }
}

impl Global for Theme {}

#[derive(IntoElement)]
struct Sidebar {}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        let inner = div()
            .h_full()
            .w(px(260.))
            .rounded_tl(px(15.))
            .rounded_bl(px(15.))
            .bg(theme.background_color);

        div()
            .id("titlebar")
            .h_full()
            .w(px(261.))
            .border_r_1()
            .border_color(gpui::white().alpha(0.08))
            .rounded_tl(px(15.))
            .rounded_bl(px(15.))
            .bg(theme.background_color)
            .child(inner)
            .shadow(smallvec::smallvec![BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.24),
                offset: point(px(1.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(0.),
            }])
    }
}

struct Luna {
    hide_sidebar: bool,
    focus_handle: FocusHandle,
}

impl Luna {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Luna {
            hide_sidebar: false,
            focus_handle,
        }
    }

    pub fn hide_sidebar(&mut self, hide: bool) -> &mut Self {
        self.hide_sidebar = hide;
        self
    }

    pub fn toggle_ui(&mut self, _: &ToggleUI, _window: &mut Window, cx: &mut Context<Self>) {
        self.hide_sidebar(!self.hide_sidebar);
        cx.notify();
    }
}

impl Render for Luna {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        div()
            .id("Luna")
            .key_context("luna")
            .track_focus(&self.focus_handle(cx))
            .relative()
            .size_full()
            .flex()
            .font_family("Berkeley Mono")
            .text_xs()
            .bg(theme.background_color)
            .text_color(theme.text_color)
            .border_1()
            .border_color(gpui::white().alpha(0.08))
            .rounded(px(16.))
            .overflow_hidden()
            .on_action(cx.listener(Self::toggle_ui))
            .on_key_down(cx.listener(|this, e: &gpui::KeyDownEvent, window, cx| {
                let toggle_ui_keystroke = Keystroke {
                    modifiers: Modifiers {
                        control: false,
                        alt: false,
                        shift: false,
                        platform: true,
                        function: false,
                    },
                    key: ".".into(),
                    key_char: None,
                };

                if e.keystroke == toggle_ui_keystroke {
                    this.toggle_ui(&ToggleUI::default(), window, cx);
                }

                cx.stop_propagation();
            }))
            .when(self.hide_sidebar, |this| this.child(Sidebar {}))
            .child(div().size_full().flex_1().bg(theme.canvas_color))
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
        cx.set_global(Theme::new());

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
