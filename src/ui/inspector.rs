//! Properties inspector for selected elements.
//!
//! The inspector displays and allows editing of properties
//! for selected elements in the canvas.

use std::collections::HashSet;

use gpui::{
    div, prelude::*, px, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Window,
};
use smallvec::SmallVec;

use crate::{
    canvas::LunaCanvas,
    node::{NodeCommon, NodeId},
    theme::Theme,
    AppState,
};

use super::property::float_input;

pub const INSPECTOR_WIDTH: f32 = 200.;

/// Represents the current selection state in the canvas
pub enum NodeSelection {
    /// No nodes are selected
    None,
    /// Exactly one node is selected
    Single(NodeId),
    /// Multiple nodes are selected
    Multiple(Vec<NodeId>),
}

impl From<HashSet<NodeId>> for NodeSelection {
    fn from(nodes: HashSet<NodeId>) -> Self {
        match nodes.len() {
            0 => NodeSelection::None,
            1 => match nodes.iter().next() {
                Some(node_id) => NodeSelection::Single(*node_id),
                None => NodeSelection::None,
            },
            _ => {
                let nodes_vec: Vec<NodeId> = nodes.into_iter().collect();
                NodeSelection::Multiple(nodes_vec)
            }
        }
    }
}

/// Stores property values for currently selected elements
///
/// Uses [`SmallVec`] to efficiently handle both single values and
/// multiple values (for mixed-value states) without heap allocation
/// in the common case.
pub struct InspectorProperties {
    pub x: SmallVec<[f32; 1]>,
    pub y: SmallVec<[f32; 1]>,
    pub width: SmallVec<[f32; 1]>,
    pub height: SmallVec<[f32; 1]>,
    pub border_width: SmallVec<[f32; 1]>,
    pub corner_radius: SmallVec<[f32; 1]>,
}

impl Default for InspectorProperties {
    fn default() -> Self {
        Self {
            x: SmallVec::new(),
            y: SmallVec::new(),
            width: SmallVec::new(),
            height: SmallVec::new(),
            border_width: SmallVec::new(),
            corner_radius: SmallVec::new(),
        }
    }
}

/// Properties panel for viewing and editing element attributes
///
/// The inspector maintains property values for selected elements and renders
/// them with appropriate controls. It handles both single selection (showing
/// exact values) and multiple selection (showing common or mixed values).
pub struct Inspector {
    state: Entity<AppState>,
    canvas: Entity<LunaCanvas>,
    properties: InspectorProperties,
}

impl Inspector {
    pub fn new(state: Entity<AppState>, canvas: Entity<LunaCanvas>) -> Self {
        Self {
            state,
            canvas,
            properties: InspectorProperties::default(),
        }
    }

    /// Updates the inspector properties based on the currently selected nodes
    pub fn update_selected_node_properties(&mut self, cx: &mut Context<Self>) {
        let canvas = self.canvas.clone();
        let selected_node_set = canvas.read(cx).selected_nodes().clone();
        let selected_nodes = NodeSelection::from(selected_node_set);

        // Clear the current properties
        self.properties.x.clear();
        self.properties.y.clear();
        self.properties.width.clear();
        self.properties.height.clear();
        self.properties.border_width.clear();
        self.properties.corner_radius.clear();

        match selected_nodes {
            NodeSelection::None => {
                // Keep properties empty for no selection
            }
            NodeSelection::Single(node_id) => {
                let canvas_read = canvas.read(cx);
                if let Some(node) = canvas_read.nodes().iter().find(|node| node.id() == node_id) {
                    self.properties.x.push(node.layout().x);
                    self.properties.y.push(node.layout().y);
                    self.properties.width.push(node.layout().width);
                    self.properties.height.push(node.layout().height);
                    self.properties.border_width.push(node.border_width());
                    self.properties.corner_radius.push(node.corner_radius());
                }
            }
            NodeSelection::Multiple(nodes) => {
                // For multiple selections, we'll collect all values and then
                // check if they're all the same to properly handle the "Mixed" state
                let canvas_read = canvas.read(cx);

                // Temporary collections for all values
                let mut all_x = Vec::new();
                let mut all_y = Vec::new();
                let mut all_width = Vec::new();
                let mut all_height = Vec::new();
                let mut all_border_width = Vec::new();
                let mut all_corner_radius = Vec::new();

                // Collect all values first
                for node_id in &nodes {
                    if let Some(node) = canvas_read
                        .nodes()
                        .iter()
                        .find(|node| node.id() == *node_id)
                    {
                        all_x.push(node.layout().x);
                        all_y.push(node.layout().y);
                        all_width.push(node.layout().width);
                        all_height.push(node.layout().height);
                        all_border_width.push(node.border_width());
                        all_corner_radius.push(node.corner_radius());
                    }
                }

                // Helper function to check if all values in a vector are the same
                let all_same = |values: &[f32]| -> bool {
                    if values.is_empty() {
                        return true;
                    }
                    let first = values[0];
                    values.iter().all(|&v| (v - first).abs() < f32::EPSILON)
                };

                // If all values are the same, just use the first one
                // Otherwise, use all values to indicate they're different (will show as "Mixed")
                if !all_x.is_empty() {
                    if all_same(&all_x) {
                        self.properties.x.push(all_x[0]);
                    } else {
                        self.properties.x.extend(all_x);
                    }
                }

                if !all_y.is_empty() {
                    if all_same(&all_y) {
                        self.properties.y.push(all_y[0]);
                    } else {
                        self.properties.y.extend(all_y);
                    }
                }

                if !all_width.is_empty() {
                    if all_same(&all_width) {
                        self.properties.width.push(all_width[0]);
                    } else {
                        self.properties.width.extend(all_width);
                    }
                }

                if !all_height.is_empty() {
                    if all_same(&all_height) {
                        self.properties.height.push(all_height[0]);
                    } else {
                        self.properties.height.extend(all_height);
                    }
                }

                if !all_border_width.is_empty() {
                    if all_same(&all_border_width) {
                        self.properties.border_width.push(all_border_width[0]);
                    } else {
                        self.properties.border_width.extend(all_border_width);
                    }
                }

                if !all_corner_radius.is_empty() {
                    if all_same(&all_corner_radius) {
                        self.properties.corner_radius.push(all_corner_radius[0]);
                    } else {
                        self.properties.corner_radius.extend(all_corner_radius);
                    }
                }
            }
        }

        cx.notify();
    }
}

impl Render for Inspector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::default();

        // Update properties based on current selection
        self.update_selected_node_properties(cx);

        // Convert SmallVec properties to Option<Vec<f32>> as needed by property_input
        let x = if self.properties.x.is_empty() {
            None
        } else {
            Some(self.properties.x.iter().cloned().collect())
        };

        let y = if self.properties.y.is_empty() {
            None
        } else {
            Some(self.properties.y.iter().cloned().collect())
        };

        let width = if self.properties.width.is_empty() {
            None
        } else {
            Some(self.properties.width.iter().cloned().collect())
        };

        let height = if self.properties.height.is_empty() {
            None
        } else {
            Some(self.properties.height.iter().cloned().collect())
        };

        let border_width = if self.properties.border_width.is_empty() {
            None
        } else {
            Some(self.properties.border_width.iter().cloned().collect())
        };

        let corner_radius = if self.properties.corner_radius.is_empty() {
            None
        } else {
            Some(self.properties.corner_radius.iter().cloned().collect())
        };

        let inner = div()
            .flex()
            .flex_col()
            .h_full()
            .w(px(INSPECTOR_WIDTH))
            .rounded_tr(px(15.))
            .rounded_br(px(15.))
            .child(
                div()
                    .px(px(8.))
                    .py(px(10.))
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .border_color(theme.tokens.inactive_border)
                    .border_b_1()
                    .child(float_input(x, "X"))
                    .child(float_input(y, "Y"))
                    .child(float_input(width, "W"))
                    .child(float_input(height, "H"))
                    .child(float_input(border_width, "B"))
                    .child(float_input(corner_radius, "R")),
            );

        div()
            .id("titlebar")
            .absolute()
            .right_0()
            .top_0()
            .h_full()
            .w(px(INSPECTOR_WIDTH + 1.))
            .cursor_default()
            .rounded_tr(px(15.))
            .rounded_br(px(15.))
            .border_color(theme.tokens.inactive_border)
            .border_l_1()
            .bg(theme.tokens.background_secondary)
            .on_click(cx.listener(|_, _, _, cx| {
                cx.stop_propagation();
            }))
            .child(inner)
    }
}
