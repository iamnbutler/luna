//! Layer list showing shapes on the canvas.

use crate::components::{panel, v_stack};
use canvas_2::Canvas;
use gpui::{
    div, px, Context, Div, Entity, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, Styled, Window,
};
use node_2::ShapeKind;
use theme_2::Theme;

/// Layer list panel showing all shapes.
pub struct LayerList {
    canvas: Entity<Canvas>,
    theme: Theme,
}

impl LayerList {
    pub fn new(canvas: Entity<Canvas>, theme: Theme) -> Self {
        Self { canvas, theme }
    }
}

impl Render for LayerList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let canvas = self.canvas.read(cx);
        let shapes = &canvas.shapes;
        let selection = &canvas.selection;
        let theme = &self.theme;

        // Build layer items (reverse for top-to-bottom display, top layer first)
        let items: Vec<_> = shapes
            .iter()
            .rev()
            .enumerate()
            .map(|(idx, shape)| {
                let id = shape.id;
                let is_selected = selection.contains(&id);
                let kind_icon = match shape.kind {
                    ShapeKind::Rectangle => "▢",
                    ShapeKind::Ellipse => "○",
                };
                let name: SharedString = format!("Shape {}", shapes.len() - idx).into();
                let item_id: SharedString = format!("layer-{}", id).into();

                LayerItem {
                    id: item_id,
                    icon: kind_icon.into(),
                    name,
                    is_selected,
                    theme: theme.clone(),
                }
            })
            .collect();

        panel(theme)
            .w(px(200.0))
            .h_full()
            .overflow_hidden()
            .child(
                div()
                    .text_xs()
                    .text_color(theme.ui_text_muted)
                    .pb(px(8.0))
                    .child("Layers"),
            )
            .child(v_stack().gap(px(2.0)).children(items))
    }
}

/// A single layer item.
struct LayerItem {
    id: SharedString,
    icon: SharedString,
    name: SharedString,
    is_selected: bool,
    theme: Theme,
}

impl IntoElement for LayerItem {
    type Element = gpui::Stateful<Div>;

    fn into_element(self) -> Self::Element {
        let bg = if self.is_selected {
            self.theme.selection.alpha(0.2)
        } else {
            gpui::transparent_black()
        };

        let border_color = if self.is_selected {
            self.theme.selection
        } else {
            gpui::transparent_black()
        };

        let hover_bg = self.theme.hover;
        let muted = self.theme.ui_text_muted;

        div()
            .id(self.id.clone())
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(8.0))
            .py(px(4.0))
            .bg(bg)
            .border_l_2()
            .border_color(border_color)
            .rounded_r(px(4.0))
            .text_sm()
            .text_color(self.theme.ui_text)
            .cursor_pointer()
            .hover(move |d| d.bg(hover_bg))
            // Fixed-width icon container
            .child(
                div()
                    .w(px(16.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(muted)
                    .child(self.icon),
            )
            .child(self.name)
    }
}
