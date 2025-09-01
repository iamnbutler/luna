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

use canvas::{AppState, LunaCanvas};
use node::{NodeCommon, NodeId};
use theme::Theme;

use crate::property::{float_input, ColorInput, PropertyType};

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
#[derive(Debug, Clone)]
pub struct InspectorProperties {
    pub x: SmallVec<[f32; 1]>,
    pub y: SmallVec<[f32; 1]>,
    pub width: SmallVec<[f32; 1]>,
    pub height: SmallVec<[f32; 1]>,
    pub border_width: SmallVec<[f32; 1]>,
    pub corner_radius: SmallVec<[f32; 1]>,
    pub border_color: SmallVec<[SharedString; 1]>,
    pub background_color: SmallVec<[SharedString; 1]>,
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
            border_color: SmallVec::new(),
            background_color: SmallVec::new(),
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
        self.properties.border_color.clear();
        self.properties.background_color.clear();

        match selected_nodes {
            NodeSelection::None => {
                // Keep properties empty for no selection
            }
            NodeSelection::Single(node_id) => {
                let canvas_read = canvas.read(cx);
                if let Some((_id, node)) =
                    canvas_read.nodes().iter().find(|(id, _)| **id == node_id)
                {
                    // Round position and size values to one decimal place
                    self.properties.x.push(node.layout().x);
                    self.properties.y.push(node.layout().y);
                    self.properties.width.push(node.layout().width);
                    self.properties.height.push(node.layout().height);
                    self.properties.border_width.push(node.border_width());
                    self.properties.corner_radius.push(node.corner_radius());

                    // Add color properties with integers instead of decimals
                    if let Some(border_color) = node.border_color() {
                        let color_str = self.format_color_string(border_color.to_string());
                        self.properties
                            .border_color
                            .push(SharedString::from(color_str));
                    }

                    if let Some(fill_color) = node.fill() {
                        let color_str = self.format_color_string(fill_color.to_string());
                        self.properties
                            .background_color
                            .push(SharedString::from(color_str));
                    }
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
                let mut all_border_colors = Vec::new();
                let mut all_background_colors = Vec::new();

                // Collect all values first
                for node_id in &nodes {
                    if let Some(node) = canvas_read
                        .nodes()
                        .iter()
                        .find(|(id, _)| **id == *node_id)
                        .map(|(_, node)| node)
                    {
                        all_x.push(node.layout().x);
                        all_y.push(node.layout().y);
                        all_width.push(node.layout().width);
                        all_height.push(node.layout().height);
                        all_border_width.push(node.border_width());
                        all_corner_radius.push(node.corner_radius());

                        // Collect color values
                        if let Some(border_color) = node.border_color() {
                            all_border_colors
                                .push(self.format_color_string(border_color.to_string()));
                        }

                        if let Some(fill_color) = node.fill() {
                            all_background_colors
                                .push(self.format_color_string(fill_color.to_string()));
                        }
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

                // Helper function to check if all strings in a vector are the same
                let all_same_str = |values: &[String]| -> bool {
                    if values.is_empty() {
                        return true;
                    }
                    let first = &values[0];
                    values.iter().all(|v| v == first)
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

                // Handle color properties
                if !all_border_colors.is_empty() {
                    if all_same_str(&all_border_colors) {
                        self.properties
                            .border_color
                            .push(SharedString::from(&all_border_colors[0]));
                    } else {
                        // For mixed values, just push one to indicate mixed state
                        self.properties
                            .border_color
                            .push(SharedString::from("Mixed"));
                    }
                }

                if !all_background_colors.is_empty() {
                    if all_same_str(&all_background_colors) {
                        self.properties
                            .background_color
                            .push(SharedString::from(&all_background_colors[0]));
                    } else {
                        // For mixed values, just push one to indicate mixed state
                        self.properties
                            .background_color
                            .push(SharedString::from("Mixed"));
                    }
                }
            }
        }

        cx.notify();
    }

    /// Format a color string to use integers instead of decimals
    fn format_color_string(&self, color_str: String) -> String {
        // Replace decimal numbers with integers in color strings
        // Example: rgba(255.00, 0.00, 0.00, 1.00) -> rgba(255, 0, 0, 1)
        let mut result = color_str;

        // Find all decimal numbers and replace them
        let decimal_regex = regex::Regex::new(r"(\d+)\.\d+").unwrap();
        result = decimal_regex.replace_all(&result, "$1").to_string();

        result
    }

    /// Converts property data to the format needed by UI components
    /// with visual rounding applied to numerical values
    fn get_ui_property_values(
        &self,
    ) -> (
        Option<Vec<f32>>,
        Option<Vec<f32>>,
        Option<Vec<f32>>,
        Option<Vec<f32>>,
        Option<Vec<f32>>,
        Option<Vec<f32>>,
        Option<SharedString>,
        Option<SharedString>,
    ) {
        // Helper function to round f32 values to one decimal place
        let round_values = |values: &[f32]| -> Vec<f32> {
            values.iter().map(|&v| (v * 10.0).round() / 10.0).collect()
        };

        // Convert SmallVec properties to Option<Vec<f32>> with rounding
        let x = if self.properties.x.is_empty() {
            None
        } else {
            Some(round_values(&self.properties.x))
        };

        let y = if self.properties.y.is_empty() {
            None
        } else {
            Some(round_values(&self.properties.y))
        };

        let width = if self.properties.width.is_empty() {
            None
        } else {
            Some(round_values(&self.properties.width))
        };

        let height = if self.properties.height.is_empty() {
            None
        } else {
            Some(round_values(&self.properties.height))
        };

        let border_width = if self.properties.border_width.is_empty() {
            None
        } else {
            Some(round_values(&self.properties.border_width))
        };

        let corner_radius = if self.properties.corner_radius.is_empty() {
            None
        } else {
            Some(round_values(&self.properties.corner_radius))
        };

        // Convert color properties for the ColorInput components
        let border_color = if self.properties.border_color.is_empty() {
            None
        } else if self.properties.border_color.len() == 1 {
            Some(self.properties.border_color[0].clone())
        } else {
            Some(SharedString::from("Mixed"))
        };

        let background_color = if self.properties.background_color.is_empty() {
            None
        } else if self.properties.background_color.len() == 1 {
            Some(self.properties.background_color[0].clone())
        } else {
            Some(SharedString::from("Mixed"))
        };

        (
            x,
            y,
            width,
            height,
            border_width,
            corner_radius,
            border_color,
            background_color,
        )
    }
}

impl Render for Inspector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::default();

        // Update properties based on current selection
        self.update_selected_node_properties(cx);

        // Get property values formatted for UI display with appropriate rounding
        let (x, y, width, height, border_width, corner_radius, border_color, background_color) =
            self.get_ui_property_values();

        // Get selected node IDs
        let selected_nodes: Vec<NodeId> = self
            .canvas
            .read(cx)
            .selected_nodes()
            .iter()
            .copied()
            .collect();

        let inner = div()
            .id("inspector-inner")
            .flex()
            .flex_col()
            .h_full()
            .w(px(INSPECTOR_WIDTH))
            .rounded_tr(px(15.))
            .rounded_br(px(15.))
            .on_click(cx.listener(|_, _, _, cx| {
                cx.stop_propagation();
            }))
            .child(
                div()
                    .px(px(8.))
                    .py(px(10.))
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .border_color(theme.tokens.inactive_border)
                    .border_b_1()
                    .child(float_input(
                        x,
                        "X",
                        PropertyType::X,
                        selected_nodes.clone(),
                        self.canvas.clone(),
                        cx,
                    ))
                    .child(float_input(
                        y,
                        "Y",
                        PropertyType::Y,
                        selected_nodes.clone(),
                        self.canvas.clone(),
                        cx,
                    ))
                    .child(float_input(
                        width,
                        "W",
                        PropertyType::Width,
                        selected_nodes.clone(),
                        self.canvas.clone(),
                        cx,
                    ))
                    .child(float_input(
                        height,
                        "H",
                        PropertyType::Height,
                        selected_nodes.clone(),
                        self.canvas.clone(),
                        cx,
                    ))
                    .child(float_input(
                        border_width,
                        "B",
                        PropertyType::BorderWidth,
                        selected_nodes.clone(),
                        self.canvas.clone(),
                        cx,
                    ))
                    .child(float_input(
                        corner_radius,
                        "R",
                        PropertyType::CornerRadius,
                        selected_nodes.clone(),
                        self.canvas.clone(),
                        cx,
                    )),
            )
            .child(
                div()
                    .px(px(8.))
                    .py(px(10.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .border_color(theme.tokens.inactive_border)
                    .border_b_1()
                    .child(ColorInput::new(background_color, SharedString::from("BG")))
                    .child(ColorInput::new(border_color, SharedString::from("BC"))),
            );

        div()
            .id("inspector")
            .absolute()
            .right_0()
            .top_0()
            .h_full()
            .w(px(INSPECTOR_WIDTH + 1.))
            .rounded_tr(px(15.))
            .rounded_br(px(15.))
            .border_color(theme.tokens.inactive_border)
            .border_l_1()
            .bg(theme.tokens.background_secondary)
            .child(inner)
    }
}
