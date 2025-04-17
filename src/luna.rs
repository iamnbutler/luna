//! # Luna: A software design tool without compromises.
//!
//! Luna is local, files on disk first. Own your own data,
//! collaborate, design on the canvas or write code.
//!
//! It's not a design tool, or a code editor, it's a tool
//! for designing software:
//!
//! That means not just pixels, but representative screens and flows
//! using an abstractionless design and editing experience.

use gpui::{
    actions, div, point, prelude::*, px, rgba, App, AppContext, Application, FocusHandle, Menu,
    MenuItem, Rgba, TitlebarOptions, Window, WindowBackgroundAppearance, WindowOptions,
};
mod geometry;

actions!(luna, [Quit]);

fn hex(hex_value: impl Into<String>) -> Rgba {
    let hex_str = hex_value.into();
    let hex_str = hex_str.trim_start_matches('#');

    let parsed_value = match hex_str.len() {
        3 => {
            let r = u32::from_str_radix(&hex_str[0..1], 16).unwrap_or(0);
            let g = u32::from_str_radix(&hex_str[1..2], 16).unwrap_or(0);
            let b = u32::from_str_radix(&hex_str[2..3], 16).unwrap_or(0);
            (r << 20) | (r << 16) | (g << 12) | (g << 8) | (b << 4) | b | (0xFF << 24)
        }
        6 => {
            let rgb = u32::from_str_radix(hex_str, 16).unwrap_or(0);
            rgb | (0xFF << 24)
        }
        8 => u32::from_str_radix(hex_str, 16).unwrap_or(0),
        _ => 0xFF0000FF,
    };

    rgba(parsed_value)
}

struct Luna {
    // The main canvas where elements are rendered and manipulated
    // active_canvas: Entity<LunaCanvas>,
    /// Focus handle for keyboard event routing
    focus_handle: FocusHandle,
}

impl Luna {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }

    fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Luna {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .items_center()
            .justify_center()
            .id("scene-graph")
            .key_context("Luna")
            .track_focus(&self.focus_handle())
            .text_xs()
            .font_family("Berkeley Mono")
            .flex()
            .flex_col()
            .relative()
            .bg(hex("#000000"))
            .size_full()
            .text_color(hex("#FFFFFF"))
            .child("Luna")
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.on_action(quit);
        cx.set_menus(vec![Menu {
            name: "Luna".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        }]);

        let window = cx
            .open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some("Luna".into()),
                        traffic_light_position: Some(point(px(8.0), px(8.0))),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| Luna::new(window, cx)),
            )
            .unwrap();

        cx.on_keyboard_layout_change({
            move |cx| {
                window.update(cx, |_, _, cx| cx.notify()).ok();
            }
        })
        .detach();

        window
            .update(cx, |view, window, cx| {
                window.focus(&view.focus_handle());
                cx.activate(true);
            })
            .unwrap();
    });
}

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}
