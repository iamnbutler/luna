//! Luna Interchange Format
//!
//! KDL-based interchange format for Luna canvas documents.
//! Pure data, no expressions - what you see is what's there.
//!
//! # Package Format
//!
//! A `.luna` project is a folder containing:
//! - `manifest.kdl` - Project metadata
//! - `pages/*.kdl` - One file per page/canvas
//! - `assets/` - Linked resources (future)
//!
//! # Document Format
//!
//! ```kdl
//! document version="0.1" {
//!   rect "abc12345" x=100 y=100 width=150 height=100 {
//!     fill h=0.5 s=0.8 l=0.5 a=1.0
//!     stroke width=2 h=0 s=0 l=0 a=1
//!     radius 8
//!   }
//!   ellipse "def67890" x=300 y=150 width=120 height=120 {
//!     stroke width=2 h=0 s=0 l=0 a=1
//!   }
//! }
//! ```

mod project;

pub use project::Project;

use kdl::{KdlDocument, KdlEntry, KdlNode};
use node::{Fill, Shape, ShapeId, ShapeKind, Stroke};

pub const FORMAT_VERSION: &str = "0.1";

/// Error type for interchange operations.
#[derive(Debug)]
pub enum InterchangeError {
    Parse(String),
    InvalidStructure(String),
    MissingField(String),
    InvalidValue(String),
}

impl std::fmt::Display for InterchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(msg) => write!(f, "Parse error: {}", msg),
            Self::InvalidStructure(msg) => write!(f, "Invalid structure: {}", msg),
            Self::MissingField(msg) => write!(f, "Missing field: {}", msg),
            Self::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
        }
    }
}

impl std::error::Error for InterchangeError {}

/// A Luna document that can be serialized to/from KDL.
#[derive(Debug, Clone)]
pub struct Document {
    pub version: String,
    pub shapes: Vec<Shape>,
}

impl Document {
    pub fn new(shapes: Vec<Shape>) -> Self {
        Self {
            version: FORMAT_VERSION.to_string(),
            shapes,
        }
    }

    /// Serialize the document to a KDL string.
    pub fn to_kdl(&self) -> String {
        let mut doc = KdlDocument::new();

        // Create document node
        let mut doc_node = KdlNode::new("document");
        doc_node.push(KdlEntry::new_prop("version", self.version.clone()));

        // Add only root shapes (no parent) - children are nested inside their parents
        let children = doc_node.children_mut().get_or_insert_with(KdlDocument::new);
        for shape in self.shapes.iter().filter(|s| s.parent.is_none()) {
            children.nodes_mut().push(shape_to_kdl(shape, &self.shapes));
        }

        doc.nodes_mut().push(doc_node);
        doc.to_string()
    }

    /// Parse a document from a KDL string.
    pub fn from_kdl(input: &str) -> Result<Self, InterchangeError> {
        let doc: KdlDocument = input
            .parse()
            .map_err(|e| InterchangeError::Parse(format!("{}", e)))?;

        // Find the document node
        let doc_node = doc
            .get("document")
            .ok_or_else(|| InterchangeError::InvalidStructure("Missing 'document' node".into()))?;

        // Get version
        let version = doc_node
            .get("version")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| FORMAT_VERSION.to_string());

        // Parse shapes from children (recursively flattens nested frames)
        let mut shapes = Vec::new();
        if let Some(children) = doc_node.children() {
            for node in children.nodes() {
                parse_shape_recursive(node, None, &mut shapes)?;
            }
        }

        Ok(Self { version, shapes })
    }
}

/// Convert a Shape to a KDL node (recursively includes children).
fn shape_to_kdl(shape: &Shape, all_shapes: &[Shape]) -> KdlNode {
    let type_name = match shape.kind {
        ShapeKind::Rectangle => "rect",
        ShapeKind::Ellipse => "ellipse",
        ShapeKind::Frame => "frame",
    };

    let mut node = KdlNode::new(type_name);

    // ID as first argument (full UUID for round-trip fidelity)
    node.push(KdlEntry::new(shape.id.to_uuid_string()));

    // Position and size as properties
    node.push(KdlEntry::new_prop("x", shape.position.x() as f64));
    node.push(KdlEntry::new_prop("y", shape.position.y() as f64));
    node.push(KdlEntry::new_prop("width", shape.size.width() as f64));
    node.push(KdlEntry::new_prop("height", shape.size.height() as f64));

    // Frame-specific: clip property
    if shape.kind == ShapeKind::Frame && shape.clip_children {
        node.push(KdlEntry::new_prop("clip", true));
    }

    // Children block for styles and nested shapes
    let mut has_children = false;
    let children = node.children_mut().get_or_insert_with(KdlDocument::new);

    if let Some(fill) = &shape.fill {
        let mut fill_node = KdlNode::new("fill");
        fill_node.push(KdlEntry::new_prop("h", fill.color.h as f64));
        fill_node.push(KdlEntry::new_prop("s", fill.color.s as f64));
        fill_node.push(KdlEntry::new_prop("l", fill.color.l as f64));
        fill_node.push(KdlEntry::new_prop("a", fill.color.a as f64));
        children.nodes_mut().push(fill_node);
        has_children = true;
    }

    if let Some(stroke) = &shape.stroke {
        let mut stroke_node = KdlNode::new("stroke");
        stroke_node.push(KdlEntry::new_prop("width", stroke.width as f64));
        stroke_node.push(KdlEntry::new_prop("h", stroke.color.h as f64));
        stroke_node.push(KdlEntry::new_prop("s", stroke.color.s as f64));
        stroke_node.push(KdlEntry::new_prop("l", stroke.color.l as f64));
        stroke_node.push(KdlEntry::new_prop("a", stroke.color.a as f64));
        children.nodes_mut().push(stroke_node);
        has_children = true;
    }

    if shape.corner_radius > 0.0 {
        let mut radius_node = KdlNode::new("radius");
        radius_node.push(KdlEntry::new(shape.corner_radius as f64));
        children.nodes_mut().push(radius_node);
        has_children = true;
    }

    // Recursively serialize child shapes (for frames)
    for child_id in &shape.children {
        if let Some(child) = all_shapes.iter().find(|s| s.id == *child_id) {
            children.nodes_mut().push(shape_to_kdl(child, all_shapes));
            has_children = true;
        }
    }

    // Remove empty children block
    if !has_children {
        *node.children_mut() = None;
    }

    node
}

/// Parse a shape node recursively, adding shapes to the flat list.
/// Sets up parent/children relationships as it goes.
fn parse_shape_recursive(
    node: &KdlNode,
    parent_id: Option<ShapeId>,
    shapes: &mut Vec<Shape>,
) -> Result<ShapeId, InterchangeError> {
    let kind = match node.name().value() {
        "rect" => ShapeKind::Rectangle,
        "ellipse" => ShapeKind::Ellipse,
        "frame" => ShapeKind::Frame,
        other => {
            return Err(InterchangeError::InvalidValue(format!(
                "Unknown shape type: {}",
                other
            )))
        }
    };

    // Parse ID from first argument (or generate new one)
    let id = node
        .entries()
        .iter()
        .find(|e| e.name().is_none())
        .and_then(|e| e.value().as_string())
        .map(|s| ShapeId::from_str(s))
        .unwrap_or_else(ShapeId::new);

    // Parse position and size
    let x = get_f32_prop(node, "x").unwrap_or(0.0);
    let y = get_f32_prop(node, "y").unwrap_or(0.0);
    let width = get_f32_prop(node, "width").unwrap_or(100.0);
    let height = get_f32_prop(node, "height").unwrap_or(100.0);

    // Parse clip property for frames
    let clip_children = node
        .get("clip")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut shape = Shape::new(
        kind,
        node::CanvasPoint::new(x, y),
        node::CanvasSize::new(width, height),
    );
    shape.id = id;
    shape.parent = parent_id;
    shape.clip_children = clip_children;

    // Collect child shape IDs (we'll parse them after adding this shape)
    let mut child_ids = Vec::new();

    // Parse children (fill, stroke, radius, and nested shapes)
    if let Some(children) = node.children() {
        for child in children.nodes() {
            match child.name().value() {
                "fill" => {
                    let h = get_f32_prop(child, "h").unwrap_or(0.0);
                    let s = get_f32_prop(child, "s").unwrap_or(0.0);
                    let l = get_f32_prop(child, "l").unwrap_or(0.0);
                    let a = get_f32_prop(child, "a").unwrap_or(1.0);
                    shape.fill = Some(Fill::new(gpui::Hsla { h, s, l, a }));
                }
                "stroke" => {
                    let width = get_f32_prop(child, "width").unwrap_or(1.0);
                    let h = get_f32_prop(child, "h").unwrap_or(0.0);
                    let s = get_f32_prop(child, "s").unwrap_or(0.0);
                    let l = get_f32_prop(child, "l").unwrap_or(0.0);
                    let a = get_f32_prop(child, "a").unwrap_or(1.0);
                    shape.stroke = Some(Stroke::new(gpui::Hsla { h, s, l, a }, width));
                }
                "radius" => {
                    if let Some(entry) = child.entries().first() {
                        if let Some(v) = entry.value().as_float() {
                            shape.corner_radius = v as f32;
                        }
                    }
                }
                // Nested shapes (rect, ellipse, frame)
                "rect" | "ellipse" | "frame" => {
                    // We'll parse these after adding the parent shape
                    // Just note we have child nodes to process
                }
                _ => {}
            }
        }
    }

    // Add this shape to the list
    shapes.push(shape);

    // Now recursively parse child shapes (must be done after parent is added)
    if let Some(children) = node.children() {
        for child in children.nodes() {
            match child.name().value() {
                "rect" | "ellipse" | "frame" => {
                    let child_id = parse_shape_recursive(child, Some(id), shapes)?;
                    child_ids.push(child_id);
                }
                _ => {}
            }
        }
    }

    // Update parent's children list (find parent in shapes and update)
    if !child_ids.is_empty() {
        if let Some(parent_shape) = shapes.iter_mut().find(|s| s.id == id) {
            parent_shape.children = child_ids;
        }
    }

    Ok(id)
}

fn get_f32_prop(node: &KdlNode, name: &str) -> Option<f32> {
    node.get(name)
        .and_then(|v| v.as_float())
        .map(|v| v as f32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_roundtrip() {
        let shapes = vec![
            Shape::rectangle(Vec2::new(100.0, 100.0), Vec2::new(150.0, 100.0))
                .with_fill(gpui::Hsla {
                    h: 0.5,
                    s: 0.8,
                    l: 0.5,
                    a: 1.0,
                })
                .with_corner_radius(8.0),
            Shape::ellipse(Vec2::new(300.0, 150.0), Vec2::new(120.0, 120.0)).with_stroke(
                gpui::Hsla {
                    h: 0.0,
                    s: 0.0,
                    l: 0.0,
                    a: 1.0,
                },
                2.0,
            ),
        ];

        let doc = Document::new(shapes);
        let kdl = doc.to_kdl();

        println!("Generated KDL:\n{}", kdl);

        let parsed = Document::from_kdl(&kdl).expect("Failed to parse");

        assert_eq!(parsed.shapes.len(), 2);
        assert_eq!(parsed.shapes[0].kind, ShapeKind::Rectangle);
        assert_eq!(parsed.shapes[1].kind, ShapeKind::Ellipse);
    }
}
