use crate::Viewport;
use glam::Vec2;
use gpui::{Context, EventEmitter, FocusHandle, Focusable, Hsla, Point};
use node_2::{Shape, ShapeId, ShapeKind, Stroke};
use std::collections::HashSet;
use theme_2::Theme;

/// Current tool mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Select,
    Pan,
    Rectangle,
    Ellipse,
}

/// Events emitted by the canvas.
#[derive(Clone, Debug)]
pub enum CanvasEvent {
    ShapeAdded(ShapeId),
    ShapeRemoved(ShapeId),
    SelectionChanged,
    ContentChanged,
}

/// Resize handle positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl ResizeHandle {
    /// All handles in order.
    pub const ALL: [ResizeHandle; 4] = [
        ResizeHandle::TopLeft,
        ResizeHandle::TopRight,
        ResizeHandle::BottomLeft,
        ResizeHandle::BottomRight,
    ];
}

/// Active drag operation.
#[derive(Clone, Debug)]
pub enum DragState {
    /// Dragging selected shapes
    MovingShapes {
        start_mouse: Vec2,
        start_positions: Vec<(ShapeId, Vec2)>,
    },
    /// Resizing selected shapes
    ResizingShapes {
        handle: ResizeHandle,
        start_mouse: Vec2,
        start_bounds: (Vec2, Vec2), // (min, max) in canvas coords
        shape_ids: Vec<ShapeId>,
        start_shape_data: Vec<(ShapeId, Vec2, Vec2)>, // (id, position, size)
    },
    /// Drawing a new shape
    DrawingShape { shape_id: ShapeId, start: Vec2 },
    /// Panning the canvas
    Panning { last_screen_pos: Vec2 },
    /// Drag-selecting shapes
    Selecting { start: Vec2 },
}

/// The canvas state.
pub struct Canvas {
    /// All shapes on the canvas, in z-order (back to front).
    pub shapes: Vec<Shape>,

    /// Currently selected shape IDs.
    pub selection: HashSet<ShapeId>,

    /// Currently hovered shape (for hover effects).
    pub hovered: Option<ShapeId>,

    /// Viewport (pan/zoom) state.
    pub viewport: Viewport,

    /// Current tool.
    pub tool: Tool,

    /// Active drag operation.
    pub drag: Option<DragState>,

    /// Default stroke for new shapes.
    pub default_stroke: Stroke,

    /// Default fill for new shapes.
    pub default_fill: Option<Hsla>,

    /// Theme colors.
    pub theme: Theme,

    /// Focus handle for keyboard events.
    focus_handle: FocusHandle,
}

impl Canvas {
    pub fn new(theme: Theme, cx: &mut Context<Self>) -> Self {
        Self {
            shapes: Vec::new(),
            selection: HashSet::new(),
            hovered: None,
            viewport: Viewport::new(),
            tool: Tool::Select,
            drag: None,
            default_stroke: Stroke::new(theme.default_stroke, 2.0),
            default_fill: None,
            theme,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Add a shape to the canvas.
    pub fn add_shape(&mut self, shape: Shape, cx: &mut Context<Self>) {
        let id = shape.id;
        self.shapes.push(shape);
        cx.emit(CanvasEvent::ShapeAdded(id));
        cx.emit(CanvasEvent::ContentChanged);
        cx.notify();
    }

    /// Remove a shape from the canvas.
    pub fn remove_shape(&mut self, id: ShapeId, cx: &mut Context<Self>) {
        if let Some(pos) = self.shapes.iter().position(|s| s.id == id) {
            self.shapes.remove(pos);
            self.selection.remove(&id);
            cx.emit(CanvasEvent::ShapeRemoved(id));
            cx.emit(CanvasEvent::ContentChanged);
            cx.notify();
        }
    }

    /// Get a shape by ID.
    pub fn get_shape(&self, id: ShapeId) -> Option<&Shape> {
        self.shapes.iter().find(|s| s.id == id)
    }

    /// Get a mutable shape by ID.
    pub fn get_shape_mut(&mut self, id: ShapeId) -> Option<&mut Shape> {
        self.shapes.iter_mut().find(|s| s.id == id)
    }

    /// Find the topmost shape at a canvas point.
    pub fn shape_at_point(&self, point: Vec2) -> Option<ShapeId> {
        // Iterate in reverse for z-order (top to bottom)
        self.shapes
            .iter()
            .rev()
            .find(|s| s.contains_point(point))
            .map(|s| s.id)
    }

    /// Select a shape, optionally adding to selection.
    pub fn select(&mut self, id: ShapeId, add_to_selection: bool, cx: &mut Context<Self>) {
        if !add_to_selection {
            self.selection.clear();
        }
        self.selection.insert(id);
        cx.emit(CanvasEvent::SelectionChanged);
        cx.notify();
    }

    /// Clear the selection.
    pub fn clear_selection(&mut self, cx: &mut Context<Self>) {
        if !self.selection.is_empty() {
            self.selection.clear();
            cx.emit(CanvasEvent::SelectionChanged);
            cx.notify();
        }
    }

    /// Delete selected shapes.
    pub fn delete_selected(&mut self, cx: &mut Context<Self>) {
        let to_remove: Vec<_> = self.selection.iter().copied().collect();
        for id in to_remove {
            self.remove_shape(id, cx);
        }
    }

    /// Duplicate selected shapes with a slight offset.
    pub fn duplicate_selected(&mut self, cx: &mut Context<Self>) {
        let offset = Vec2::new(20.0, 20.0);
        let to_duplicate: Vec<_> = self
            .shapes
            .iter()
            .filter(|s| self.selection.contains(&s.id))
            .cloned()
            .collect();

        if to_duplicate.is_empty() {
            return;
        }

        // Clear current selection
        self.selection.clear();

        // Create duplicates with new IDs and offset positions
        for mut shape in to_duplicate {
            shape.id = ShapeId::new();
            shape.position += offset;
            let new_id = shape.id;
            self.shapes.push(shape);
            self.selection.insert(new_id);
            cx.emit(CanvasEvent::ShapeAdded(new_id));
        }

        cx.emit(CanvasEvent::SelectionChanged);
        cx.emit(CanvasEvent::ContentChanged);
        cx.notify();
    }

    /// Move selected shapes by a delta.
    pub fn move_selected(&mut self, delta: Vec2, cx: &mut Context<Self>) {
        for shape in &mut self.shapes {
            if self.selection.contains(&shape.id) {
                shape.translate(delta);
            }
        }
        cx.emit(CanvasEvent::ContentChanged);
        cx.notify();
    }

    /// Start drawing a new shape.
    pub fn start_draw(&mut self, kind: ShapeKind, start: Vec2, cx: &mut Context<Self>) {
        let mut shape = Shape::new(kind, start, Vec2::ZERO);
        shape.stroke = Some(self.default_stroke);
        shape.fill = self.default_fill.map(|c| node_2::Fill::new(c));

        let id = shape.id;
        self.shapes.push(shape);
        self.drag = Some(DragState::DrawingShape {
            shape_id: id,
            start,
        });
        cx.notify();
    }

    /// Update the shape being drawn.
    pub fn update_draw(&mut self, current: Vec2, cx: &mut Context<Self>) {
        // Copy data from drag state to avoid borrow issues
        let drag_info = match &self.drag {
            Some(DragState::DrawingShape { shape_id, start }) => Some((*shape_id, *start)),
            _ => None,
        };

        if let Some((shape_id, start)) = drag_info {
            if let Some(shape) = self.get_shape_mut(shape_id) {
                // Calculate size and position (handle negative drag)
                let min = Vec2::new(start.x.min(current.x), start.y.min(current.y));
                let max = Vec2::new(start.x.max(current.x), start.y.max(current.y));
                shape.position = min;
                shape.size = max - min;
                cx.notify();
            }
        }
    }

    /// Finish drawing a shape.
    pub fn finish_draw(&mut self, cx: &mut Context<Self>) {
        if let Some(DragState::DrawingShape { shape_id, .. }) = self.drag.take() {
            // Select the newly drawn shape
            self.selection.clear();
            self.selection.insert(shape_id);
            cx.emit(CanvasEvent::ShapeAdded(shape_id));
            cx.emit(CanvasEvent::SelectionChanged);
            cx.emit(CanvasEvent::ContentChanged);
            cx.notify();
        }
    }

    /// Start moving selected shapes.
    pub fn start_move(&mut self, start_mouse: Vec2, _cx: &mut Context<Self>) {
        let positions: Vec<_> = self
            .shapes
            .iter()
            .filter(|s| self.selection.contains(&s.id))
            .map(|s| (s.id, s.position))
            .collect();

        self.drag = Some(DragState::MovingShapes {
            start_mouse,
            start_positions: positions,
        });
    }

    /// Update shape positions during move.
    pub fn update_move(&mut self, current_mouse: Vec2, cx: &mut Context<Self>) {
        // Copy data to avoid borrow issues
        let (start_mouse, positions): (Vec2, Vec<_>) = match &self.drag {
            Some(DragState::MovingShapes { start_mouse, start_positions }) => {
                (*start_mouse, start_positions.clone())
            }
            _ => return,
        };

        let delta = current_mouse - start_mouse;

        for (id, start_pos) in positions {
            if let Some(shape) = self.get_shape_mut(id) {
                shape.position = start_pos + delta;
            }
        }
        cx.notify();
    }

    /// Finish moving shapes.
    pub fn finish_move(&mut self, cx: &mut Context<Self>) {
        if matches!(self.drag, Some(DragState::MovingShapes { .. })) {
            self.drag = None;
            cx.emit(CanvasEvent::ContentChanged);
            cx.notify();
        }
    }

    /// Start resizing selected shapes.
    pub fn start_resize(&mut self, handle: ResizeHandle, start_mouse: Vec2, _cx: &mut Context<Self>) {
        let Some((min, max)) = self.selection_bounds() else {
            return;
        };

        let shape_ids: Vec<_> = self.selection.iter().copied().collect();
        let start_shape_data: Vec<_> = self
            .shapes
            .iter()
            .filter(|s| self.selection.contains(&s.id))
            .map(|s| (s.id, s.position, s.size))
            .collect();

        self.drag = Some(DragState::ResizingShapes {
            handle,
            start_mouse,
            start_bounds: (min, max),
            shape_ids,
            start_shape_data,
        });
    }

    /// Update shape sizes during resize.
    pub fn update_resize(&mut self, current_mouse: Vec2, cx: &mut Context<Self>) {
        let (handle, start_mouse, start_bounds, start_shape_data) = match &self.drag {
            Some(DragState::ResizingShapes {
                handle,
                start_mouse,
                start_bounds,
                start_shape_data,
                ..
            }) => (*handle, *start_mouse, *start_bounds, start_shape_data.clone()),
            _ => return,
        };

        let (start_min, start_max) = start_bounds;
        let start_size = start_max - start_min;

        // Calculate new bounds based on which handle is being dragged
        let delta = current_mouse - start_mouse;
        let (raw_min, raw_max) = match handle {
            ResizeHandle::TopLeft => (start_min + delta, start_max),
            ResizeHandle::TopRight => (
                Vec2::new(start_min.x, start_min.y + delta.y),
                Vec2::new(start_max.x + delta.x, start_max.y),
            ),
            ResizeHandle::BottomLeft => (
                Vec2::new(start_min.x + delta.x, start_min.y),
                Vec2::new(start_max.x, start_max.y + delta.y),
            ),
            ResizeHandle::BottomRight => (start_min, start_max + delta),
        };

        // Handle axis flipping - normalize so min < max on each axis
        let new_min = Vec2::new(raw_min.x.min(raw_max.x), raw_min.y.min(raw_max.y));
        let new_max = Vec2::new(raw_min.x.max(raw_max.x), raw_min.y.max(raw_max.y));

        // Check if axes are flipped relative to original
        let flip_x = raw_min.x > raw_max.x;
        let flip_y = raw_min.y > raw_max.y;

        // Ensure minimum size
        let min_size = 1.0;
        let new_size = (new_max - new_min).max(Vec2::splat(min_size));

        // Scale factor (use absolute values since we handle flipping separately)
        let scale = Vec2::new(
            if start_size.x > 0.0 { new_size.x / start_size.x } else { 1.0 },
            if start_size.y > 0.0 { new_size.y / start_size.y } else { 1.0 },
        );

        // Update each shape proportionally
        for (id, orig_pos, orig_size) in start_shape_data {
            if let Some(shape) = self.get_shape_mut(id) {
                // Calculate relative position within original bounds (0 to 1)
                let rel_pos = orig_pos - start_min;

                // Apply flip: if flipped, mirror the relative position
                let rel_pos = Vec2::new(
                    if flip_x { start_size.x - rel_pos.x - orig_size.x } else { rel_pos.x },
                    if flip_y { start_size.y - rel_pos.y - orig_size.y } else { rel_pos.y },
                );

                // Scale position and size
                shape.position = new_min + rel_pos * scale;
                shape.size = orig_size * scale;
            }
        }
        cx.notify();
    }

    /// Finish resizing shapes.
    pub fn finish_resize(&mut self, cx: &mut Context<Self>) {
        if matches!(self.drag, Some(DragState::ResizingShapes { .. })) {
            self.drag = None;
            cx.emit(CanvasEvent::ContentChanged);
            cx.notify();
        }
    }

    /// Start panning.
    pub fn start_pan(&mut self, screen_pos: Vec2) {
        self.drag = Some(DragState::Panning { last_screen_pos: screen_pos });
    }

    /// Update pan with new screen position.
    pub fn update_pan(&mut self, current_screen_pos: Vec2, cx: &mut Context<Self>) {
        if let Some(DragState::Panning { last_screen_pos }) = &mut self.drag {
            let delta = current_screen_pos - *last_screen_pos;
            self.viewport.pan(delta);
            *last_screen_pos = current_screen_pos;
            cx.notify();
        }
    }

    /// Finish panning.
    pub fn finish_pan(&mut self) {
        if matches!(self.drag, Some(DragState::Panning { .. })) {
            self.drag = None;
        }
    }

    /// Zoom at a screen point.
    pub fn zoom_at(&mut self, screen_point: Point<f32>, factor: f32, cx: &mut Context<Self>) {
        self.viewport.zoom_at(screen_point, factor);
        cx.notify();
    }

    /// Get the bounding box of selected shapes in canvas coordinates.
    pub fn selection_bounds(&self) -> Option<(Vec2, Vec2)> {
        let selected: Vec<_> = self
            .shapes
            .iter()
            .filter(|s| self.selection.contains(&s.id))
            .collect();

        if selected.is_empty() {
            return None;
        }

        let mut min = Vec2::new(f32::MAX, f32::MAX);
        let mut max = Vec2::new(f32::MIN, f32::MIN);

        for shape in selected {
            let (shape_min, shape_max) = shape.bounds();
            min = min.min(shape_min);
            max = max.max(shape_max);
        }

        Some((min, max))
    }
}

impl EventEmitter<CanvasEvent> for Canvas {}

impl Focusable for Canvas {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
