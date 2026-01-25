use crate::Viewport;
use glam::Vec2;
use gpui::{Context, EventEmitter, FocusHandle, Focusable, Hsla, Point};
use node_2::{
    compute_layout, CanvasDelta, CanvasPoint, CanvasSize, LayoutInput, ScreenPoint, Shape, ShapeId,
    ShapeKind, Stroke,
};
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
    Frame,
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
        start_mouse: CanvasPoint,
        start_positions: Vec<(ShapeId, CanvasPoint)>,
    },
    /// Resizing selected shapes
    ResizingShapes {
        handle: ResizeHandle,
        start_mouse: CanvasPoint,
        start_bounds: (CanvasPoint, CanvasPoint), // (min, max) in canvas coords
        shape_ids: Vec<ShapeId>,
        start_shape_data: Vec<(ShapeId, CanvasPoint, CanvasSize)>, // (id, position, size)
    },
    /// Drawing a new shape
    DrawingShape { shape_id: ShapeId, start: CanvasPoint },
    /// Panning the canvas
    Panning { last_screen_pos: ScreenPoint },
    /// Drag-selecting shapes
    Selecting { start: CanvasPoint },
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

    /// Find the deepest shape at a canvas point.
    ///
    /// For frames with children, recursively checks children first to find
    /// the deepest (most nested) shape at the point.
    pub fn shape_at_point(&self, point: CanvasPoint) -> Option<ShapeId> {
        self.shape_at_point_recursive(point, None)
    }

    /// Recursive helper for shape_at_point.
    /// If parent_id is Some, only checks children of that parent.
    fn shape_at_point_recursive(
        &self,
        point: CanvasPoint,
        parent_id: Option<ShapeId>,
    ) -> Option<ShapeId> {
        // Get shapes at this level (matching parent)
        let shapes_at_level: Vec<_> = self
            .shapes
            .iter()
            .filter(|s| s.parent == parent_id)
            .collect();

        // Iterate in reverse for z-order (top to bottom)
        for shape in shapes_at_level.iter().rev() {
            // Calculate world position for hit testing
            let world_pos = shape.world_position(&self.shapes);
            let world_bounds_min = world_pos;
            let world_bounds_max = CanvasPoint(world_pos.0 + shape.size.0);

            let hit = point.0.x >= world_bounds_min.0.x
                && point.0.x <= world_bounds_max.0.x
                && point.0.y >= world_bounds_min.0.y
                && point.0.y <= world_bounds_max.0.y;

            if hit {
                // If this shape has children, check them first (they render on top)
                if !shape.children.is_empty() {
                    if let Some(child_hit) = self.shape_at_point_recursive(point, Some(shape.id)) {
                        return Some(child_hit);
                    }
                }
                // No child hit, return this shape
                return Some(shape.id);
            }
        }

        None
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

    /// Add a shape as a child of a frame.
    ///
    /// Converts the child's position from absolute canvas coordinates to
    /// relative coordinates (relative to parent's origin).
    pub fn add_child(&mut self, child_id: ShapeId, parent_id: ShapeId, cx: &mut Context<Self>) {
        // Get parent's world position
        let parent_world = self
            .get_shape(parent_id)
            .map(|p| p.world_position(&self.shapes));

        let Some(parent_world) = parent_world else {
            return;
        };

        // Update child: convert position to relative and set parent
        if let Some(child) = self.get_shape_mut(child_id) {
            // Convert absolute position to relative (subtract parent's world position)
            let child_world = child.position;
            child.position = CanvasPoint(child_world.0 - parent_world.0);
            child.parent = Some(parent_id);
        }

        // Update parent: add child to children list
        if let Some(parent) = self.get_shape_mut(parent_id) {
            if !parent.children.contains(&child_id) {
                parent.children.push(child_id);
            }
        }

        // If parent has layout enabled, apply it to reposition children
        self.apply_layout_for_frame(parent_id);

        cx.emit(CanvasEvent::ContentChanged);
        cx.notify();
    }

    /// Remove a shape from its parent.
    ///
    /// Converts the child's position from relative coordinates back to
    /// absolute canvas coordinates.
    pub fn unparent(&mut self, child_id: ShapeId, cx: &mut Context<Self>) {
        // Get child's current parent and world position before unparenting
        let (parent_id, child_world) = {
            let Some(child) = self.get_shape(child_id) else {
                return;
            };
            let Some(parent_id) = child.parent else {
                return; // Already unparented
            };
            (parent_id, child.world_position(&self.shapes))
        };

        // Update child: convert position to absolute and clear parent
        if let Some(child) = self.get_shape_mut(child_id) {
            child.position = child_world;
            child.parent = None;
        }

        // Update parent: remove child from children list
        if let Some(parent) = self.get_shape_mut(parent_id) {
            parent.children.retain(|&id| id != child_id);
        }

        cx.emit(CanvasEvent::ContentChanged);
        cx.notify();
    }

    /// Find the topmost frame that fully contains a shape's bounds.
    ///
    /// Returns None if no frame contains the shape.
    fn find_containing_frame(&self, shape_id: ShapeId) -> Option<ShapeId> {
        let shape = self.get_shape(shape_id)?;
        let shape_min = shape.position.0;
        let shape_max = shape.position.0 + shape.size.0;

        // Find frames (in reverse z-order to get topmost first)
        self.shapes
            .iter()
            .rev()
            .filter(|s| s.kind == ShapeKind::Frame && s.id != shape_id)
            .find(|frame| {
                let frame_world = frame.world_position(&self.shapes);
                let frame_min = frame_world.0;
                let frame_max = frame_world.0 + frame.size.0;

                // Check if shape is fully inside frame
                shape_min.x >= frame_min.x
                    && shape_min.y >= frame_min.y
                    && shape_max.x <= frame_max.x
                    && shape_max.y <= frame_max.y
            })
            .map(|f| f.id)
    }

    /// Auto-parent a shape if it's fully inside a frame.
    ///
    /// Called after drawing a new shape to automatically nest it
    /// inside any containing frame.
    fn auto_parent_if_inside_frame(&mut self, shape_id: ShapeId, cx: &mut Context<Self>) {
        if let Some(frame_id) = self.find_containing_frame(shape_id) {
            self.add_child(shape_id, frame_id, cx);
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
        let offset = CanvasDelta::new(20.0, 20.0);
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
            shape.position = shape.position + offset;
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
    /// Shapes in autolayout frames are skipped.
    pub fn move_selected(&mut self, delta: CanvasDelta, cx: &mut Context<Self>) {
        // Collect IDs of shapes that are in autolayout (can't move)
        let in_layout: std::collections::HashSet<_> = self
            .selection
            .iter()
            .filter(|id| self.is_in_autolayout(**id))
            .copied()
            .collect();

        for shape in &mut self.shapes {
            if self.selection.contains(&shape.id) && !in_layout.contains(&shape.id) {
                shape.translate(delta);
            }
        }
        cx.emit(CanvasEvent::ContentChanged);
        cx.notify();
    }

    /// Start drawing a new shape.
    pub fn start_draw(&mut self, kind: ShapeKind, start: CanvasPoint, cx: &mut Context<Self>) {
        let mut shape = Shape::new(kind, start, CanvasSize::new(0.0, 0.0));
        shape.stroke = Some(self.default_stroke);
        shape.fill = self.default_fill.map(|c| node_2::Fill::new(c));
        // Frames clip children by default
        if kind == ShapeKind::Frame {
            shape.clip_children = true;
        }

        let id = shape.id;
        self.shapes.push(shape);
        self.drag = Some(DragState::DrawingShape {
            shape_id: id,
            start,
        });
        cx.notify();
    }

    /// Update the shape being drawn.
    pub fn update_draw(&mut self, current: CanvasPoint, cx: &mut Context<Self>) {
        // Copy data from drag state to avoid borrow issues
        let drag_info = match &self.drag {
            Some(DragState::DrawingShape { shape_id, start }) => Some((*shape_id, *start)),
            _ => None,
        };

        if let Some((shape_id, start)) = drag_info {
            if let Some(shape) = self.get_shape_mut(shape_id) {
                // Calculate size and position (handle negative drag)
                let min_x = start.x().min(current.x());
                let min_y = start.y().min(current.y());
                let max_x = start.x().max(current.x());
                let max_y = start.y().max(current.y());
                shape.position = CanvasPoint::new(min_x, min_y);
                shape.size = CanvasSize::new(max_x - min_x, max_y - min_y);
                cx.notify();
            }
        }
    }

    /// Finish drawing a shape.
    pub fn finish_draw(&mut self, cx: &mut Context<Self>) {
        if let Some(DragState::DrawingShape { shape_id, .. }) = self.drag.take() {
            // Auto-parent to containing frame if applicable
            self.auto_parent_if_inside_frame(shape_id, cx);

            // Select the newly drawn shape
            self.selection.clear();
            self.selection.insert(shape_id);
            // Switch back to Select tool
            self.tool = Tool::Select;
            cx.emit(CanvasEvent::ShapeAdded(shape_id));
            cx.emit(CanvasEvent::SelectionChanged);
            cx.emit(CanvasEvent::ContentChanged);
            cx.notify();
        }
    }

    /// Start moving selected shapes.
    /// Returns false if move was blocked (e.g., shapes in autolayout).
    pub fn start_move(&mut self, start_mouse: CanvasPoint, _cx: &mut Context<Self>) -> bool {
        // Block moving shapes that are children of autolayout frames
        if self.selection_in_autolayout() {
            return false;
        }

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
        true
    }

    /// Update shape positions during move.
    pub fn update_move(&mut self, current_mouse: CanvasPoint, cx: &mut Context<Self>) {
        // Copy data to avoid borrow issues
        let (start_mouse, positions): (CanvasPoint, Vec<_>) = match &self.drag {
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
    /// Returns false if resize was blocked (e.g., shapes in autolayout).
    pub fn start_resize(&mut self, handle: ResizeHandle, start_mouse: CanvasPoint, _cx: &mut Context<Self>) -> bool {
        // Block resizing shapes that are children of autolayout frames
        if self.selection_in_autolayout() {
            return false;
        }

        let Some((min, max)) = self.selection_bounds() else {
            return false;
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
        true
    }

    /// Update shape sizes during resize.
    pub fn update_resize(&mut self, current_mouse: CanvasPoint, cx: &mut Context<Self>) {
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

        // Extract raw Vec2 values for math operations
        let (start_min, start_max) = (start_bounds.0 .0, start_bounds.1 .0);
        let start_size = start_max - start_min;
        let start_mouse = start_mouse.0;
        let current_mouse = current_mouse.0;

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
                // Extract raw values
                let orig_pos = orig_pos.0;
                let orig_size = orig_size.0;

                // Calculate relative position within original bounds (0 to 1)
                let rel_pos = orig_pos - start_min;

                // Apply flip: if flipped, mirror the relative position
                let rel_pos = Vec2::new(
                    if flip_x { start_size.x - rel_pos.x - orig_size.x } else { rel_pos.x },
                    if flip_y { start_size.y - rel_pos.y - orig_size.y } else { rel_pos.y },
                );

                // Scale position and size
                shape.position = CanvasPoint(new_min + rel_pos * scale);
                shape.size = CanvasSize(orig_size * scale);
            }
        }
        cx.notify();
    }

    /// Finish resizing shapes.
    pub fn finish_resize(&mut self, cx: &mut Context<Self>) {
        if let Some(DragState::ResizingShapes { shape_ids, .. }) = self.drag.take() {
            // If any resized shapes are frames with layout, reapply their layout
            for shape_id in &shape_ids {
                if self.shapes.iter().any(|s| s.id == *shape_id && s.has_layout()) {
                    self.apply_layout_for_frame(*shape_id);
                }
            }
            cx.emit(CanvasEvent::ContentChanged);
            cx.notify();
        }
    }

    /// Start panning.
    pub fn start_pan(&mut self, screen_pos: ScreenPoint) {
        self.drag = Some(DragState::Panning { last_screen_pos: screen_pos });
    }

    /// Update pan with new screen position.
    pub fn update_pan(&mut self, current_screen_pos: ScreenPoint, cx: &mut Context<Self>) {
        if let Some(DragState::Panning { last_screen_pos }) = &mut self.drag {
            let delta = current_screen_pos.0 - last_screen_pos.0;
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

    /// Check if a shape is a child of a layout-enabled frame.
    pub fn is_in_autolayout(&self, shape_id: ShapeId) -> bool {
        self.shapes
            .iter()
            .find(|s| s.id == shape_id)
            .map(|s| s.is_in_layout(&self.shapes))
            .unwrap_or(false)
    }

    /// Check if any selected shapes are in autolayout.
    pub fn selection_in_autolayout(&self) -> bool {
        self.selection.iter().any(|id| self.is_in_autolayout(*id))
    }

    /// Get the bounding box of selected shapes in canvas coordinates.
    pub fn selection_bounds(&self) -> Option<(CanvasPoint, CanvasPoint)> {
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
            min = min.min(shape_min.0);
            max = max.max(shape_max.0);
        }

        Some((CanvasPoint(min), CanvasPoint(max)))
    }

    /// Apply layout for a single frame to its children.
    /// Does nothing if the frame doesn't have layout enabled.
    pub fn apply_layout_for_frame(&mut self, frame_id: ShapeId) {
        // Gather frame info and children order
        let (frame_size, layout, children_ids) = {
            let Some(frame) = self.shapes.iter().find(|s| s.id == frame_id) else {
                return;
            };
            let Some(layout) = frame.layout.clone() else {
                return; // No layout enabled
            };
            (frame.size, layout, frame.children.clone())
        };

        // Gather children info in the order specified by frame.children
        let child_inputs: Vec<LayoutInput> = children_ids
            .iter()
            .filter_map(|child_id| {
                self.shapes.iter().find(|s| s.id == *child_id).map(|child| LayoutInput {
                    id: child.id,
                    size: child.size,
                    width_mode: child.child_layout.width_mode,
                    height_mode: child.child_layout.height_mode,
                })
            })
            .collect();

        if child_inputs.is_empty() {
            return;
        }

        // Compute layout
        let outputs = compute_layout(frame_size, &layout, &child_inputs);

        // Apply results to children
        for output in outputs {
            if let Some(child) = self.shapes.iter_mut().find(|s| s.id == output.id) {
                child.position = output.position;
                child.size = output.size;
            }
        }
    }

    /// Apply layout for all frames that have layout enabled.
    pub fn apply_all_layouts(&mut self) {
        // Collect frame IDs (we need to avoid borrowing issues)
        let frame_ids: Vec<ShapeId> = self
            .shapes
            .iter()
            .filter(|s| s.has_layout())
            .map(|s| s.id)
            .collect();

        // Process each frame
        for frame_id in frame_ids {
            self.apply_layout_for_frame(frame_id);
        }
    }
}

impl EventEmitter<CanvasEvent> for Canvas {}

impl Focusable for Canvas {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
