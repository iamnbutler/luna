use crate::project::{
    CanvasState, LunaProject, NodeRelationship, Page, SerializedColor, SerializedLayout,
    SerializedNode, SerializedShadow,
};
use anyhow::{anyhow, Result};
use canvas::{AppState, LunaCanvas};
use gpui::{App, Context, Entity};
use node::frame::FrameNode;
use node::shape::ShapeNode;
use node::{AnyNode, NodeCommon, NodeFactory, NodeId, Shadow, ShadowOffset};
use scene_graph::SceneGraph;
use std::collections::{HashMap, HashSet};

/// Serializes the current canvas state into a Luna project
pub fn serialize_canvas(
    canvas: &Entity<LunaCanvas>,
    scene_graph: &Entity<SceneGraph>,
    app_state: &Entity<AppState>,
    cx: &App,
) -> Result<LunaProject> {
    let canvas_data = canvas.read(cx);
    let scene = scene_graph.read(cx);
    let state = app_state.read(cx);

    let mut project = LunaProject::new();

    // Get the first (and currently only) page
    if let Some(page) = project.pages.get_mut(0) {
        // Serialize canvas viewport state
        let scroll_pos = canvas_data.get_scroll_position();
        page.canvas = CanvasState {
            viewport_x: scroll_pos.x,
            viewport_y: scroll_pos.y,
            zoom: canvas_data.zoom(),
            background_color: Some(state.current_background_color.into()),
            selected_nodes: canvas_data
                .selected_nodes()
                .iter()
                .map(|node_id| node_id.0)
                .collect(),
        };

        // Serialize all nodes
        let mut serialized_nodes = Vec::new();
        let mut node_relationships = Vec::new();

        for (node_id, any_node) in canvas_data.nodes().iter() {
            let serialized = serialize_node(any_node)?;
            serialized_nodes.push(serialized);

            // Get children from scene graph if this node has any
            if let Some(scene_node_id) = scene.get_scene_node_for_data_node(*node_id) {
                let children = scene.get_children(scene_node_id);
                if !children.is_empty() {
                    let child_ids: Vec<usize> = children
                        .iter()
                        .filter_map(|scene_child| {
                            scene
                                .get_data_node_for_scene_node(*scene_child)
                                .map(|node_id| node_id.0)
                        })
                        .collect();

                    if !child_ids.is_empty() {
                        node_relationships.push(NodeRelationship {
                            parent_id: node_id.0,
                            child_ids,
                        });
                    }
                }
            }
        }

        page.nodes = serialized_nodes;
        page.hierarchy = node_relationships;
    }

    Ok(project)
}

/// Deserializes a Luna project into canvas state
pub fn deserialize_canvas(
    project: &LunaProject,
    canvas: &Entity<LunaCanvas>,
    scene_graph: &Entity<SceneGraph>,
    app_state: &Entity<AppState>,
    cx: &mut App,
) -> Result<()> {
    let page = project
        .active_page()
        .ok_or_else(|| anyhow!("No active page in project"))?;

    // Clear existing canvas state
    canvas.update(cx, |canvas, cx| {
        canvas.clear_all(cx);
        canvas.mark_dirty(cx);
    });

    scene_graph.update(cx, |scene, _| {
        scene.clear();
    });

    // Update app state colors
    if let Some(bg_color) = &page.canvas.background_color {
        app_state.update(cx, |state, cx| {
            state.current_background_color = bg_color.clone().into();
            cx.notify();
        });
    }

    // Create a node factory for generating nodes
    let mut factory = NodeFactory::new();
    let mut id_mapping: HashMap<usize, NodeId> = HashMap::new();

    // First pass: create all nodes
    eprintln!("Deserializing {} nodes", page.nodes.len());
    canvas.update(cx, |canvas, cx| {
        for serialized_node in &page.nodes {
            match deserialize_node(serialized_node, &mut factory) {
                Ok(node) => {
                    let node_id = node.id();
                    eprintln!(
                        "Created node {:?} with layout: {:?}",
                        node_id,
                        node.layout()
                    );
                    id_mapping.insert(get_serialized_node_id(serialized_node), node_id);
                    canvas.add_node(node, cx);
                    canvas.mark_dirty(cx);
                    eprintln!("Added node {:?} to canvas", node_id);
                }
                Err(e) => {
                    // Failed to deserialize node - skip it
                    eprintln!("Failed to deserialize node: {}", e);
                }
            }
        }
        eprintln!("Canvas now has {} nodes", canvas.nodes().len());
    });

    // Second pass: rebuild hierarchy in scene graph
    eprintln!("Rebuilding scene graph hierarchy");
    scene_graph.update(cx, |scene, _| {
        // Add all nodes to scene graph first
        for &node_id in id_mapping.values() {
            scene.add_node(node_id);
            eprintln!("Added node {:?} to scene graph", node_id);
        }

        // Then establish parent-child relationships
        for relationship in &page.hierarchy {
            if let Some(&parent_id) = id_mapping.get(&relationship.parent_id) {
                for child_id in &relationship.child_ids {
                    if let Some(&child_node_id) = id_mapping.get(child_id) {
                        // Add child to parent in scene graph
                        if let (Some(parent_scene), Some(child_scene)) = (
                            scene.get_scene_node_for_data_node(parent_id),
                            scene.get_scene_node_for_data_node(child_node_id),
                        ) {
                            scene.add_child(parent_scene, child_scene);
                            eprintln!("Added child {:?} to parent {:?}", child_node_id, parent_id);
                        }
                    }
                }
            }
        }
    });

    // Restore canvas viewport state
    canvas.update(cx, |canvas, cx| {
        eprintln!(
            "Restoring viewport to ({}, {})",
            page.canvas.viewport_x, page.canvas.viewport_y
        );
        canvas.set_scroll_position(
            gpui::point(page.canvas.viewport_x, page.canvas.viewport_y),
            cx,
        );
        canvas.set_zoom(page.canvas.zoom, cx);

        // Restore selection
        canvas.deselect_all_nodes(cx);
        for serialized_id in &page.canvas.selected_nodes {
            if let Some(&node_id) = id_mapping.get(serialized_id) {
                canvas.select_node(node_id);
            }
        }

        eprintln!(
            "Final canvas state: {} nodes, viewport at ({}, {})",
            canvas.nodes().len(),
            canvas.get_scroll_position().x,
            canvas.get_scroll_position().y
        );

        // Force a full canvas update after loading
        canvas.mark_dirty(cx);
        cx.notify();
    });

    Ok(())
}

/// Serializes a single node
fn serialize_node(node: &AnyNode) -> Result<SerializedNode> {
    match node {
        AnyNode::Frame(frame) => Ok(SerializedNode::Frame {
            id: frame.id.0,
            layout: frame.layout.clone().into(),
            fill: frame.fill.map(Into::into),
            border_color: frame.border_color.map(Into::into),
            border_width: frame.border_width,
            corner_radius: frame.corner_radius,
            shadows: frame
                .shadows
                .iter()
                .map(|shadow| SerializedShadow {
                    color: shadow.color.into(),
                    x_offset: shadow.offset.x(),
                    y_offset: shadow.offset.y(),
                    blur_radius: shadow.blur_radius,
                    spread_radius: shadow.spread_radius,
                })
                .collect(),
        }),
        AnyNode::Shape(shape) => Ok(SerializedNode::Shape {
            id: shape.id.0,
            layout: shape.layout.clone().into(),
            shape_type: format!("{:?}", shape.shape_type),
            fill: shape.fill.map(Into::into),
            border_color: shape.border_color.map(Into::into),
            border_width: shape.border_width,
            corner_radius: shape.corner_radius,
            shadows: shape
                .shadows
                .iter()
                .map(|shadow| SerializedShadow {
                    color: shadow.color.into(),
                    x_offset: shadow.offset.x(),
                    y_offset: shadow.offset.y(),
                    blur_radius: shadow.blur_radius,
                    spread_radius: shadow.spread_radius,
                })
                .collect(),
        }),
    }
}

/// Deserializes a single node
fn deserialize_node(serialized: &SerializedNode, factory: &mut NodeFactory) -> Result<AnyNode> {
    match serialized {
        SerializedNode::Frame {
            id,
            layout,
            fill,
            border_color,
            border_width,
            corner_radius,
            shadows,
        } => {
            let mut frame = factory.create_frame();

            // Override the generated ID with the serialized one
            frame.id = NodeId(*id);
            frame.layout = layout.clone().into();
            frame.fill = fill.as_ref().map(|c| c.clone().into());
            frame.border_color = border_color.as_ref().map(|c| c.clone().into());
            frame.border_width = *border_width;
            frame.corner_radius = *corner_radius;

            for shadow in shadows {
                frame.shadows.push(Shadow {
                    color: shadow.color.clone().into(),
                    offset: ShadowOffset::new(shadow.x_offset, shadow.y_offset),
                    blur_radius: shadow.blur_radius,
                    spread_radius: shadow.spread_radius,
                });
            }

            Ok(AnyNode::Frame(frame))
        }
        SerializedNode::Shape {
            id,
            layout,
            shape_type: _,
            fill,
            border_color,
            border_width,
            corner_radius,
            shadows,
        } => {
            // For now, create shapes as frames since we don't have shape deserialization
            // This can be enhanced later when shape types are properly implemented
            let mut frame = factory.create_frame();

            frame.id = NodeId(*id);
            frame.layout = layout.clone().into();
            frame.fill = fill.as_ref().map(|c| c.clone().into());
            frame.border_color = border_color.as_ref().map(|c| c.clone().into());
            frame.border_width = *border_width;
            frame.corner_radius = *corner_radius;

            for shadow in shadows {
                frame.shadows.push(Shadow {
                    color: shadow.color.clone().into(),
                    offset: ShadowOffset::new(shadow.x_offset, shadow.y_offset),
                    blur_radius: shadow.blur_radius,
                    spread_radius: shadow.spread_radius,
                });
            }

            Ok(AnyNode::Frame(frame))
        }
    }
}

/// Gets the ID from a serialized node
fn get_serialized_node_id(node: &SerializedNode) -> usize {
    match node {
        SerializedNode::Frame { id, .. } => *id,
        SerializedNode::Shape { id, .. } => *id,
    }
}
