//! Tool rail for selecting canvas tools.
//!
//! A vertical toolbar inspired by Adobe-style design tools.

use canvas_2::{Canvas, Tool};
use gpui::{
    div, px, svg, Context, Div, ElementId, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window,
};
use theme_2::Theme;

/// Tool rail panel showing available tools.
pub struct ToolRail {
    canvas: Entity<Canvas>,
    theme: Theme,
}

impl ToolRail {
    pub fn new(canvas: Entity<Canvas>, theme: Theme) -> Self {
        Self { canvas, theme }
    }
}

impl Render for ToolRail {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_tool = self.canvas.read(cx).tool;
        let theme = self.theme.clone();
        let canvas = self.canvas.clone();

        div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .p(px(4.0))
            .children([
                ToolButton::new("select", Tool::Select, "svg/arrow_pointer.svg", current_tool, theme.clone(), canvas.clone()),
                ToolButton::new("pan", Tool::Pan, "svg/hand.svg", current_tool, theme.clone(), canvas.clone()),
                ToolButton::new("rectangle", Tool::Rectangle, "svg/square.svg", current_tool, theme.clone(), canvas.clone()),
                ToolButton::new("ellipse", Tool::Ellipse, "svg/shapes.svg", current_tool, theme.clone(), canvas),
            ])
    }
}

/// A single tool button.
struct ToolButton {
    id: ElementId,
    tool: Tool,
    icon_path: SharedString,
    is_active: bool,
    theme: Theme,
    canvas: Entity<Canvas>,
}

impl ToolButton {
    fn new(
        id: impl Into<SharedString>,
        tool: Tool,
        icon_path: impl Into<SharedString>,
        current_tool: Tool,
        theme: Theme,
        canvas: Entity<Canvas>,
    ) -> Self {
        Self {
            id: ElementId::Name(id.into()),
            tool,
            icon_path: icon_path.into(),
            is_active: current_tool == tool,
            theme,
            canvas,
        }
    }
}

impl IntoElement for ToolButton {
    type Element = gpui::Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let bg = if self.is_active {
            self.theme.selection.alpha(0.2)
        } else {
            gpui::transparent_black()
        };

        let border_color = if self.is_active {
            self.theme.selection
        } else {
            gpui::transparent_black()
        };

        let hover_bg = self.theme.hover;
        let icon_color = if self.is_active {
            self.theme.selection
        } else {
            self.theme.ui_text_muted
        };

        let tool = self.tool;
        let canvas = self.canvas;

        div()
            .id(self.id)
            .size(px(28.0))
            .flex()
            .items_center()
            .justify_center()
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .rounded(px(4.0))
            .cursor_pointer()
            .hover(move |d| d.bg(hover_bg))
            .on_click(move |_, _window, cx| {
                canvas.update(cx, |canvas, cx| {
                    canvas.tool = tool;
                    cx.notify();
                });
            })
            .child(
                svg()
                    .path(self.icon_path)
                    .size(px(16.0))
                    .text_color(icon_color),
            )
    }
}
