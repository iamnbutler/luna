//! Properties panel for selected shapes.

use crate::components::{h_stack, panel, v_stack};
use crate::input::{input, InputColors, InputState, InputStateEvent};
use canvas_2::{Canvas, CanvasEvent};
use gpui::{
    div, px, AppContext, Context, Entity, Focusable, Hsla, InteractiveElement, IntoElement,
    ParentElement, Render, StatefulInteractiveElement, Styled, Subscription, Window,
};
use node_2::{
    CanvasPoint, CanvasSize, CrossAxisAlignment, Fill, FrameLayout, LayoutDirection,
    MainAxisAlignment, Padding, ShapeId, ShapeKind, Stroke,
};
use theme_2::Theme;

/// Properties panel showing details of selected shapes.
pub struct PropertiesPanel {
    canvas: Entity<Canvas>,
    theme: Theme,
    // Input states for position
    x_input: Entity<InputState>,
    y_input: Entity<InputState>,
    // Input states for size
    w_input: Entity<InputState>,
    h_input: Entity<InputState>,
    // Input states for fill
    fill_color_input: Entity<InputState>,
    // Input states for stroke
    stroke_width_input: Entity<InputState>,
    stroke_color_input: Entity<InputState>,
    // Input state for corner radius
    corner_radius_input: Entity<InputState>,
    // Layout inputs (for frames)
    layout_gap_input: Entity<InputState>,
    layout_padding_input: Entity<InputState>,
    // Track current selection and values to know when to update inputs
    last_selection_id: Option<ShapeId>,
    last_position: CanvasPoint,
    last_size: CanvasSize,
    last_fill: Option<Fill>,
    last_stroke: Option<Stroke>,
    last_corner_radius: f32,
    last_layout: Option<FrameLayout>,
    // Track computed vs user values for display styling
    position_is_computed: bool,
    size_is_computed: (bool, bool), // (width_computed, height_computed)
    // Store user-set values for tooltip display
    user_position: CanvasPoint,
    user_size: CanvasSize,
    _subscriptions: Vec<Subscription>,
}

impl PropertiesPanel {
    pub fn new(canvas: Entity<Canvas>, theme: Theme, cx: &mut Context<Self>) -> Self {
        let x_input = cx.new(|cx| InputState::new_singleline(cx));
        let y_input = cx.new(|cx| InputState::new_singleline(cx));
        let w_input = cx.new(|cx| InputState::new_singleline(cx));
        let h_input = cx.new(|cx| InputState::new_singleline(cx));
        let fill_color_input = cx.new(|cx| InputState::new_singleline(cx));
        let stroke_width_input = cx.new(|cx| InputState::new_singleline(cx));
        let stroke_color_input = cx.new(|cx| InputState::new_singleline(cx));
        let corner_radius_input = cx.new(|cx| InputState::new_singleline(cx));
        let layout_gap_input = cx.new(|cx| InputState::new_singleline(cx));
        let layout_padding_input = cx.new(|cx| InputState::new_singleline(cx));

        // Subscribe to input changes
        let x_sub = cx.subscribe(&x_input, Self::on_x_changed);
        let y_sub = cx.subscribe(&y_input, Self::on_y_changed);
        let w_sub = cx.subscribe(&w_input, Self::on_w_changed);
        let h_sub = cx.subscribe(&h_input, Self::on_h_changed);
        let fill_color_sub = cx.subscribe(&fill_color_input, Self::on_fill_color_changed);
        let stroke_width_sub = cx.subscribe(&stroke_width_input, Self::on_stroke_width_changed);
        let stroke_color_sub = cx.subscribe(&stroke_color_input, Self::on_stroke_color_changed);
        let corner_radius_sub =
            cx.subscribe(&corner_radius_input, Self::on_corner_radius_changed);
        let layout_gap_sub = cx.subscribe(&layout_gap_input, Self::on_layout_gap_changed);
        let layout_padding_sub = cx.subscribe(&layout_padding_input, Self::on_layout_padding_changed);

        // Subscribe to canvas changes to update inputs
        let canvas_sub = cx.subscribe(&canvas, Self::on_canvas_changed);

        Self {
            canvas,
            theme,
            x_input,
            y_input,
            w_input,
            h_input,
            fill_color_input,
            stroke_width_input,
            stroke_color_input,
            corner_radius_input,
            layout_gap_input,
            layout_padding_input,
            last_selection_id: None,
            last_position: CanvasPoint::default(),
            last_size: CanvasSize::default(),
            last_fill: None,
            last_stroke: None,
            last_corner_radius: 0.0,
            last_layout: None,
            position_is_computed: false,
            size_is_computed: (false, false),
            user_position: CanvasPoint::default(),
            user_size: CanvasSize::default(),
            _subscriptions: vec![
                x_sub,
                y_sub,
                w_sub,
                h_sub,
                fill_color_sub,
                stroke_width_sub,
                stroke_color_sub,
                corner_radius_sub,
                layout_gap_sub,
                layout_padding_sub,
                canvas_sub,
            ],
        }
    }

    fn on_canvas_changed(
        &mut self,
        _canvas: Entity<Canvas>,
        _event: &canvas_2::CanvasEvent,
        cx: &mut Context<Self>,
    ) {
        // Trigger re-render to sync inputs
        cx.notify();
    }

    fn sync_inputs_from_canvas(&mut self, window: &Window, cx: &mut Context<Self>) {
        // Extract data from canvas first to avoid borrow issues
        let shape_data = {
            let canvas = self.canvas.read(cx);
            canvas
                .shapes
                .iter()
                .find(|s| canvas.selection.contains(&s.id))
                .map(|shape| {
                    (
                        shape.id,
                        shape.effective_position(), // Use computed position if available
                        shape.effective_size(),     // Use computed size if available
                        shape.fill.clone(),
                        shape.stroke.clone(),
                        shape.corner_radius,
                        shape.layout.clone(),
                        // Track computed state
                        shape.has_computed_position(),
                        shape.computed_size.is_some(),
                        // Store user-set values
                        shape.position,
                        shape.size,
                    )
                })
        };

        if let Some((shape_id, position, size, fill, stroke, corner_radius, layout, pos_computed, size_computed, user_pos, user_sz)) = shape_data {
            // Update computed state tracking
            self.position_is_computed = pos_computed;
            self.size_is_computed = (size_computed, size_computed);
            self.user_position = user_pos;
            self.user_size = user_sz;
            let selection_changed = self.last_selection_id != Some(shape_id);
            let position_changed = self.last_position != position;
            let size_changed = self.last_size != size;
            let fill_changed = self.last_fill != fill;
            let stroke_changed = self.last_stroke != stroke;
            let corner_radius_changed = self.last_corner_radius != corner_radius;
            let layout_changed = self.last_layout != layout;

            // Update tracking
            self.last_selection_id = Some(shape_id);
            self.last_position = position;
            self.last_size = size;
            self.last_fill = fill.clone();
            self.last_stroke = stroke.clone();
            self.last_corner_radius = corner_radius;
            self.last_layout = layout.clone();

            // Update inputs if values changed, but only if not focused (avoid fighting with user)
            if selection_changed || position_changed {
                if !self.x_input.focus_handle(cx).is_focused(window) {
                    self.x_input.update(cx, |input, cx| {
                        input.set_content(format!("{:.0}", position.x()), cx);
                    });
                }
                if !self.y_input.focus_handle(cx).is_focused(window) {
                    self.y_input.update(cx, |input, cx| {
                        input.set_content(format!("{:.0}", position.y()), cx);
                    });
                }
            }

            if selection_changed || size_changed {
                if !self.w_input.focus_handle(cx).is_focused(window) {
                    self.w_input.update(cx, |input, cx| {
                        input.set_content(format!("{:.0}", size.width()), cx);
                    });
                }
                if !self.h_input.focus_handle(cx).is_focused(window) {
                    self.h_input.update(cx, |input, cx| {
                        input.set_content(format!("{:.0}", size.height()), cx);
                    });
                }
            }

            if selection_changed || fill_changed {
                if !self.fill_color_input.focus_handle(cx).is_focused(window) {
                    let content = fill
                        .as_ref()
                        .map(|f| hsla_to_hex(f.color))
                        .unwrap_or_default();
                    self.fill_color_input.update(cx, |input, cx| {
                        input.set_content(content, cx);
                    });
                }
            }

            if selection_changed || stroke_changed {
                if let Some(s) = &stroke {
                    if !self.stroke_width_input.focus_handle(cx).is_focused(window) {
                        self.stroke_width_input.update(cx, |input, cx| {
                            input.set_content(format!("{:.0}", s.width), cx);
                        });
                    }
                    if !self.stroke_color_input.focus_handle(cx).is_focused(window) {
                        self.stroke_color_input.update(cx, |input, cx| {
                            input.set_content(hsla_to_hex(s.color), cx);
                        });
                    }
                }
            }

            if selection_changed || corner_radius_changed {
                if !self.corner_radius_input.focus_handle(cx).is_focused(window) {
                    self.corner_radius_input.update(cx, |input, cx| {
                        input.set_content(format!("{:.0}", corner_radius), cx);
                    });
                }
            }

            // Sync layout inputs
            if selection_changed || layout_changed {
                if let Some(ref l) = layout {
                    if !self.layout_gap_input.focus_handle(cx).is_focused(window) {
                        self.layout_gap_input.update(cx, |input, cx| {
                            input.set_content(format!("{:.0}", l.gap), cx);
                        });
                    }
                    if !self.layout_padding_input.focus_handle(cx).is_focused(window) {
                        // Show uniform padding if all sides are equal, otherwise show "mixed"
                        let p = &l.padding;
                        if p.top == p.right && p.right == p.bottom && p.bottom == p.left {
                            self.layout_padding_input.update(cx, |input, cx| {
                                input.set_content(format!("{:.0}", p.top), cx);
                            });
                        } else {
                            self.layout_padding_input.update(cx, |input, cx| {
                                input.set_content("mixed".to_string(), cx);
                            });
                        }
                    }
                } else {
                    // No layout - clear inputs
                    if !self.layout_gap_input.focus_handle(cx).is_focused(window) {
                        self.layout_gap_input.update(cx, |input, cx| {
                            input.set_content("0".to_string(), cx);
                        });
                    }
                    if !self.layout_padding_input.focus_handle(cx).is_focused(window) {
                        self.layout_padding_input.update(cx, |input, cx| {
                            input.set_content("0".to_string(), cx);
                        });
                    }
                }
            }
        } else {
            self.last_selection_id = None;
            self.last_position = CanvasPoint::default();
            self.last_size = CanvasSize::default();
            self.last_fill = None;
            self.last_stroke = None;
            self.last_corner_radius = 0.0;
            self.last_layout = None;
            self.position_is_computed = false;
            self.size_is_computed = (false, false);
            self.user_position = CanvasPoint::default();
            self.user_size = CanvasSize::default();
        }
    }

    fn on_x_changed(
        &mut self,
        _input: Entity<InputState>,
        event: &InputStateEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputStateEvent::TextChanged) {
            self.apply_position_x(cx);
        }
    }

    fn on_y_changed(
        &mut self,
        _input: Entity<InputState>,
        event: &InputStateEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputStateEvent::TextChanged) {
            self.apply_position_y(cx);
        }
    }

    fn on_w_changed(
        &mut self,
        _input: Entity<InputState>,
        event: &InputStateEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputStateEvent::TextChanged) {
            self.apply_size_w(cx);
        }
    }

    fn on_h_changed(
        &mut self,
        _input: Entity<InputState>,
        event: &InputStateEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputStateEvent::TextChanged) {
            self.apply_size_h(cx);
        }
    }

    fn on_fill_color_changed(
        &mut self,
        _input: Entity<InputState>,
        event: &InputStateEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputStateEvent::TextChanged) {
            self.apply_fill_color(cx);
        }
    }

    fn apply_position_x(&mut self, cx: &mut Context<Self>) {
        let value = self.x_input.read(cx).content().to_string();
        if let Ok(x) = value.parse::<f32>() {
            let parent_frame_id = self.canvas.update(cx, |canvas, cx| {
                let shape_id = canvas.selection.iter().next().copied();
                if let Some(shape) = shape_id.and_then(|id| {
                    canvas.shapes.iter_mut().find(|s| s.id == id)
                }) {
                    shape.position.0.x = x;
                    cx.emit(CanvasEvent::ContentChanged);
                    cx.notify();
                    // Check if shape is in a layout frame
                    if let Some(parent_id) = shape.parent {
                        let parent_has_layout = canvas.shapes.iter()
                            .find(|s| s.id == parent_id)
                            .map(|p| p.layout.is_some())
                            .unwrap_or(false);
                        if parent_has_layout {
                            return Some(parent_id);
                        }
                    }
                }
                None
            });
            // Reapply parent layout if shape is in autolayout
            if let Some(parent_id) = parent_frame_id {
                self.canvas.update(cx, |canvas, cx| {
                    canvas.apply_layout_for_frame(parent_id);
                    cx.emit(CanvasEvent::ContentChanged);
                    cx.notify();
                });
            }
        }
    }

    fn apply_position_y(&mut self, cx: &mut Context<Self>) {
        let value = self.y_input.read(cx).content().to_string();
        if let Ok(y) = value.parse::<f32>() {
            let parent_frame_id = self.canvas.update(cx, |canvas, cx| {
                let shape_id = canvas.selection.iter().next().copied();
                if let Some(shape) = shape_id.and_then(|id| {
                    canvas.shapes.iter_mut().find(|s| s.id == id)
                }) {
                    shape.position.0.y = y;
                    cx.emit(CanvasEvent::ContentChanged);
                    cx.notify();
                    // Check if shape is in a layout frame
                    if let Some(parent_id) = shape.parent {
                        let parent_has_layout = canvas.shapes.iter()
                            .find(|s| s.id == parent_id)
                            .map(|p| p.layout.is_some())
                            .unwrap_or(false);
                        if parent_has_layout {
                            return Some(parent_id);
                        }
                    }
                }
                None
            });
            // Reapply parent layout if shape is in autolayout
            if let Some(parent_id) = parent_frame_id {
                self.canvas.update(cx, |canvas, cx| {
                    canvas.apply_layout_for_frame(parent_id);
                    cx.emit(CanvasEvent::ContentChanged);
                    cx.notify();
                });
            }
        }
    }

    fn apply_size_w(&mut self, cx: &mut Context<Self>) {
        let value = self.w_input.read(cx).content().to_string();
        if let Ok(w) = value.parse::<f32>() {
            if w > 0.0 {
                let parent_frame_id = self.canvas.update(cx, |canvas, cx| {
                    let shape_id = canvas.selection.iter().next().copied();
                    if let Some(shape) = shape_id.and_then(|id| {
                        canvas.shapes.iter_mut().find(|s| s.id == id)
                    }) {
                        shape.size.0.x = w;
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                        // Check if shape is in a layout frame
                        if let Some(parent_id) = shape.parent {
                            let parent_has_layout = canvas.shapes.iter()
                                .find(|s| s.id == parent_id)
                                .map(|p| p.layout.is_some())
                                .unwrap_or(false);
                            if parent_has_layout {
                                return Some(parent_id);
                            }
                        }
                    }
                    None
                });
                // Reapply parent layout if shape is in autolayout
                if let Some(parent_id) = parent_frame_id {
                    self.canvas.update(cx, |canvas, cx| {
                        canvas.apply_layout_for_frame(parent_id);
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                    });
                }
            }
        }
    }

    fn apply_size_h(&mut self, cx: &mut Context<Self>) {
        let value = self.h_input.read(cx).content().to_string();
        if let Ok(h) = value.parse::<f32>() {
            if h > 0.0 {
                let parent_frame_id = self.canvas.update(cx, |canvas, cx| {
                    let shape_id = canvas.selection.iter().next().copied();
                    if let Some(shape) = shape_id.and_then(|id| {
                        canvas.shapes.iter_mut().find(|s| s.id == id)
                    }) {
                        shape.size.0.y = h;
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                        // Check if shape is in a layout frame
                        if let Some(parent_id) = shape.parent {
                            let parent_has_layout = canvas.shapes.iter()
                                .find(|s| s.id == parent_id)
                                .map(|p| p.layout.is_some())
                                .unwrap_or(false);
                            if parent_has_layout {
                                return Some(parent_id);
                            }
                        }
                    }
                    None
                });
                // Reapply parent layout if shape is in autolayout
                if let Some(parent_id) = parent_frame_id {
                    self.canvas.update(cx, |canvas, cx| {
                        canvas.apply_layout_for_frame(parent_id);
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                    });
                }
            }
        }
    }

    fn apply_fill_color(&mut self, cx: &mut Context<Self>) {
        let value = self.fill_color_input.read(cx).content().to_string();
        if value.is_empty() {
            // Empty value removes fill
            self.canvas.update(cx, |canvas, cx| {
                if let Some(shape) = canvas
                    .shapes
                    .iter_mut()
                    .find(|s| canvas.selection.contains(&s.id))
                {
                    shape.fill = None;
                    cx.emit(CanvasEvent::ContentChanged);
                    cx.notify();
                }
            });
        } else if let Some(color) = hex_to_hsla(&value) {
            self.canvas.update(cx, |canvas, cx| {
                if let Some(shape) = canvas
                    .shapes
                    .iter_mut()
                    .find(|s| canvas.selection.contains(&s.id))
                {
                    shape.fill = Some(Fill::new(color));
                    cx.emit(CanvasEvent::ContentChanged);
                    cx.notify();
                }
            });
        }
    }

    fn on_stroke_width_changed(
        &mut self,
        _input: Entity<InputState>,
        event: &InputStateEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputStateEvent::TextChanged) {
            self.apply_stroke_width(cx);
        }
    }

    fn on_stroke_color_changed(
        &mut self,
        _input: Entity<InputState>,
        event: &InputStateEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputStateEvent::TextChanged) {
            self.apply_stroke_color(cx);
        }
    }

    fn apply_stroke_width(&mut self, cx: &mut Context<Self>) {
        let value = self.stroke_width_input.read(cx).content().to_string();
        let default_color = self.theme.default_stroke;
        if let Ok(width) = value.parse::<f32>() {
            if width >= 0.0 {
                self.canvas.update(cx, |canvas, cx| {
                    if let Some(shape) = canvas
                        .shapes
                        .iter_mut()
                        .find(|s| canvas.selection.contains(&s.id))
                    {
                        if let Some(stroke) = &mut shape.stroke {
                            stroke.width = width;
                        } else {
                            // Create stroke with default color if none exists
                            shape.stroke = Some(Stroke::new(default_color, width));
                        }
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                    }
                });
            }
        }
    }

    fn apply_stroke_color(&mut self, cx: &mut Context<Self>) {
        let value = self.stroke_color_input.read(cx).content().to_string();
        if let Some(color) = hex_to_hsla(&value) {
            self.canvas.update(cx, |canvas, cx| {
                if let Some(shape) = canvas
                    .shapes
                    .iter_mut()
                    .find(|s| canvas.selection.contains(&s.id))
                {
                    if let Some(stroke) = &mut shape.stroke {
                        stroke.color = color;
                    } else {
                        // Create stroke with default width if none exists
                        shape.stroke = Some(Stroke::new(color, 2.0));
                    }
                    cx.emit(CanvasEvent::ContentChanged);
                    cx.notify();
                }
            });
        }
    }

    fn on_corner_radius_changed(
        &mut self,
        _input: Entity<InputState>,
        event: &InputStateEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputStateEvent::TextChanged) {
            self.apply_corner_radius(cx);
        }
    }

    fn on_layout_gap_changed(
        &mut self,
        _input: Entity<InputState>,
        event: &InputStateEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputStateEvent::TextChanged) {
            self.apply_layout_gap(cx);
        }
    }

    fn on_layout_padding_changed(
        &mut self,
        _input: Entity<InputState>,
        event: &InputStateEvent,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputStateEvent::TextChanged) {
            self.apply_layout_padding(cx);
        }
    }

    fn apply_corner_radius(&mut self, cx: &mut Context<Self>) {
        let value = self.corner_radius_input.read(cx).content().to_string();
        if let Ok(radius) = value.parse::<f32>() {
            if radius >= 0.0 {
                self.canvas.update(cx, |canvas, cx| {
                    if let Some(shape) = canvas
                        .shapes
                        .iter_mut()
                        .find(|s| canvas.selection.contains(&s.id))
                    {
                        // Clamp corner radius to half the smaller dimension to prevent overlap
                        let max_radius = shape.size.width().min(shape.size.height()) / 2.0;
                        shape.corner_radius = radius.min(max_radius);
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                    }
                });
            }
        }
    }

    fn apply_layout_gap(&mut self, cx: &mut Context<Self>) {
        let value = self.layout_gap_input.read(cx).content().to_string();
        if let Ok(gap) = value.parse::<f32>() {
            if gap >= 0.0 {
                let frame_id = self.canvas.update(cx, |canvas, cx| {
                    let shape_id = canvas.selection.iter().next().copied();
                    if let Some(shape) = shape_id.and_then(|id| {
                        canvas.shapes.iter_mut().find(|s| s.id == id)
                    }) {
                        if let Some(ref mut layout) = shape.layout {
                            layout.gap = gap;
                        } else {
                            shape.layout = Some(FrameLayout {
                                gap,
                                ..Default::default()
                            });
                        }
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                        return Some(shape.id);
                    }
                    None
                });
                if let Some(frame_id) = frame_id {
                    self.canvas.update(cx, |canvas, cx| {
                        canvas.apply_layout_for_frame(frame_id);
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                    });
                }
            }
        }
    }

    fn apply_layout_padding(&mut self, cx: &mut Context<Self>) {
        let value = self.layout_padding_input.read(cx).content().to_string();
        if value == "mixed" {
            return;
        }
        if let Ok(padding) = value.parse::<f32>() {
            if padding >= 0.0 {
                let frame_id = self.canvas.update(cx, |canvas, cx| {
                    let shape_id = canvas.selection.iter().next().copied();
                    if let Some(shape) = shape_id.and_then(|id| {
                        canvas.shapes.iter_mut().find(|s| s.id == id)
                    }) {
                        if let Some(ref mut layout) = shape.layout {
                            layout.padding = Padding::all(padding);
                        } else {
                            shape.layout = Some(FrameLayout {
                                padding: Padding::all(padding),
                                ..Default::default()
                            });
                        }
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                        return Some(shape.id);
                    }
                    None
                });
                if let Some(frame_id) = frame_id {
                    self.canvas.update(cx, |canvas, cx| {
                        canvas.apply_layout_for_frame(frame_id);
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                    });
                }
            }
        }
    }

    /// Toggle autolayout on/off for selected frame
    pub fn toggle_autolayout(&mut self, cx: &mut Context<Self>) {
        // Returns (should_apply_layout, frame_id_to_clear)
        let (apply_frame_id, clear_frame_id) = self.canvas.update(cx, |canvas, cx| {
            let shape_id = canvas.selection.iter().next().copied();
            if let Some(shape) = shape_id.and_then(|id| {
                canvas.shapes.iter_mut().find(|s| s.id == id)
            }) {
                if shape.kind == ShapeKind::Frame {
                    let frame_id = shape.id;
                    if shape.layout.is_some() {
                        // Turning off layout - need to clear computed values
                        shape.layout = None;
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                        return (None, Some(frame_id));
                    } else {
                        // Turning on layout
                        shape.layout = Some(FrameLayout::default());
                        cx.emit(CanvasEvent::ContentChanged);
                        cx.notify();
                        return (Some(frame_id), None);
                    }
                }
            }
            (None, None)
        });

        // Clear computed values if layout was disabled
        if let Some(frame_id) = clear_frame_id {
            self.canvas.update(cx, |canvas, cx| {
                canvas.clear_layout_for_frame(frame_id);
                cx.emit(CanvasEvent::ContentChanged);
                cx.notify();
            });
        }

        // Apply layout if it was enabled
        if let Some(frame_id) = apply_frame_id {
            self.canvas.update(cx, |canvas, cx| {
                canvas.apply_layout_for_frame(frame_id);
                cx.emit(CanvasEvent::ContentChanged);
                cx.notify();
            });
        }
    }

    /// Set layout direction for selected frame
    pub fn set_layout_direction(&mut self, direction: LayoutDirection, cx: &mut Context<Self>) {
        let frame_id = self.canvas.update(cx, |canvas, cx| {
            let shape_id = canvas.selection.iter().next().copied();
            if let Some(shape) = shape_id.and_then(|id| {
                canvas.shapes.iter_mut().find(|s| s.id == id)
            }) {
                if let Some(ref mut layout) = shape.layout {
                    layout.direction = direction;
                    cx.emit(CanvasEvent::ContentChanged);
                    cx.notify();
                    return Some(shape.id);
                }
            }
            None
        });
        if let Some(frame_id) = frame_id {
            self.canvas.update(cx, |canvas, cx| {
                canvas.apply_layout_for_frame(frame_id);
                cx.emit(CanvasEvent::ContentChanged);
                cx.notify();
            });
        }
    }

    /// Set main axis alignment for selected frame
    pub fn set_main_axis_alignment(&mut self, alignment: MainAxisAlignment, cx: &mut Context<Self>) {
        let frame_id = self.canvas.update(cx, |canvas, cx| {
            let shape_id = canvas.selection.iter().next().copied();
            if let Some(shape) = shape_id.and_then(|id| {
                canvas.shapes.iter_mut().find(|s| s.id == id)
            }) {
                if let Some(ref mut layout) = shape.layout {
                    layout.main_axis_alignment = alignment;
                    cx.emit(CanvasEvent::ContentChanged);
                    cx.notify();
                    return Some(shape.id);
                }
            }
            None
        });
        if let Some(frame_id) = frame_id {
            self.canvas.update(cx, |canvas, cx| {
                canvas.apply_layout_for_frame(frame_id);
                cx.emit(CanvasEvent::ContentChanged);
                cx.notify();
            });
        }
    }

    /// Set cross axis alignment for selected frame
    pub fn set_cross_axis_alignment(&mut self, alignment: CrossAxisAlignment, cx: &mut Context<Self>) {
        let frame_id = self.canvas.update(cx, |canvas, cx| {
            let shape_id = canvas.selection.iter().next().copied();
            if let Some(shape) = shape_id.and_then(|id| {
                canvas.shapes.iter_mut().find(|s| s.id == id)
            }) {
                if let Some(ref mut layout) = shape.layout {
                    layout.cross_axis_alignment = alignment;
                    cx.emit(CanvasEvent::ContentChanged);
                    cx.notify();
                    return Some(shape.id);
                }
            }
            None
        });
        if let Some(frame_id) = frame_id {
            self.canvas.update(cx, |canvas, cx| {
                canvas.apply_layout_for_frame(frame_id);
                cx.emit(CanvasEvent::ContentChanged);
                cx.notify();
            });
        }
    }

    fn input_colors(&self) -> InputColors {
        InputColors {
            selection: self.theme.selection.opacity(0.3),
            cursor: self.theme.selection,
            placeholder: self.theme.ui_text_muted,
        }
    }
}

impl Render for PropertiesPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Sync inputs if needed
        self.sync_inputs_from_canvas(window, cx);

        let canvas = self.canvas.read(cx);
        let theme = &self.theme;

        // Get selected shapes
        let selected: Vec<_> = canvas
            .shapes
            .iter()
            .filter(|s| canvas.selection.contains(&s.id))
            .collect();

        let colors = self.input_colors();

        let content = if selected.is_empty() {
            div()
                .text_sm()
                .text_color(theme.ui_text_muted)
                .child("No selection")
        } else if selected.len() == 1 {
            let shape = selected[0];
            let kind_name = match shape.kind {
                ShapeKind::Rectangle => "Rectangle",
                ShapeKind::Ellipse => "Ellipse",
                ShapeKind::Frame => "Frame",
            };

            v_stack()
                .gap(px(12.0))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.ui_text)
                        .child(kind_name),
                )
                // Position (shows computed values in muted color if in autolayout)
                .child(
                    v_stack()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.ui_text_muted)
                                .child(if self.position_is_computed { "Position (auto)" } else { "Position" }),
                        )
                        .child(
                            h_stack()
                                .gap(px(8.0))
                                .child(input_field_with_computed(
                                    "X",
                                    &self.x_input,
                                    theme,
                                    &colors,
                                    self.position_is_computed,
                                    if self.position_is_computed { Some(format!("{:.0}", self.user_position.x())) } else { None },
                                    cx,
                                ))
                                .child(input_field_with_computed(
                                    "Y",
                                    &self.y_input,
                                    theme,
                                    &colors,
                                    self.position_is_computed,
                                    if self.position_is_computed { Some(format!("{:.0}", self.user_position.y())) } else { None },
                                    cx,
                                )),
                        ),
                )
                // Size (shows computed values in muted color if stretched)
                .child(
                    v_stack()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.ui_text_muted)
                                .child(if self.size_is_computed.0 || self.size_is_computed.1 { "Size (auto)" } else { "Size" }),
                        )
                        .child(
                            h_stack()
                                .gap(px(8.0))
                                .child(input_field_with_computed(
                                    "W",
                                    &self.w_input,
                                    theme,
                                    &colors,
                                    self.size_is_computed.0,
                                    if self.size_is_computed.0 { Some(format!("{:.0}", self.user_size.width())) } else { None },
                                    cx,
                                ))
                                .child(input_field_with_computed(
                                    "H",
                                    &self.h_input,
                                    theme,
                                    &colors,
                                    self.size_is_computed.1,
                                    if self.size_is_computed.1 { Some(format!("{:.0}", self.user_size.height())) } else { None },
                                    cx,
                                )),
                        ),
                )
                // Corner Radius (only for rectangles)
                .children(if shape.kind == ShapeKind::Rectangle {
                    Some(
                        v_stack()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.ui_text_muted)
                                    .child("Corner Radius"),
                            )
                            .child(input_field(
                                "",
                                &self.corner_radius_input,
                                theme,
                                &colors,
                                cx,
                            )),
                    )
                } else {
                    None
                })
                // Fill
                .child(
                    v_stack()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.ui_text_muted)
                                .child("Fill"),
                        )
                        .child(
                            h_stack()
                                .gap(px(8.0))
                                .items_center()
                                .child(color_swatch(
                                    shape.fill.as_ref().map(|f| f.color),
                                    theme,
                                ))
                                .child(input_field(
                                    "",
                                    &self.fill_color_input,
                                    theme,
                                    &colors,
                                    cx,
                                )),
                        ),
                )
                // Stroke
                .child(
                    v_stack()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.ui_text_muted)
                                .child("Stroke"),
                        )
                        .child(
                            h_stack()
                                .gap(px(8.0))
                                .items_center()
                                .child(color_swatch(
                                    shape.stroke.as_ref().map(|s| s.color),
                                    theme,
                                ))
                                .child(input_field(
                                    "",
                                    &self.stroke_color_input,
                                    theme,
                                    &colors,
                                    cx,
                                ))
                                .child(input_field(
                                    "W",
                                    &self.stroke_width_input,
                                    theme,
                                    &colors,
                                    cx,
                                )),
                        ),
                )
                // Autolayout (only for frames)
                .children(if shape.kind == ShapeKind::Frame {
                    let has_layout = shape.layout.is_some();
                    let layout = shape.layout.clone().unwrap_or_default();
                    let this = cx.entity().clone();

                    Some(
                        v_stack()
                            .gap(px(8.0))
                            .child(
                                h_stack()
                                    .gap(px(8.0))
                                    .items_center()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.ui_text_muted)
                                            .child("Autolayout"),
                                    )
                                    .child({
                                        let this = this.clone();
                                        div()
                                            .id("autolayout-toggle")
                                            .px(px(8.0))
                                            .py(px(2.0))
                                            .rounded(px(2.0))
                                            .text_xs()
                                            .cursor_pointer()
                                            .bg(if has_layout {
                                                theme.selection.opacity(0.3)
                                            } else {
                                                theme.ui_background
                                            })
                                            .border_1()
                                            .border_color(if has_layout {
                                                theme.selection
                                            } else {
                                                theme.ui_border
                                            })
                                            .text_color(theme.ui_text)
                                            .child(if has_layout { "On" } else { "Off" })
                                            .on_click(move |_, _, cx| {
                                                this.update(cx, |panel, cx| {
                                                    panel.toggle_autolayout(cx);
                                                });
                                            })
                                    }),
                            )
                            .children(if has_layout {
                                let theme_clone = theme.clone();
                                Some(
                                    v_stack()
                                        .gap(px(8.0))
                                        // Direction buttons
                                        .child(
                                            h_stack()
                                                .gap(px(4.0))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme_clone.ui_text_muted)
                                                        .w(px(50.0))
                                                        .child("Direction"),
                                                )
                                                .child(
                                                    h_stack()
                                                        .gap(px(4.0))
                                                        .child({
                                                            let this = this.clone();
                                                            clickable_toggle(
                                                                "Row",
                                                                layout.direction == LayoutDirection::Row,
                                                                &theme_clone,
                                                                "dir-row",
                                                                move |_, cx| {
                                                                    this.update(cx, |panel, cx| {
                                                                        panel.set_layout_direction(LayoutDirection::Row, cx);
                                                                    });
                                                                },
                                                            )
                                                        })
                                                        .child({
                                                            let this = this.clone();
                                                            clickable_toggle(
                                                                "Col",
                                                                layout.direction == LayoutDirection::Column,
                                                                &theme_clone,
                                                                "dir-col",
                                                                move |_, cx| {
                                                                    this.update(cx, |panel, cx| {
                                                                        panel.set_layout_direction(LayoutDirection::Column, cx);
                                                                    });
                                                                },
                                                            )
                                                        }),
                                                ),
                                        )
                                        // Gap input
                                        .child(
                                            h_stack()
                                                .gap(px(4.0))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme_clone.ui_text_muted)
                                                        .w(px(50.0))
                                                        .child("Gap"),
                                                )
                                                .child(input_field("", &self.layout_gap_input, &theme_clone, &colors, cx)),
                                        )
                                        // Padding input
                                        .child(
                                            h_stack()
                                                .gap(px(4.0))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme_clone.ui_text_muted)
                                                        .w(px(50.0))
                                                        .child("Padding"),
                                                )
                                                .child(input_field("", &self.layout_padding_input, &theme_clone, &colors, cx)),
                                        )
                                        // Main axis alignment
                                        .child(
                                            v_stack()
                                                .gap(px(4.0))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme_clone.ui_text_muted)
                                                        .child("Main Axis"),
                                                )
                                                .child(
                                                    h_stack()
                                                        .gap(px(2.0))
                                                        .child({
                                                            let this = this.clone();
                                                            clickable_toggle(
                                                                "Start",
                                                                layout.main_axis_alignment == MainAxisAlignment::Start,
                                                                &theme_clone,
                                                                "main-start",
                                                                move |_, cx| {
                                                                    this.update(cx, |panel, cx| {
                                                                        panel.set_main_axis_alignment(MainAxisAlignment::Start, cx);
                                                                    });
                                                                },
                                                            )
                                                        })
                                                        .child({
                                                            let this = this.clone();
                                                            clickable_toggle(
                                                                "Center",
                                                                layout.main_axis_alignment == MainAxisAlignment::Center,
                                                                &theme_clone,
                                                                "main-center",
                                                                move |_, cx| {
                                                                    this.update(cx, |panel, cx| {
                                                                        panel.set_main_axis_alignment(MainAxisAlignment::Center, cx);
                                                                    });
                                                                },
                                                            )
                                                        })
                                                        .child({
                                                            let this = this.clone();
                                                            clickable_toggle(
                                                                "End",
                                                                layout.main_axis_alignment == MainAxisAlignment::End,
                                                                &theme_clone,
                                                                "main-end",
                                                                move |_, cx| {
                                                                    this.update(cx, |panel, cx| {
                                                                        panel.set_main_axis_alignment(MainAxisAlignment::End, cx);
                                                                    });
                                                                },
                                                            )
                                                        })
                                                        .child({
                                                            let this = this.clone();
                                                            clickable_toggle(
                                                                "Space",
                                                                layout.main_axis_alignment == MainAxisAlignment::SpaceBetween,
                                                                &theme_clone,
                                                                "main-space",
                                                                move |_, cx| {
                                                                    this.update(cx, |panel, cx| {
                                                                        panel.set_main_axis_alignment(MainAxisAlignment::SpaceBetween, cx);
                                                                    });
                                                                },
                                                            )
                                                        }),
                                                ),
                                        )
                                        // Cross axis alignment
                                        .child(
                                            v_stack()
                                                .gap(px(4.0))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme_clone.ui_text_muted)
                                                        .child("Cross Axis"),
                                                )
                                                .child(
                                                    h_stack()
                                                        .gap(px(2.0))
                                                        .child({
                                                            let this = this.clone();
                                                            clickable_toggle(
                                                                "Start",
                                                                layout.cross_axis_alignment == CrossAxisAlignment::Start,
                                                                &theme_clone,
                                                                "cross-start",
                                                                move |_, cx| {
                                                                    this.update(cx, |panel, cx| {
                                                                        panel.set_cross_axis_alignment(CrossAxisAlignment::Start, cx);
                                                                    });
                                                                },
                                                            )
                                                        })
                                                        .child({
                                                            let this = this.clone();
                                                            clickable_toggle(
                                                                "Center",
                                                                layout.cross_axis_alignment == CrossAxisAlignment::Center,
                                                                &theme_clone,
                                                                "cross-center",
                                                                move |_, cx| {
                                                                    this.update(cx, |panel, cx| {
                                                                        panel.set_cross_axis_alignment(CrossAxisAlignment::Center, cx);
                                                                    });
                                                                },
                                                            )
                                                        })
                                                        .child({
                                                            let this = this.clone();
                                                            clickable_toggle(
                                                                "End",
                                                                layout.cross_axis_alignment == CrossAxisAlignment::End,
                                                                &theme_clone,
                                                                "cross-end",
                                                                move |_, cx| {
                                                                    this.update(cx, |panel, cx| {
                                                                        panel.set_cross_axis_alignment(CrossAxisAlignment::End, cx);
                                                                    });
                                                                },
                                                            )
                                                        })
                                                        .child({
                                                            let this = this.clone();
                                                            clickable_toggle(
                                                                "Stretch",
                                                                layout.cross_axis_alignment == CrossAxisAlignment::Stretch,
                                                                &theme_clone,
                                                                "cross-stretch",
                                                                move |_, cx| {
                                                                    this.update(cx, |panel, cx| {
                                                                        panel.set_cross_axis_alignment(CrossAxisAlignment::Stretch, cx);
                                                                    });
                                                                },
                                                            )
                                                        }),
                                                ),
                                        ),
                                )
                            } else {
                                None
                            }),
                    )
                } else {
                    None
                })
        } else {
            div()
                .text_sm()
                .text_color(theme.ui_text_muted)
                .child(format!("{} shapes selected", selected.len()))
        };

        panel(theme)
            .w(px(200.0))
            .h_full()
            .child(
                div()
                    .text_xs()
                    .text_color(theme.ui_text_muted)
                    .pb(px(8.0))
                    .child("Properties"),
            )
            .child(content)
    }
}

fn input_field(
    label: &str,
    input_state: &Entity<InputState>,
    theme: &Theme,
    colors: &InputColors,
    cx: &gpui::App,
) -> impl IntoElement {
    input_field_with_computed(label, input_state, theme, colors, false, None, cx)
}

/// Input field that can show computed values with special styling.
/// When `is_computed` is true, the text is shown in muted color.
/// If `user_value` is provided, it's shown as a tooltip on hover.
fn input_field_with_computed(
    label: &str,
    input_state: &Entity<InputState>,
    theme: &Theme,
    colors: &InputColors,
    is_computed: bool,
    user_value: Option<String>,
    cx: &gpui::App,
) -> impl IntoElement {
    // Text color: muted (60% opacity) if computed, normal otherwise
    let text_color = if is_computed {
        theme.ui_text.opacity(0.6)
    } else {
        theme.ui_text
    };

    let mut field = h_stack()
        .flex_1()
        .gap(px(4.0))
        .child(
            div()
                .text_xs()
                .text_color(theme.ui_text_muted)
                .w(px(14.0))
                .child(label.to_string()),
        )
        .child(
            input(input_state, cx)
                .colors(colors.clone())
                .flex_1()
                .h(px(22.0))
                .px(px(6.0))
                .bg(theme_2::hsla(0.0, 0.0, 0.95, 1.0))
                .border_1()
                .border_color(if is_computed {
                    theme.ui_border.opacity(0.5)
                } else {
                    theme.ui_border
                })
                .rounded(px(2.0))
                .text_xs()
                .text_color(text_color),
        );

    // Add tooltip showing user-set value if this is a computed value
    if let Some(user_val) = user_value {
        field = field.child(
            div()
                .absolute()
                .right_0()
                .top_0()
                .text_xs()
                .text_color(theme.ui_text_muted)
                .opacity(0.0)
                .hover(|s| s.opacity(1.0))
                .child(format!("({})", user_val)),
        );
    }

    field
}

fn color_swatch(color: Option<Hsla>, theme: &Theme) -> impl IntoElement {
    div()
        .size(px(20.0))
        .rounded(px(2.0))
        .border_1()
        .border_color(theme.ui_border)
        .bg(color.unwrap_or(gpui::hsla(0.0, 0.0, 0.9, 1.0)))
}

fn clickable_toggle<F>(
    label: &str,
    selected: bool,
    theme: &Theme,
    id: impl Into<gpui::SharedString>,
    on_click: F,
) -> gpui::Stateful<gpui::Div>
where
    F: Fn(&gpui::ClickEvent, &mut gpui::App) + 'static,
{
    div()
        .id(gpui::ElementId::Name(id.into()))
        .px(px(6.0))
        .py(px(2.0))
        .rounded(px(2.0))
        .text_xs()
        .cursor_pointer()
        .bg(if selected {
            theme.selection.opacity(0.3)
        } else {
            theme.ui_background
        })
        .border_1()
        .border_color(if selected {
            theme.selection
        } else {
            theme.ui_border
        })
        .text_color(theme.ui_text)
        .child(label.to_string())
        .on_click(move |event, _, cx| on_click(event, cx))
}

/// Convert HSLA color to hex string (e.g., "#FF0000")
fn hsla_to_hex(c: Hsla) -> String {
    // Convert HSL to RGB
    let (r, g, b) = hsl_to_rgb(c.h, c.s, c.l);
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// Convert hex string to HSLA color
fn hex_to_hsla(hex: &str) -> Option<Hsla> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    let (h, s, l) = rgb_to_hsl(r, g, b);
    Some(gpui::hsla(h, s, l, 1.0))
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l * 255.0) as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if max == min {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };

    (h, s, l)
}
