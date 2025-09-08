//! Left sidebar containing tools and controls.
//!
//! The sidebar renders the primary tool selection interface and
//! other controls for interacting with the canvas.

use canvas::{tools::ToolStrip, LunaCanvas};
use gpui::{
    actions, div, hsla, point, prelude::*, px, svg, App, Application, AssetSource, BoxShadow,
    ElementId, Entity, FocusHandle, Focusable, Global, Hsla, IntoElement, Keystroke, Menu,
    MenuItem, Modifiers, Pixels, Point, SharedString, TitlebarOptions, UpdateGlobal, WeakEntity,
    Window, WindowBackgroundAppearance, WindowOptions,
};
use project::ProjectState;
use theme::Theme;

use super::{layer_list::LayerList, Titlebar};

/// Container for tool selection and other canvas controls
///
/// Renders the vertical sidebar on the left side of the application,
/// hosting the [`ToolStrip`] and other controls for canvas interaction.
pub struct Sidebar {
    canvas: Entity<LunaCanvas>,
    layer_list: Entity<LayerList>,
    project_state: Entity<ProjectState>,
}

impl Sidebar {
    pub fn new(
        canvas: Entity<LunaCanvas>,
        project_state: Entity<ProjectState>,
        cx: &mut Context<Self>,
    ) -> Self {
        let layer_list = cx.new(|cx| LayerList::new(canvas.clone(), cx));
        Self {
            canvas,
            layer_list,
            project_state,
        }
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
            .child(
                div()
                    .w_full()
                    .h(px(Titlebar::HEIGHT))
                    .flex()
                    .items_center()
                    .px_2()
                    .child({
                        let project_state = self.project_state.read(cx);
                        let display_name = project_state.display_name();
                        let is_dirty = project_state.is_dirty;

                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .text_sm()
                            .text_color(token.text)
                            .when(is_dirty, |d| {
                                d.child(div().size_2().rounded_full().bg(token.overlay0))
                            })
                            .child(display_name)
                    }),
            )
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
