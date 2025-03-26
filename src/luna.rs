use std::{fs, path::PathBuf};

use gpui::{
    actions, div, hsla, point, prelude::*, px, svg, App, Application, AssetSource, BoxShadow,
    FocusHandle, Focusable, Global, Hsla, IntoElement, Keystroke, Menu, MenuItem, Modifiers,
    SharedString, TitlebarOptions, Window, WindowBackgroundAppearance, WindowOptions,
};

use anyhow::Result;
use strum::{EnumIter, IntoEnumIterator as _};

actions!(luna, [Quit, ToggleUI]);

const TITLEBAR_HEIGHT: f32 = 31.;

pub struct Theme {
    pub background_color: Hsla,
    pub canvas_color: Hsla,
    pub foreground: Hsla,
    pub foreground_muted: Hsla,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            background_color: hsla(222.0 / 360.0, 0.12, 0.2, 1.0),
            canvas_color: hsla(224.0 / 360., 0.12, 0.19, 1.0),
            foreground: hsla(0.0, 1.0, 1.0, 1.0),
            foreground_muted: hsla(0.0, 1.0, 1.0, 0.6),
        }
    }
}

impl Theme {
    pub fn get_global(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }
}

impl Global for Theme {}

#[derive(EnumIter)]
pub enum ToolKind {
    ArrowPointer,
    Frame,
    Image,
    Pencil,
    Shapes,
    Square,
    TextCursor,
}

impl ToolKind {
    pub fn src(self) -> SharedString {
        match self {
            ToolKind::ArrowPointer => "svg/arrow_pointer.svg".into(),
            ToolKind::Frame => "svg/frame.svg".into(),
            ToolKind::Image => "svg/image.svg".into(),
            ToolKind::Pencil => "svg/pencil.svg".into(),
            ToolKind::Shapes => "svg/shapes.svg".into(),
            ToolKind::Square => "svg/square.svg".into(),
            ToolKind::TextCursor => "svg/text_cursor.svg".into(),
        }
    }
}

/// Returns a [ToolButton]
pub fn tool_button(tool: ToolKind) -> impl IntoElement {
    ToolButton::new(tool)
}

#[derive(IntoElement)]
pub struct ToolButton {
    src: SharedString,
}

impl ToolButton {
    pub fn new(tool: ToolKind) -> Self {
        ToolButton { src: tool.src() }
    }
}

impl RenderOnce for ToolButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        div()
            .size(px(21.))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .rounded(px(2.))
            .hover(|div| div.bg(theme.foreground.opacity(0.05)))
            .child(
                svg()
                    .path(self.src)
                    .size(px(15.))
                    .text_color(theme.foreground_muted),
            )
    }
}

#[derive(IntoElement)]
struct ToolStrip {}

impl ToolStrip {
    pub fn new() -> Self {
        ToolStrip {}
    }
}

impl RenderOnce for ToolStrip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        div()
            .id("tool_strip")
            .h_full()
            .w(px(25.))
            .flex()
            .flex_col()
            .gap(px(3.))
            .children(ToolKind::iter().map(tool_button))
    }
}

#[derive(IntoElement)]
struct Sidebar {}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        let inner = div()
            .flex()
            .flex_col()
            .h_full()
            .w(px(260.))
            .rounded_tl(px(15.))
            .rounded_bl(px(15.))
            .px(px(4.))
            .bg(theme.background_color)
            .child(div().w_full().h(px(TITLEBAR_HEIGHT)))
            .child(div().flex().flex_1().w_full().child(ToolStrip::new()));

        div()
            .id("titlebar")
            .h_full()
            .w(px(261.))
            .border_r_1()
            .border_color(gpui::white().alpha(0.08))
            .rounded_tl(px(15.))
            .rounded_bl(px(15.))
            .bg(theme.background_color)
            .shadow(smallvec::smallvec![BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.24),
                offset: point(px(1.), px(0.)),
                blur_radius: px(0.),
                spread_radius: px(0.),
            }])
            .child(inner)
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
            .text_color(theme.foreground)
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
            .when(!self.hide_sidebar, |this| this.child(Sidebar {}))
            .child(div().size_full().flex_1().bg(theme.canvas_color))
    }
}

impl Focusable for Luna {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

struct Assets {
    base: PathBuf,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        fs::read(self.base.join(path))
            .map(|data| Some(std::borrow::Cow::Owned(data)))
            .map_err(|err| err.into())
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|entry| entry.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(|err| err.into())
    }
}

fn main() {
    Application::new()
        .with_assets(Assets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        })
        .run(|cx: &mut App| {
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
                |window, cx| cx.new(|cx| Luna::new(cx)),
            )
            .unwrap();
        });
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}
