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

use super::TITLEBAR_HEIGHT;

/// Container for tool selection and other canvas controls
///
/// Renders the vertical sidebar on the left side of the application,
/// hosting the [`ToolStrip`] and other controls for canvas interaction.
pub struct Sidebar {
    canvas: Entity<LunaCanvas>,
}

impl Sidebar {
    pub fn new(canvas: Entity<LunaCanvas>) -> Self {
        Self { canvas }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        let inner = div()
            .flex()
            .flex_col()
            .h_full()
            .w(px(35.))
            .rounded_tl(px(15.))
            .rounded_bl(px(15.))
            .child(div().w_full().h(px(TITLEBAR_HEIGHT)))
            .child(div().flex().flex_1().w_full().child(ToolStrip::new()));

        div()
            .id("titlebar")
            .absolute()
            .top_0()
            .left_0()
            .h_full()
            .w(px(36.))
            .cursor_default()
            .rounded_tl(px(15.))
            .rounded_bl(px(15.))
            .child(inner)
    }
}
