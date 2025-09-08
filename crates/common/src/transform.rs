//! Simple 2D transformation for canvas operations
//!
//! This module provides a minimal transformation structure for 2D canvas operations
//! that only support translation and uniform scaling - exactly what Luna needs.
//! No rotation, no skew, no complex matrix operations.

use glam::Vec2;

/// A simple 2D transformation that supports only translation and uniform scale.
///
/// This is all we need for Luna's canvas operations:
/// - Pan (scroll): translation offset
/// - Zoom: uniform scale factor
///
/// Memory usage: 12 bytes (vs 64 bytes for a 4x4 matrix)
/// Operations: 2 multiplies + 2 adds (vs 16 multiplies + 12 adds for matrix)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CanvasTransform {
    /// Translation offset (scroll position)
    pub offset: Vec2,
    /// Uniform scale factor (zoom level)
    pub scale: f32,
}

impl CanvasTransform {
    /// Creates an identity transform (no translation, no scaling)
    pub fn identity() -> Self {
        Self {
            offset: Vec2::ZERO,
            scale: 1.0,
        }
    }

    /// Creates a transform with only translation
    pub fn from_translation(offset: Vec2) -> Self {
        Self { offset, scale: 1.0 }
    }

    /// Creates a transform with only scaling
    pub fn from_scale(scale: f32) -> Self {
        Self {
            offset: Vec2::ZERO,
            scale,
        }
    }

    /// Creates a transform with both translation and scaling
    pub fn new(offset: Vec2, scale: f32) -> Self {
        Self { offset, scale }
    }

    /// Applies this transformation to a point
    ///
    /// Formula: point * scale + offset
    pub fn apply(&self, point: Vec2) -> Vec2 {
        point * self.scale + self.offset
    }

    /// Applies this transformation to a vector (direction/size)
    /// Vectors are not affected by translation, only by scale
    pub fn apply_vector(&self, vector: Vec2) -> Vec2 {
        vector * self.scale
    }

    /// Applies the inverse of this transformation to a point
    ///
    /// Formula: (point - offset) / scale
    pub fn apply_inverse(&self, point: Vec2) -> Vec2 {
        (point - self.offset) / self.scale
    }

    /// Applies the inverse of this transformation to a vector
    pub fn apply_inverse_vector(&self, vector: Vec2) -> Vec2 {
        vector / self.scale
    }

    /// Composes two transformations: first applies self, then other
    ///
    /// This is useful for parent-child relationships, though we'll
    /// mostly just add positions directly instead of composing transforms
    pub fn then(&self, other: &CanvasTransform) -> CanvasTransform {
        CanvasTransform {
            offset: other.apply(self.offset),
            scale: self.scale * other.scale,
        }
    }

    /// Returns the inverse transformation
    pub fn inverse(&self) -> CanvasTransform {
        CanvasTransform {
            offset: -self.offset / self.scale,
            scale: 1.0 / self.scale,
        }
    }
}

/// Viewport transformation for converting between window and canvas coordinates
///
/// This handles the common case of a centered viewport with zoom and pan
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ViewportTransform {
    /// The scroll position in canvas space
    pub scroll: Vec2,
    /// The zoom level (1.0 = 100%, 2.0 = 200%, etc.)
    pub zoom: f32,
    /// The center of the viewport in window space
    pub viewport_center: Vec2,
}

impl ViewportTransform {
    /// Creates a new viewport transform
    pub fn new(scroll: Vec2, zoom: f32, viewport_center: Vec2) -> Self {
        Self {
            scroll,
            zoom,
            viewport_center,
        }
    }

    /// Converts a point from window coordinates to canvas coordinates
    ///
    /// Steps:
    /// 1. Center the point relative to viewport center
    /// 2. Apply inverse zoom (divide by zoom)
    /// 3. Add scroll offset
    pub fn window_to_canvas(&self, window_pos: Vec2) -> Vec2 {
        ((window_pos - self.viewport_center) / self.zoom) + self.scroll
    }

    /// Converts a point from canvas coordinates to window coordinates
    ///
    /// Steps:
    /// 1. Subtract scroll offset
    /// 2. Apply zoom (multiply by zoom)
    /// 3. Uncenter relative to viewport center
    pub fn canvas_to_window(&self, canvas_pos: Vec2) -> Vec2 {
        ((canvas_pos - self.scroll) * self.zoom) + self.viewport_center
    }

    /// Converts a size/vector from window space to canvas space
    /// (not affected by scroll, only by zoom)
    pub fn window_to_canvas_vector(&self, window_vector: Vec2) -> Vec2 {
        window_vector / self.zoom
    }

    /// Converts a size/vector from canvas space to window space
    /// (not affected by scroll, only by zoom)
    pub fn canvas_to_window_vector(&self, canvas_vector: Vec2) -> Vec2 {
        canvas_vector * self.zoom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_transform() {
        let transform = CanvasTransform::identity();
        let point = Vec2::new(10.0, 20.0);

        assert_eq!(transform.apply(point), point);
        assert_eq!(transform.apply_inverse(point), point);
    }

    #[test]
    fn test_translation() {
        let transform = CanvasTransform::from_translation(Vec2::new(5.0, 10.0));
        let point = Vec2::new(10.0, 20.0);

        assert_eq!(transform.apply(point), Vec2::new(15.0, 30.0));
        assert_eq!(transform.apply_inverse(Vec2::new(15.0, 30.0)), point);
    }

    #[test]
    fn test_scale() {
        let transform = CanvasTransform::from_scale(2.0);
        let point = Vec2::new(10.0, 20.0);

        assert_eq!(transform.apply(point), Vec2::new(20.0, 40.0));
        assert_eq!(transform.apply_inverse(Vec2::new(20.0, 40.0)), point);
    }

    #[test]
    fn test_combined_transform() {
        let transform = CanvasTransform::new(Vec2::new(5.0, 10.0), 2.0);
        let point = Vec2::new(10.0, 20.0);

        // point * 2 + (5, 10) = (20, 40) + (5, 10) = (25, 50)
        assert_eq!(transform.apply(point), Vec2::new(25.0, 50.0));

        // Inverse should recover original
        assert_eq!(transform.apply_inverse(transform.apply(point)), point);
    }

    #[test]
    fn test_viewport_transform() {
        let viewport = ViewportTransform::new(
            Vec2::new(100.0, 200.0), // scroll
            2.0,                     // zoom
            Vec2::new(400.0, 300.0), // viewport center
        );

        let window_pos = Vec2::new(500.0, 400.0);

        // ((500-400, 400-300) / 2) + (100, 200) = (50, 50) + (100, 200) = (150, 250)
        let canvas_pos = viewport.window_to_canvas(window_pos);
        assert_eq!(canvas_pos, Vec2::new(150.0, 250.0));

        // Round trip should give us back the original
        assert_eq!(viewport.canvas_to_window(canvas_pos), window_pos);
    }

    #[test]
    fn test_transform_composition() {
        let parent = CanvasTransform::new(Vec2::new(10.0, 20.0), 2.0);
        let child = CanvasTransform::new(Vec2::new(5.0, 5.0), 1.5);

        let composed = parent.then(&child);

        // child applies first: offset (10, 20) * 1.5 + (5, 5) = (15, 30) + (5, 5) = (20, 35)
        assert_eq!(composed.offset, Vec2::new(20.0, 35.0));
        // scales multiply: 2.0 * 1.5 = 3.0
        assert_eq!(composed.scale, 3.0);
    }
}
