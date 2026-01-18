//! Properties panel for selected shapes.

use crate::components::{h_stack, panel, v_stack};
use crate::input::{input, InputColors, InputState, InputStateEvent};
use canvas_2::Canvas;
use glam::Vec2;
use gpui::{
    div, px, AppContext, Context, Entity, Focusable, IntoElement, ParentElement, Render, Styled,
    Subscription, Window,
};
use node_2::{ShapeId, ShapeKind};
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
    // Track current selection and values to know when to update inputs
    last_selection_id: Option<ShapeId>,
    last_position: Vec2,
    last_size: Vec2,
    _subscriptions: Vec<Subscription>,
}

impl PropertiesPanel {
    pub fn new(canvas: Entity<Canvas>, theme: Theme, cx: &mut Context<Self>) -> Self {
        let x_input = cx.new(|cx| InputState::new_singleline(cx));
        let y_input = cx.new(|cx| InputState::new_singleline(cx));
        let w_input = cx.new(|cx| InputState::new_singleline(cx));
        let h_input = cx.new(|cx| InputState::new_singleline(cx));

        // Subscribe to input changes
        let x_sub = cx.subscribe(&x_input, Self::on_x_changed);
        let y_sub = cx.subscribe(&y_input, Self::on_y_changed);
        let w_sub = cx.subscribe(&w_input, Self::on_w_changed);
        let h_sub = cx.subscribe(&h_input, Self::on_h_changed);

        // Subscribe to canvas changes to update inputs
        let canvas_sub = cx.subscribe(&canvas, Self::on_canvas_changed);

        Self {
            canvas,
            theme,
            x_input,
            y_input,
            w_input,
            h_input,
            last_selection_id: None,
            last_position: Vec2::ZERO,
            last_size: Vec2::ZERO,
            _subscriptions: vec![x_sub, y_sub, w_sub, h_sub, canvas_sub],
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
                .map(|shape| (shape.id, shape.position, shape.size))
        };

        if let Some((shape_id, position, size)) = shape_data {
            let selection_changed = self.last_selection_id != Some(shape_id);
            let position_changed = self.last_position != position;
            let size_changed = self.last_size != size;

            // Update tracking
            self.last_selection_id = Some(shape_id);
            self.last_position = position;
            self.last_size = size;

            // Update inputs if values changed, but only if not focused (avoid fighting with user)
            if selection_changed || position_changed {
                if !self.x_input.focus_handle(cx).is_focused(window) {
                    self.x_input.update(cx, |input, cx| {
                        input.set_content(format!("{:.0}", position.x), cx);
                    });
                }
                if !self.y_input.focus_handle(cx).is_focused(window) {
                    self.y_input.update(cx, |input, cx| {
                        input.set_content(format!("{:.0}", position.y), cx);
                    });
                }
            }

            if selection_changed || size_changed {
                if !self.w_input.focus_handle(cx).is_focused(window) {
                    self.w_input.update(cx, |input, cx| {
                        input.set_content(format!("{:.0}", size.x), cx);
                    });
                }
                if !self.h_input.focus_handle(cx).is_focused(window) {
                    self.h_input.update(cx, |input, cx| {
                        input.set_content(format!("{:.0}", size.y), cx);
                    });
                }
            }
        } else {
            self.last_selection_id = None;
            self.last_position = Vec2::ZERO;
            self.last_size = Vec2::ZERO;
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

    fn apply_position_x(&mut self, cx: &mut Context<Self>) {
        let value = self.x_input.read(cx).content().to_string();
        if let Ok(x) = value.parse::<f32>() {
            self.canvas.update(cx, |canvas, cx| {
                if let Some(shape) = canvas
                    .shapes
                    .iter_mut()
                    .find(|s| canvas.selection.contains(&s.id))
                {
                    shape.position.x = x;
                    cx.notify();
                }
            });
        }
    }

    fn apply_position_y(&mut self, cx: &mut Context<Self>) {
        let value = self.y_input.read(cx).content().to_string();
        if let Ok(y) = value.parse::<f32>() {
            self.canvas.update(cx, |canvas, cx| {
                if let Some(shape) = canvas
                    .shapes
                    .iter_mut()
                    .find(|s| canvas.selection.contains(&s.id))
                {
                    shape.position.y = y;
                    cx.notify();
                }
            });
        }
    }

    fn apply_size_w(&mut self, cx: &mut Context<Self>) {
        let value = self.w_input.read(cx).content().to_string();
        if let Ok(w) = value.parse::<f32>() {
            if w > 0.0 {
                self.canvas.update(cx, |canvas, cx| {
                    if let Some(shape) = canvas
                        .shapes
                        .iter_mut()
                        .find(|s| canvas.selection.contains(&s.id))
                    {
                        shape.size.x = w;
                        cx.notify();
                    }
                });
            }
        }
    }

    fn apply_size_h(&mut self, cx: &mut Context<Self>) {
        let value = self.h_input.read(cx).content().to_string();
        if let Ok(h) = value.parse::<f32>() {
            if h > 0.0 {
                self.canvas.update(cx, |canvas, cx| {
                    if let Some(shape) = canvas
                        .shapes
                        .iter_mut()
                        .find(|s| canvas.selection.contains(&s.id))
                    {
                        shape.size.y = h;
                        cx.notify();
                    }
                });
            }
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
                // Position
                .child(
                    v_stack()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.ui_text_muted)
                                .child("Position"),
                        )
                        .child(
                            h_stack()
                                .gap(px(8.0))
                                .child(input_field("X", &self.x_input, theme, &colors, cx))
                                .child(input_field("Y", &self.y_input, theme, &colors, cx)),
                        ),
                )
                // Size
                .child(
                    v_stack()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.ui_text_muted)
                                .child("Size"),
                        )
                        .child(
                            h_stack()
                                .gap(px(8.0))
                                .child(input_field("W", &self.w_input, theme, &colors, cx))
                                .child(input_field("H", &self.h_input, theme, &colors, cx)),
                        ),
                )
                // Fill (read-only for now)
                .child(property_section_color(
                    "Fill",
                    theme,
                    shape.fill.as_ref().map(|f| f.color),
                ))
                // Stroke (read-only for now)
                .child(property_section_stroke(
                    "Stroke",
                    theme,
                    shape.stroke.as_ref(),
                ))
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
    h_stack()
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
                .border_color(theme.ui_border)
                .rounded(px(2.0))
                .text_xs()
                .text_color(theme.ui_text),
        )
}

fn property_section_color(
    label: &str,
    theme: &Theme,
    color: Option<gpui::Hsla>,
) -> impl IntoElement {
    v_stack()
        .gap(px(4.0))
        .child(
            div()
                .text_xs()
                .text_color(theme.ui_text_muted)
                .child(label.to_string()),
        )
        .child(
            h_stack().gap(px(8.0)).child(if let Some(c) = color {
                h_stack()
                    .gap(px(4.0))
                    .child(
                        div()
                            .size(px(20.0))
                            .rounded(px(2.0))
                            .border_1()
                            .border_color(theme.ui_border)
                            .bg(c),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.ui_text)
                            .child(format_color(c)),
                    )
            } else {
                h_stack()
                    .child(div().text_xs().text_color(theme.ui_text_muted).child("None"))
            }),
        )
}

fn property_section_stroke(
    label: &str,
    theme: &Theme,
    stroke: Option<&node_2::Stroke>,
) -> impl IntoElement {
    v_stack()
        .gap(px(4.0))
        .child(
            div()
                .text_xs()
                .text_color(theme.ui_text_muted)
                .child(label.to_string()),
        )
        .child(if let Some(s) = stroke {
            h_stack()
                .gap(px(8.0))
                .child(
                    div()
                        .size(px(20.0))
                        .rounded(px(2.0))
                        .border_1()
                        .border_color(theme.ui_border)
                        .bg(s.color),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.ui_text)
                        .child(format!("{:.0}px", s.width)),
                )
        } else {
            h_stack().child(div().text_xs().text_color(theme.ui_text_muted).child("None"))
        })
}

fn format_color(c: gpui::Hsla) -> String {
    format!("hsla({:.0}, {:.0}%, {:.0}%)", c.h * 360.0, c.s * 100.0, c.l * 100.0)
}
