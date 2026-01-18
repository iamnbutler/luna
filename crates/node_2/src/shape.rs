use crate::ShapeId;
use glam::Vec2;
use gpui::Hsla;
use serde::{Deserialize, Serialize};

/// The kind of shape.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
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
/// Shapes are flat (no hierarchy). Z-order is determined by
/// position in the containing list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shape {
    pub id: ShapeId,
    pub kind: ShapeKind,

    // Geometry
    pub position: Vec2,
    pub size: Vec2,

    // Style
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub corner_radius: f32,
}

impl Shape {
    pub fn new(kind: ShapeKind, position: Vec2, size: Vec2) -> Self {
        Self {
            id: ShapeId::new(),
            kind,
            position,
            size,
            fill: None,
            stroke: Some(Stroke::default()),
            corner_radius: 0.0,
        }
    }

    pub fn rectangle(position: Vec2, size: Vec2) -> Self {
        Self::new(ShapeKind::Rectangle, position, size)
    }

    pub fn ellipse(position: Vec2, size: Vec2) -> Self {
        Self::new(ShapeKind::Ellipse, position, size)
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

    /// Returns the bounding box as (min, max) corners.
    pub fn bounds(&self) -> (Vec2, Vec2) {
        (self.position, self.position + self.size)
    }

    /// Check if a point is inside this shape's bounding box.
    pub fn contains_point(&self, point: Vec2) -> bool {
        let (min, max) = self.bounds();
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }

    /// Move the shape by a delta.
    pub fn translate(&mut self, delta: Vec2) {
        self.position += delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_bounds() {
        let shape = Shape::rectangle(Vec2::new(10.0, 20.0), Vec2::new(100.0, 50.0));
        let (min, max) = shape.bounds();
        assert_eq!(min, Vec2::new(10.0, 20.0));
        assert_eq!(max, Vec2::new(110.0, 70.0));
    }

    #[test]
    fn test_contains_point() {
        let shape = Shape::rectangle(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        assert!(shape.contains_point(Vec2::new(50.0, 50.0)));
        assert!(shape.contains_point(Vec2::new(0.0, 0.0)));
        assert!(shape.contains_point(Vec2::new(100.0, 100.0)));
        assert!(!shape.contains_point(Vec2::new(-1.0, 50.0)));
        assert!(!shape.contains_point(Vec2::new(101.0, 50.0)));
    }
}
