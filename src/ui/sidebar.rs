//! Left sidebar containing tools and controls.
//!
//! The sidebar renders the primary tool selection interface and
//! other controls for interacting with the canvas.

use crate::{canvas::LunaCanvas, theme::Theme, tools::ToolStrip};
use gpui::{
    actions, div, hsla, point, prelude::*, px, svg, App, Application, AssetSource, BoxShadow,
    ElementId, Entity, FocusHandle, Focusable, Global, Hsla, IntoElement, Keystroke, Menu,
    MenuItem, Modifiers, Pixels, Point, SharedString, TitlebarOptions, UpdateGlobal, WeakEntity,
    Window, WindowBackgroundAppearance, WindowOptions,
};

use super::{layer_list::LayerList, Titlebar};

/// Container for tool selection and other canvas controls
///
/// Renders the vertical sidebar on the left side of the application,
/// hosting the [`ToolStrip`] and other controls for canvas interaction.
pub struct Sidebar {
    canvas: Entity<LunaCanvas>,
    layer_list: Entity<LayerList>,
}

impl Sidebar {
    pub fn new(canvas: Entity<LunaCanvas>, cx: &mut Context<Self>) -> Self {
        let layer_list = cx.new(|cx| LayerList::new(canvas.clone(), cx));
        Self { canvas, layer_list }
    }
}

impl Sidebar {
    pub const INITIAL_WIDTH: f32 = 220.;
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let token = &Theme::get_global(cx).tokens;

        let inner = div()
            .id("sidebar-inner")
            .flex()
            .flex_col()
            .h_full()
            .w(px(Self::INITIAL_WIDTH))
            .rounded_tl(px(15.))
            .rounded_bl(px(15.))
            .child(div().w_full().h(px(Titlebar::HEIGHT)))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .child(ToolStrip::new())
                    .child(self.layer_list.clone()),
            );

        div()
            .id("sidebar")
            .key_context("Sidebar")
            .absolute()
            .top_0()
            .left_0()
            .h_full()
            .w(px(Self::INITIAL_WIDTH + 1.))
            .bg(token.background_secondary)
            .border_r_1()
            .border_color(token.inactive_border)
            .cursor_default()
            .rounded_tl(px(15.))
            .rounded_bl(px(15.))
            .child(inner)
    }
}
