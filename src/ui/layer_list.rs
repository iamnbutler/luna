//! Layer list component for displaying and managing elements.
//!
//! Provides a hierarchical view of elements in the canvas,
//! showing their selection state and allowing interaction.

use gpui::{div, prelude::*, px, App, ElementId, Entity, Hsla, IntoElement, SharedString, Window};

use std::collections::HashSet;

use crate::{
    canvas::LunaCanvas,
    node::{frame::FrameNode, NodeCommon, NodeId, NodeType},
    theme::Theme,
};

/// Individual item in the layer list representing a canvas element
#[derive(IntoElement)]
pub struct LayerListItem {
    kind: NodeType,
    name: SharedString,
    selected: bool,
    nesting_level: usize,
}

impl LayerListItem {
    pub fn new(kind: NodeType, name: impl Into<SharedString>) -> Self {
        Self {
            kind,
            name: name.into(),
            selected: false,
            nesting_level: 0,
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
            .on_click(|e, _, _| {})
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
    fn find_parent(&self, nodes: &[FrameNode], node_id: NodeId) -> Option<NodeId> {
        for node in nodes {
            if node.children().contains(&node_id) {
                return Some(node.id());
            }
        }
        None
    }

    // Build the layer list items with hierarchy
    fn build_items(
        &self,
        nodes: &[FrameNode],
        parent_id: Option<NodeId>,
        nesting_level: usize,
        selected_nodes: &HashSet<NodeId>,
    ) -> Vec<LayerListItem> {
        let mut items = Vec::new();

        // Find nodes that are children of the given parent (or root nodes if parent_id is None)
        let children = if let Some(parent) = parent_id {
            // Get children of the specified parent
            nodes
                .iter()
                .filter(|node| self.find_parent(nodes, node.id()) == Some(parent))
                .collect::<Vec<_>>()
        } else {
            // Get root nodes (those without parents)
            nodes
                .iter()
                .filter(|node| self.find_parent(nodes, node.id()).is_none())
                .collect::<Vec<_>>()
        };

        // Create items for these nodes and their children
        for node in children {
            let node_id = node.id();
            let name = format!("Frame {}", node_id.0);
            let selected = selected_nodes.contains(&node_id);

            // Add this node
            items.push(
                LayerListItem::new(NodeType::Frame, name)
                    .selected(selected)
                    .nesting_level(nesting_level),
            );

            // Add children recursively with increased nesting level
            if !node.children().is_empty() {
                let child_items =
                    self.build_items(nodes, Some(node_id), nesting_level + 1, selected_nodes);
                items.extend(child_items);
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

        // Get all nodes from Canvas
        let canvas = self.canvas.read(cx);
        let nodes = canvas.nodes().clone(); // Clone to avoid borrow issues
        let selected_nodes = canvas.selected_nodes().clone();

        // Build hierarchical layer items
        let items = self.build_items(&nodes, None, 0, &selected_nodes);

        // Add all items to the layer list
        for item in items {
            layers = layers.child(item);
        }

        layers
    }
}
