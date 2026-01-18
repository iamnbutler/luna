use glam::Vec2;
use gpui::{Bounds, Point, Size};

/// Camera/viewport state for the canvas.
#[derive(Clone, Debug)]
pub struct Viewport {
    /// Pan offset in canvas coordinates
    pub offset: Vec2,
    /// Zoom level (1.0 = 100%)
    pub zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert a point from screen coordinates to canvas coordinates.
    pub fn screen_to_canvas(&self, screen_point: Point<f32>) -> Vec2 {
        Vec2::new(
            (screen_point.x / self.zoom) - self.offset.x,
            (screen_point.y / self.zoom) - self.offset.y,
        )
    }

    /// Convert a point from canvas coordinates to screen coordinates.
    pub fn canvas_to_screen(&self, canvas_point: Vec2) -> Point<f32> {
        Point::new(
            (canvas_point.x + self.offset.x) * self.zoom,
            (canvas_point.y + self.offset.y) * self.zoom,
        )
    }

    /// Convert a size from canvas to screen coordinates.
    pub fn canvas_to_screen_size(&self, size: Vec2) -> Size<f32> {
        Size {
            width: size.x * self.zoom,
            height: size.y * self.zoom,
        }
    }

    /// Convert canvas bounds to screen bounds.
    pub fn canvas_to_screen_bounds(&self, position: Vec2, size: Vec2) -> Bounds<f32> {
        let origin = self.canvas_to_screen(position);
        let size = self.canvas_to_screen_size(size);
        Bounds { origin, size }
    }

    /// Pan the viewport by a delta in screen coordinates.
    pub fn pan(&mut self, delta: Vec2) {
        self.offset += delta / self.zoom;
    }

    /// Zoom the viewport, keeping a screen point fixed.
    pub fn zoom_at(&mut self, screen_point: Point<f32>, factor: f32) {
        let old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.1, 10.0);

        // Adjust offset to keep the point under cursor fixed
        if self.zoom != old_zoom {
            self.offset.x = screen_point.x / self.zoom - (screen_point.x / old_zoom - self.offset.x);
            self.offset.y = screen_point.y / self.zoom - (screen_point.y / old_zoom - self.offset.y);
        }
    }

    /// Reset to default view.
    pub fn reset(&mut self) {
        self.offset = Vec2::ZERO;
        self.zoom = 1.0;
    }
}
