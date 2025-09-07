//! Layer list component for displaying and managing elements.
//!
//! Provides a hierarchical view of elements in the canvas,
//! showing their selection state and allowing interaction.

use gpui::{
    div, prelude::*, px, App, ElementId, Entity, Hsla, IntoElement, List, SharedString, WeakEntity,
    Window,
};

use std::collections::{HashMap, HashSet};

use canvas::LunaCanvas;
use node::{AnyNode, NodeCommon, NodeId, NodeType};
use theme::Theme;

/// Individual item in the layer list representing a canvas element
#[derive(IntoElement)]
pub struct LayerListItem {
    kind: NodeType,
    node_id: NodeId,
    name: SharedString,
    selected: bool,
    nesting_level: usize,
    weak_canvas_handle: WeakEntity<LunaCanvas>,
}

impl LayerListItem {
    pub fn new(
        weak_canvas_handle: WeakEntity<LunaCanvas>,
        node_id: NodeId,
        name: impl Into<SharedString>,
        kind: NodeType,
    ) -> Self {
        Self {
            kind,
            node_id,
            name: name.into(),
            selected: false,
            nesting_level: 0,
            weak_canvas_handle,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn nesting_level(mut self, level: usize) -> Self {
        self.nesting_level = level;
        self
    }
}

impl RenderOnce for LayerListItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::get_global(cx);

        let text_color = if self.selected {
            theme.tokens.text
        } else {
            theme.tokens.subtext0
        };

        let indentation = px(10.0 + (self.nesting_level as f32 * 10.0));

        div()
            .id(ElementId::Name(format!("layer-{}", self.name).into()))
            .pl(indentation)
            .flex()
            .items_center()
            .rounded_tl(px(4.))
            .rounded_bl(px(4.))
            .when(self.selected, |div| div.bg(theme.tokens.selected))
            .active(|div| div.bg(theme.tokens.surface2.opacity(0.7)))
            .text_color(text_color)
            .gap(px(10.))
            .on_click(move |e, _, cx| {
                let canvas = self
                    .weak_canvas_handle
                    .upgrade()
                    .expect("Canvas handle is dead");
                canvas.update(cx, |canvas, cx| {
                    canvas.select_node(self.node_id);
                });
            })
            .child(div().text_color(text_color.alpha(0.8)).child("□"))
            .child(self.name)
    }
}

/// Container for the list of layer items representing canvas elements
pub struct LayerList {
    canvas: Entity<LunaCanvas>,
}

impl LayerList {
    pub fn new(canvas: Entity<LunaCanvas>, cx: &mut Context<Self>) -> Self {
        Self { canvas }
    }

    // Helper method to find the parent of a node
    fn find_parent(&self, nodes: &HashMap<NodeId, AnyNode>, child_id: NodeId) -> Option<NodeId> {
        for (node_id, node) in nodes {
            if let Some(frame) = node.as_frame() {
                if frame.children().contains(&child_id) {
                    return Some(*node_id);
                }
            }
        }
        None
    }

    // Build the layer list items with hierarchy
    fn build_items(
        &self,
        weak_canvas_handle: WeakEntity<LunaCanvas>,
        nodes: &HashMap<NodeId, AnyNode>,
        parent_id: Option<NodeId>,
        nesting_level: usize,
        selected_nodes: &HashSet<NodeId>,
    ) -> Vec<LayerListItem> {
        let mut items = Vec::new();

        let children = if let Some(parent) = parent_id {
            nodes
                .iter()
                .filter(|(node_id, _)| self.find_parent(nodes, **node_id) == Some(parent))
                .collect::<Vec<_>>()
        } else {
            // root nodes
            nodes
                .iter()
                .filter(|(node_id, _)| self.find_parent(nodes, **node_id).is_none())
                .collect::<Vec<_>>()
        };

        for (node_id, node) in children {
            let node_id = *node_id;
            let node_type = node.node_type();
            let name = match node_type {
                NodeType::Frame => format!("Frame {}", node_id.0),
                NodeType::Shape => format!("Shape {}", node_id.0),
            };
            let selected = selected_nodes.contains(&node_id);

            items.push(
                LayerListItem::new(weak_canvas_handle.clone(), node_id, name, node_type)
                    .selected(selected)
                    .nesting_level(nesting_level),
            );

            // Add children
            if let Some(frame) = node.as_frame() {
                if !frame.children().is_empty() {
                    let child_items = self.build_items(
                        weak_canvas_handle.clone(),
                        nodes,
                        Some(node_id),
                        nesting_level + 1,
                        selected_nodes,
                    );
                    items.extend(child_items);
                }
            }
        }

        items
    }
}

impl Render for LayerList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut layers = div()
            .id("layer-list")
            .key_context("LayerList")
            .flex()
            .flex_col()
            .flex_1()
            .pt_1();

        let canvas = self.canvas.read(cx);
        let nodes = canvas.nodes().clone();
        let selected_nodes = canvas.selected_nodes().clone();
        let weak_canvas_handle = self.canvas.clone().downgrade();

        let items = self.build_items(weak_canvas_handle, &nodes, None, 0, &selected_nodes);

        for item in items {
            layers = layers.child(item);
        }

        layers
    }
}
