use gpui::{
    actions, div, hsla, point, prelude::*, px, svg, App, Application, AssetSource, BoxShadow,
    ElementId, Entity, FocusHandle, Focusable, Global, Hsla, IntoElement, Keystroke, Menu,
    MenuItem, Modifiers, Pixels, Point, SharedString, TitlebarOptions, UpdateGlobal, WeakEntity,
    Window, WindowBackgroundAppearance, WindowOptions,
};

use crate::{canvas::LunaCanvas, node::NodeType, theme::Theme};

#[derive(IntoElement)]
pub struct LayerListItem {
    kind: NodeType,
    name: SharedString,
    selected: bool,
}

impl LayerListItem {
    pub fn new(kind: NodeType, name: impl Into<SharedString>) -> Self {
        Self {
            kind,
            name: name.into(),
            selected: false,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl RenderOnce for LayerListItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        let text_color = if self.selected {
            theme.tokens.foreground
        } else {
            theme.tokens.subtext0 // Use subtext0 for unselected items
        };

        div()
            .id(ElementId::Name(format!("layer-{}", self.name).into()))
            .pl(px(10.))
            .flex()
            .items_center()
            .rounded_tl(px(4.))
            .rounded_bl(px(4.))
            .when(self.selected, |div| div.bg(theme.tokens.selected)) // Already has appropriate opacity
            .active(|div| div.bg(theme.tokens.surface2.opacity(0.7))) // Use surface2 for hover state
            .text_color(text_color)
            .gap(px(10.))
            // .child(
            //     svg()
            //         .path(self.kind.icon_src())
            //         .size(px(11.))
            //         .text_color(text_color.alpha(0.8)),
            // )
            .child(self.name)
    }
}

#[derive(IntoElement)]
struct LayerList {
    canvas: Entity<LunaCanvas>,
}

impl LayerList {
    fn new(canvas: Entity<LunaCanvas>) -> Self {
        Self { canvas }
    }
}

impl RenderOnce for LayerList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut layers = div().flex().flex_col().flex_1().pt_1();

        // Get all nodes from Canvas
        let canvas = self.canvas.read(cx);

        // Add all nodes to the layer list
        for node in &canvas.nodes {
            let kind = NodeType::Rectangle; // We only have rectangle nodes now

            let name = format!("Node {}", node.id.0);
            let selected = canvas.is_node_selected(node.id);

            layers = layers.child(LayerListItem::new(kind, name).selected(selected));
        }

        layers
    }
}
