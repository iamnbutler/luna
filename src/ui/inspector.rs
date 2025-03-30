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

use super::property::property_input;

pub const INSPECTOR_WIDTH: f32 = 200.;

pub enum NodeSelection {
    None,
    Single(NodeId),
    Multiple(Vec<NodeId>),
}

impl From<HashSet<NodeId>> for NodeSelection {
    fn from(nodes: HashSet<NodeId>) -> Self {
        match nodes.len() {
            0 => NodeSelection::None,
            1 => {
                let node_id = nodes.iter().next().unwrap_or({
                    println!("Couldn't select node, falling back to None");
                    return NodeSelection::None;
                });
                NodeSelection::Single(*node_id)
            }
            _ => {
                let nodes_vec: Vec<NodeId> = nodes.into_iter().collect();
                NodeSelection::Multiple(nodes_vec.into_iter().collect())
            }
        }
    }
}

pub struct InspectorProperties {
    pub x: SmallVec<[f32; 1]>,
    pub y: SmallVec<[f32; 1]>,
    pub width: SmallVec<[f32; 1]>,
    pub height: SmallVec<[f32; 1]>,
}

impl Default for InspectorProperties {
    fn default() -> Self {
        Self {
            x: SmallVec::new(),
            y: SmallVec::new(),
            width: SmallVec::new(),
            height: SmallVec::new(),
        }
    }
}

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
        let selected_node_set = canvas.read(cx).selected_nodes.clone();
        let selected_nodes = NodeSelection::from(selected_node_set);
        
        // Clear the current properties
        self.properties.x.clear();
        self.properties.y.clear();
        self.properties.width.clear();
        self.properties.height.clear();
        
        match selected_nodes {
            NodeSelection::None => {
                // Keep properties empty for no selection
            },
            NodeSelection::Single(node_id) => {
                let canvas_read = canvas.read(cx);
                if let Some(node) = canvas_read.nodes.iter().find(|node| node.id() == node_id) {
                    self.properties.x.push(node.layout().x);
                    self.properties.y.push(node.layout().y);
                    self.properties.width.push(node.layout().width);
                    self.properties.height.push(node.layout().height);
                }
            },
            NodeSelection::Multiple(nodes) => {
                let canvas_read = canvas.read(cx);
                for node_id in &nodes {
                    if let Some(node) = canvas_read.nodes.iter().find(|node| node.id() == *node_id) {
                        self.properties.x.push(node.layout().x);
                        self.properties.y.push(node.layout().y);
                        self.properties.width.push(node.layout().width);
                        self.properties.height.push(node.layout().height);
                    }
                }
            }
        }
    }
}

impl Render for Inspector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::new();
        
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
                    .border_color(theme.foreground.alpha(0.06))
                    .border_b_1()
                    .child(property_input(x, "X"))
                    .child(property_input(y, "Y"))
                    .child(property_input(width, "W"))
                    .child(property_input(height, "H")),
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
            .border_color(theme.foreground.alpha(0.06))
            .border_l_1()
            .bg(theme.background_color)
            .on_click(cx.listener(|_, _, _, cx| {
                cx.stop_propagation();
            }))
            .child(inner)
    }
}
