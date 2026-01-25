use crate::coords::{CanvasDelta, CanvasPoint, CanvasSize};
use crate::layout::{ChildLayout, FrameLayout};
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
    /// Position in canvas space (for root shapes) or relative to parent (for children).
    /// This is the user-specified position, which may be overridden by layout.
    pub position: CanvasPoint,
    /// User-specified size. May be overridden by layout (see computed_size).
    pub size: CanvasSize,

    // Layout-computed geometry (set by layout engine, cleared when layout is disabled)
    /// Position computed by the layout engine. When Some, this overrides `position` for rendering.
    pub computed_position: Option<CanvasPoint>,
    /// Size computed by the layout engine. When Some, this overrides `size` for rendering.
    pub computed_size: Option<CanvasSize>,

    // Hierarchy
    /// Parent shape ID (None for root-level shapes)
    pub parent: Option<ShapeId>,
    /// Child shape IDs (only meaningful for Frame shapes)
    pub children: Vec<ShapeId>,
    /// Whether to clip children to this shape's bounds (only for Frames)
    pub clip_children: bool,

    // Layout
    /// Autolayout configuration (only meaningful for Frame shapes).
    /// When Some, children are positioned automatically by the layout engine.
    /// When None, children use manual absolute positioning.
    pub layout: Option<FrameLayout>,
    /// Child-specific layout settings.
    /// Controls how this shape behaves when it's a child of a layout frame.
    pub child_layout: ChildLayout,

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
            computed_position: None,
            computed_size: None,
            parent: None,
            children: Vec::new(),
            clip_children: false,
            layout: None,
            child_layout: ChildLayout::default(),
            fill: None,
            stroke: Some(Stroke::default()),
            corner_radius: 0.0,
        }
    }

    /// Get the effective position (computed if in layout, otherwise user-specified).
    pub fn effective_position(&self) -> CanvasPoint {
        self.computed_position.unwrap_or(self.position)
    }

    /// Get the effective size (computed if in layout, otherwise user-specified).
    pub fn effective_size(&self) -> CanvasSize {
        self.computed_size.unwrap_or(self.size)
    }

    /// Check if position is computed (differs from user-specified).
    pub fn has_computed_position(&self) -> bool {
        self.computed_position.is_some()
    }

    /// Check if size is computed (differs from user-specified).
    pub fn has_computed_size(&self) -> bool {
        self.computed_size.is_some()
    }

    /// Clear computed values (call when layout is disabled or shape leaves layout).
    pub fn clear_computed(&mut self) {
        self.computed_position = None;
        self.computed_size = None;
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

    /// Enable autolayout on this frame.
    pub fn with_layout(mut self, layout: FrameLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    /// Set child layout settings (how this shape behaves in a parent layout).
    pub fn with_child_layout(mut self, child_layout: ChildLayout) -> Self {
        self.child_layout = child_layout;
        self
    }

    /// Check if this shape has autolayout enabled.
    pub fn has_layout(&self) -> bool {
        self.layout.is_some()
    }

    /// Check if this shape is in a layout (has a parent with layout).
    pub fn is_in_layout(&self, shapes: &[Shape]) -> bool {
        match self.parent {
            None => false,
            Some(parent_id) => shapes
                .iter()
                .find(|s| s.id == parent_id)
                .map(|p| p.has_layout())
                .unwrap_or(false),
        }
    }

    /// Get world position (canvas space) accounting for parent chain.
    ///
    /// For root shapes, returns effective position.
    /// For child shapes, walks up the parent chain to compute absolute position.
    /// Uses computed position if available (from layout).
    pub fn world_position(&self, shapes: &[Shape]) -> CanvasPoint {
        let pos = self.effective_position();
        match self.parent {
            None => pos,
            Some(parent_id) => {
                let parent = shapes.iter().find(|s| s.id == parent_id);
                match parent {
                    Some(p) => {
                        let parent_world = p.world_position(shapes);
                        CanvasPoint(pos.0 + parent_world.0)
                    }
                    None => pos, // Orphaned child, treat as root
                }
            }
        }
    }

    /// Returns the bounding box as (min, max) corners using effective position/size.
    pub fn bounds(&self) -> (CanvasPoint, CanvasPoint) {
        let pos = self.effective_position();
        let size = self.effective_size();
        let max = CanvasPoint(pos.0 + size.0);
        (pos, max)
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
