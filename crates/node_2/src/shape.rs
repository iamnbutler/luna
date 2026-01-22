use crate::coords::{CanvasDelta, CanvasPoint, CanvasSize};
use crate::ShapeId;
use glam::Vec2;
use gpui::Hsla;
use serde::{Deserialize, Serialize};

/// The kind of shape.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
    Frame,
}

impl Default for ShapeKind {
    fn default() -> Self {
        Self::Rectangle
    }
}

/// Fill style for a shape.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    pub color: Hsla,
}

impl Fill {
    pub fn new(color: Hsla) -> Self {
        Self { color }
    }

    pub fn none() -> Option<Self> {
        None
    }
}

/// Stroke style for a shape.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub color: Hsla,
    pub width: f32,
}

impl Stroke {
    pub fn new(color: Hsla, width: f32) -> Self {
        Self { color, width }
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            color: gpui::black(),
            width: 2.0,
        }
    }
}

/// A shape on the canvas.
///
/// Shapes can be hierarchical - frames contain child shapes with
/// relative positioning. Z-order is determined by position in the
/// containing list, with children always rendering on top of parents.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shape {
    pub id: ShapeId,
    pub kind: ShapeKind,

    // Geometry (using typed coordinates)
    /// Position in canvas space (for root shapes) or relative to parent (for children)
    pub position: CanvasPoint,
    pub size: CanvasSize,

    // Hierarchy
    /// Parent shape ID (None for root-level shapes)
    pub parent: Option<ShapeId>,
    /// Child shape IDs (only meaningful for Frame shapes)
    pub children: Vec<ShapeId>,
    /// Whether to clip children to this shape's bounds (only for Frames)
    pub clip_children: bool,

    // Style
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub corner_radius: f32,
}

impl Shape {
    pub fn new(kind: ShapeKind, position: CanvasPoint, size: CanvasSize) -> Self {
        Self {
            id: ShapeId::new(),
            kind,
            position,
            size,
            parent: None,
            children: Vec::new(),
            clip_children: false,
            fill: None,
            stroke: Some(Stroke::default()),
            corner_radius: 0.0,
        }
    }

    pub fn rectangle(position: Vec2, size: Vec2) -> Self {
        Self::new(ShapeKind::Rectangle, CanvasPoint(position), CanvasSize(size))
    }

    pub fn ellipse(position: Vec2, size: Vec2) -> Self {
        Self::new(ShapeKind::Ellipse, CanvasPoint(position), CanvasSize(size))
    }

    pub fn frame(position: Vec2, size: Vec2) -> Self {
        let mut shape = Self::new(ShapeKind::Frame, CanvasPoint(position), CanvasSize(size));
        shape.clip_children = true; // Frames clip by default
        shape
    }

    pub fn with_fill(mut self, color: Hsla) -> Self {
        self.fill = Some(Fill::new(color));
        self
    }

    pub fn with_stroke(mut self, color: Hsla, width: f32) -> Self {
        self.stroke = Some(Stroke::new(color, width));
        self
    }

    pub fn with_corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    pub fn with_clip_children(mut self, clip: bool) -> Self {
        self.clip_children = clip;
        self
    }

    /// Get world position (canvas space) accounting for parent chain.
    ///
    /// For root shapes, returns `self.position`.
    /// For child shapes, walks up the parent chain to compute absolute position.
    pub fn world_position(&self, shapes: &[Shape]) -> CanvasPoint {
        match self.parent {
            None => self.position,
            Some(parent_id) => {
                let parent = shapes.iter().find(|s| s.id == parent_id);
                match parent {
                    Some(p) => {
                        let parent_world = p.world_position(shapes);
                        CanvasPoint(self.position.0 + parent_world.0)
                    }
                    None => self.position, // Orphaned child, treat as root
                }
            }
        }
    }

    /// Returns the bounding box as (min, max) corners in canvas space.
    pub fn bounds(&self) -> (CanvasPoint, CanvasPoint) {
        let max = CanvasPoint(self.position.0 + self.size.0);
        (self.position, max)
    }

    /// Check if a point is inside this shape's bounding box.
    pub fn contains_point(&self, point: CanvasPoint) -> bool {
        let (min, max) = self.bounds();
        point.0.x >= min.0.x && point.0.x <= max.0.x && point.0.y >= min.0.y && point.0.y <= max.0.y
    }

    /// Move the shape by a delta.
    pub fn translate(&mut self, delta: CanvasDelta) {
        self.position = self.position + delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_bounds() {
        let shape = Shape::rectangle(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));
        let (min, max) = shape.bounds();
        assert_eq!(min, CanvasPoint::new(10.0, 20.0));
        assert_eq!(max, CanvasPoint::new(110.0, 70.0));
    }

    #[test]
    fn test_contains_point() {
        let shape = Shape::rectangle(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        assert!(shape.contains_point(CanvasPoint::new(50.0, 50.0)));
        assert!(shape.contains_point(CanvasPoint::new(0.0, 0.0)));
        assert!(shape.contains_point(CanvasPoint::new(100.0, 100.0)));
        assert!(!shape.contains_point(CanvasPoint::new(-1.0, 50.0)));
        assert!(!shape.contains_point(CanvasPoint::new(101.0, 50.0)));
    }
}
