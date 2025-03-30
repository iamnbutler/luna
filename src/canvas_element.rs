#![allow(unused, dead_code)]
use gpui::{
    hsla, prelude::*, px, relative, App, ContentMask, DispatchPhase, ElementId, Entity, Focusable,
    Hitbox, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Style,
    TextStyle, TextStyleRefinement, Window,
};
use gpui::{point, Bounds, Point, Size};

use crate::scene_graph::SceneGraph;
use crate::theme::Theme;
use crate::{
    canvas::{register_canvas_action, Canvas},
    interactivity::ActiveDrag,
    node::{NodeCommon, NodeId, NodeLayout, NodeType, RectangleNode},
    util::{round_to_pixel, rounded_point},
    GlobalState, ToolKind,
};

#[derive(Clone)]
pub struct CanvasStyle {
    pub background: Hsla,
    pub cursor_color: Hsla,
    pub scrollbar_thickness: Pixels,
    pub text: TextStyle,
}

impl CanvasStyle {
    pub fn new(cx: &App) -> Self {
        let theme = Theme::get_global(cx);

        Self {
            background: theme.canvas_color,
            cursor_color: theme.cursor_color,
            ..Default::default()
        }
    }
}

impl Default for CanvasStyle {
    fn default() -> Self {
        Self {
            background: hsla(0.0, 0.0, 0.0, 1.0),
            cursor_color: hsla(0.0, 0.0, 1.0, 1.0),
            scrollbar_thickness: px(6.0),
            text: TextStyle::default(),
        }
    }
}

pub struct CanvasLayout {
    hitbox: Hitbox,
}

/// CanvasElement uses  prefixes for identifying the role of methods within the canvas.
///
/// - handle_: handle user input events
/// - layout_: layout elements within the canvas
/// - paint_: paint elements within the canvas
/// - data_:  returns some derived data for other methods to use within the canvas
pub struct CanvasElement {
    canvas: Entity<Canvas>,
    style: CanvasStyle,
}

impl CanvasElement {
    pub fn new(canvas: &Entity<Canvas>, scene_graph: &Entity<SceneGraph>, cx: &mut App) -> Self {
        let style = CanvasStyle::new(cx);

        Self {
            canvas: canvas.clone(),
            style,
        }
    }

    pub fn register_actions(&self, window: &mut Window, cx: &mut App) {
        let canvas = &self.canvas;
        canvas.update(cx, |canvas, cx| {
            for action in canvas.actions.borrow().values() {
                (action)(window, cx)
            }
        });

        register_canvas_action(canvas, window, Canvas::clear_selection);
    }

    // handle_mouse_down, etc
    fn handle_left_mouse_down(
        canvas: &mut Canvas,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Canvas>,
    ) {
        if window.default_prevented() {
            return;
        }

        println!("Left mouse down");

        let position = event.position;

        let canvas_point = point(position.x.0, position.y.0);

        match canvas.active_tool {
            ToolKind::Selection => {
                if let Some(node_id) = canvas.top_node_at_point(canvas_point) {
                    canvas.select_node(node_id);
                    canvas.mark_dirty(cx);
                } else {
                    canvas.active_drag = Some(ActiveDrag {
                        start_position: position,
                        current_position: position,
                    });
                    canvas.mark_dirty(cx);
                }
            }
            ToolKind::Rectangle => {
                // Use the generate_id method directly since it already returns the correct type
                let new_node_id = canvas.generate_id();

                let active_drag = ActiveDrag {
                    start_position: position,
                    current_position: position,
                };
                canvas.active_element_draw = Some((new_node_id, NodeType::Rectangle, active_drag));
                canvas.mark_dirty(cx);
            }
            _ => {}
        }

        cx.stop_propagation();
    }

    fn handle_left_mouse_up(
        canvas: &mut Canvas,
        event: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<Canvas>,
    ) {
        // check if selection is pending
        // if so, clear it and fire any selection events

        println!("Left mouse up");

        let position = event.position;
        let canvas_point = point(position.x.0, position.y.0);
        let state = GlobalState::get(cx);
        let app_state = canvas.app_state().clone().read(cx);
        let current_background_color = app_state.current_background_color.clone();
        let current_border_color = app_state.current_border_color.clone();

        // Check if we have an active element draw operation
        if let Some((node_id, node_type, active_drag)) = canvas.active_element_draw.take() {
            match (node_type, &canvas.active_tool) {
                (NodeType::Rectangle, ToolKind::Rectangle) => {
                    // Calculate rectangle dimensions
                    let start_pos = active_drag.start_position;
                    let end_pos = active_drag.current_position;

                    let min_x = start_pos.x.0.min(end_pos.x.0);
                    let min_y = start_pos.y.0.min(end_pos.y.0);
                    let width = (start_pos.x.0 - end_pos.x.0).abs();
                    let height = (start_pos.y.0 - end_pos.y.0).abs();

                    // Only create a rectangle if it has meaningful dimensions
                    if width >= 2.0 && height >= 2.0 {
                        // Get the global state for colors and sidebar info
                        let state = GlobalState::get(cx);

                        // Convert window coordinates to canvas coordinates
                        let canvas_point = canvas.window_to_canvas_point(Point::new(min_x, min_y));
                        let rel_x = canvas_point.x;
                        let rel_y = canvas_point.y;

                        println!(
                            "Window coords: ({}, {}), Converted to canvas coords: ({}, {})",
                            min_x, min_y, rel_x, rel_y
                        );

                        // Create a new rectangle node
                        let mut rect = RectangleNode::new(node_id);

                        // Set position and size
                        *rect.layout_mut() = NodeLayout::new(rel_x, rel_y, width, height);

                        // Set colors
                        rect.set_fill(Some(current_background_color));
                        rect.set_border(Some(current_border_color), 1.0);

                        // Add the node to the canvas
                        canvas.add_node(rect);
                        canvas.mark_dirty(cx);
                    }
                }
                _ => {}
            }
        }

        // End of drag, clean up any remaining selection
        canvas.active_drag.take();

        cx.stop_propagation();
    }

    fn handle_mouse_drag(
        canvas: &mut Canvas,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Canvas>,
    ) {
        println!("mouse drag");

        let position = event.position;

        let canvas_point = point(position.x.0, position.y.0);

        // if we are selecting, check if we have entered the bounds of
        // any root node. If so, select it.

        if let Some(active_drag) = canvas.active_drag.take() {
            canvas.active_drag = Some(ActiveDrag {
                start_position: active_drag.start_position,
                current_position: position,
            });
        }

        if let Some(active_draw) = canvas.active_element_draw.take() {
            match canvas.active_tool {
                ToolKind::Rectangle => {
                    let new_drag = ActiveDrag {
                        start_position: active_draw.2.start_position,
                        current_position: position,
                    };
                    canvas.active_element_draw = Some((active_draw.0, active_draw.1, new_drag));
                    canvas.mark_dirty(cx);
                }
                _ => {}
            }
        }
    }

    fn handle_mouse_move(
        canvas: &mut Canvas,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Canvas>,
    ) {
        // if canvas.position_has_hitbox()  {
        // handle hover event
        // } else {
        // return
        // }
    }

    fn paint_selection(
        &self,
        active_drag: &ActiveDrag,
        layout: &CanvasLayout,
        window: &mut Window,
        theme: &Theme,
    ) {
        let min_x = round_to_pixel(
            active_drag
                .start_position
                .x
                .min(active_drag.current_position.x),
        );
        let min_y = round_to_pixel(
            active_drag
                .start_position
                .y
                .min(active_drag.current_position.y),
        );
        let width =
            round_to_pixel((active_drag.start_position.x - active_drag.current_position.x).abs());
        let height =
            round_to_pixel((active_drag.start_position.y - active_drag.current_position.y).abs());

        window.paint_layer(layout.hitbox.bounds, |window| {
            // Use the theme parameter instead of getting it from cx
            // Round the position to ensure pixel-perfect rendering
            let position = rounded_point(min_x, min_y);

            let rect_bounds = Bounds {
                origin: position,
                size: Size::new(width, height),
            };

            window.paint_quad(gpui::fill(rect_bounds, theme.selected.opacity(0.08)));
            window.paint_quad(gpui::outline(rect_bounds, theme.selected));
            window.request_animation_frame();
        });
    }

    /// Paint a retangular element like a rectangle, square or frame
    /// as it is being created by clicking and dragging the tool
    fn paint_draw_rectangle(
        &self,
        new_node_id: usize,
        active_drag: &ActiveDrag,
        layout: &CanvasLayout,
        window: &mut Window,
        state: &GlobalState,
        cx: &App,
    ) {
        // Get the raw cursor positions directly from the drag event
        // These are in absolute window coordinates where the mouse is positioned
        let start_pos = active_drag.start_position;
        let current_pos = active_drag.current_position;

        // Calculate rectangle bounds in window coordinates
        // Don't round yet to avoid accumulating rounding errors
        let min_x = start_pos.x.min(current_pos.x);
        let min_y = start_pos.y.min(current_pos.y);
        let width = (start_pos.x - current_pos.x).abs();
        let height = (start_pos.y - current_pos.y).abs();

        // Round once after all coordinate conversions for pixel-perfect rendering
        let position = rounded_point(min_x, min_y);

        let rect_bounds = Bounds {
            origin: position,
            size: Size::new(width, height),
        };

        let app_state = self.canvas.read(cx).app_state().clone().read(cx);

        window.paint_quad(gpui::fill(rect_bounds, app_state.current_background_color));
        window.paint_quad(gpui::outline(rect_bounds, app_state.current_border_color));
        window.request_animation_frame();
    }

    /// Paint the background layer of the canvas.
    ///
    /// Everything on this layer has the same draw order.
    pub fn paint_canvas_background(
        &self,
        layout: &CanvasLayout,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.paint_layer(layout.hitbox.bounds, |window| {
            window.paint_quad(gpui::fill(layout.hitbox.bounds, self.style.background));
        });
    }

    /// Register mouse listeners like click, hover and drag events.
    ///
    /// Despite not being visually "painted", mouse listeners are registered
    /// using `window.on_{}_event`, which is only available in the paint phase.
    ///
    /// Thus the `paint` prefix.
    fn paint_mouse_listeners(&mut self, layout: &CanvasLayout, window: &mut Window, cx: &mut App) {
        window.on_mouse_event({
            let canvas = self.canvas.clone();
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble {
                    match event.button {
                        MouseButton::Left => canvas.update(cx, |canvas, cx| {
                            Self::handle_left_mouse_down(canvas, event, window, cx);
                        }),
                        MouseButton::Right => canvas.update(cx, |canvas, cx| {
                            // todo
                        }),
                        _ => {}
                    }
                }
            }
        });

        window.on_mouse_event({
            let canvas = self.canvas.clone();
            move |event: &MouseUpEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble {
                    match event.button {
                        MouseButton::Left => canvas.update(cx, |canvas, cx| {
                            Self::handle_left_mouse_up(canvas, event, window, cx)
                        }),
                        MouseButton::Right => canvas.update(cx, |canvas, cx| {
                            // todo
                        }),
                        _ => {}
                    }
                }
            }
        });

        window.on_mouse_event({
            let canvas = self.canvas.clone();
            move |event: &MouseMoveEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble {
                    canvas.update(cx, |canvas, cx| {
                        if event.pressed_button == Some(MouseButton::Left)
                            || event.pressed_button == Some(MouseButton::Middle)
                        {
                            Self::handle_mouse_drag(canvas, event, window, cx)
                        }

                        Self::handle_mouse_move(canvas, event, window, cx)
                    });
                }
            }
        });
    }

    fn paint_nodes(
        &self,
        layout: &CanvasLayout,
        window: &mut Window,
        theme: &Theme,
        cx: &App, // Change from &mut App to &App since we only need immutable access
    ) {
        // Fetch all visible nodes from the canvas
        let canvas = self.canvas.read(cx);
        let visible_nodes = canvas.visible_nodes();
        let selected_nodes = &canvas.selected_nodes;
        let zoom = canvas.zoom();
        let scroll_position = canvas.content_bounds().origin;

        window.paint_layer(layout.hitbox.bounds, |window| {
            // Paint each node
            for node in visible_nodes {
                // Get node position and size from the layout
                let layout = node.layout();

                // Apply zoom and scroll transformations
                let adjusted_x = (layout.x - scroll_position.x) * zoom;
                let adjusted_y = (layout.y - scroll_position.y) * zoom;
                let adjusted_width = layout.width * zoom;
                let adjusted_height = layout.height * zoom;

                // Create bounds for the node
                let bounds = Bounds {
                    origin: Point::new(gpui::Pixels(adjusted_x), gpui::Pixels(adjusted_y)),
                    size: Size::new(gpui::Pixels(adjusted_width), gpui::Pixels(adjusted_height)),
                };

                // Paint the fill if it exists
                if let Some(fill_color) = node.fill() {
                    window.paint_quad(gpui::fill(bounds, fill_color));
                }

                // Paint the border if it exists
                if let Some(border_color) = node.border_color() {
                    window.paint_quad(gpui::outline(bounds, border_color));
                }

                // Draw selection indicator if the node is selected
                if selected_nodes.contains(&node.id()) {
                    // Create a slightly larger bounds for selection indicator
                    let selection_bounds = Bounds {
                        origin: Point::new(
                            bounds.origin.x - gpui::Pixels(2.0),
                            bounds.origin.y - gpui::Pixels(2.0),
                        ),
                        size: Size::new(
                            bounds.size.width + gpui::Pixels(4.0),
                            bounds.size.height + gpui::Pixels(4.0),
                        ),
                    };
                    window.paint_quad(gpui::outline(selection_bounds, theme.selected));
                }
            }

            window.request_animation_frame();
        });
    }
}

impl Element for CanvasElement {
    type RequestLayoutState = ();
    type PrepaintState = CanvasLayout;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, ()) {
        // prepare the overall dimensions of the canvas before
        // we prepaint it
        self.canvas.update(cx, |canvas, cx| {
            let layout_id = {
                let mut style = Style::default();
                // TODO: impl actual size
                style.size.height = relative(1.).into();
                style.size.width = relative(1.).into();
                // style.size.height = px(500.).into();
                // style.size.width = px(700.).into();

                // TODO: use data_furthest_node_positions to calculate
                // how big the initial canvas should be
                window.request_layout(style, None, cx)
            };
            (layout_id, ())
        })
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> Self::PrepaintState {
        // set up canvas styles
        let text_style = TextStyleRefinement {
            font_size: Some(self.style.text.font_size),
            line_height: Some(self.style.text.line_height),
            ..Default::default()
        };
        // let focus_handle = self.focus_handle(cx);
        // window.set_focus_handle(&focus_handle, cx);

        window.with_text_style(Some(text_style), |window| {
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                // todo: we probably need somethink like zed::editor::EditorSnapshot here

                let style = self.style.clone();
                let hitbox = window.insert_hitbox(bounds, false);

                // Check for active drags in the canvas itself
                let has_active_drag = {
                    let canvas = self.canvas.read(cx);
                    canvas.active_drag.is_some()
                };

                if !has_active_drag {
                    // anything that shouldn't be painted when
                    // dragging goes in here
                }

                CanvasLayout { hitbox }
            })
        })
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        layout: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        let canvas = self.canvas.clone();
        let key_context = self.canvas.update(cx, |canvas, cx| canvas.key_context());

        window.set_key_context(key_context);

        // register_actions
        // register_key_listeners

        let text_style = TextStyleRefinement {
            font_size: Some(self.style.text.font_size),
            line_height: Some(self.style.text.line_height),
            ..Default::default()
        };

        window.with_text_style(Some(text_style), |window| {
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                self.paint_mouse_listeners(layout, window, cx);
                self.paint_canvas_background(layout, window, cx);

                // Get theme and global state
                let theme = Theme::get_global(cx);
                let state = GlobalState::get(cx);

                // Clone the canvas to avoid multiple borrows of cx
                let canvas_clone = self.canvas.clone();

                // First paint the nodes (this uses read-only access to canvas)
                self.paint_nodes(layout, window, theme, cx);

                // Now get any needed data for additional paint operations
                let (active_drag, active_element_draw, active_tool) = {
                    let canvas = canvas_clone.read(cx);
                    (
                        canvas.active_drag.clone(),
                        canvas.active_element_draw.clone(),
                        canvas.active_tool.clone(),
                    )
                };

                // Paint selection rectangle if dragging with selection tool
                if let Some(active_drag) = active_drag {
                    self.paint_selection(&active_drag, layout, window, theme);
                }

                // Paint rectangle preview if drawing with rectangle tool
                if let Some(element_draw) = active_element_draw {
                    match active_tool {
                        ToolKind::Rectangle => {
                            self.paint_draw_rectangle(
                                element_draw.0 .0,
                                &element_draw.2,
                                layout,
                                window,
                                state,
                                cx,
                            );
                        }
                        _ => {}
                    }
                }
            });
        })
    }
}

impl IntoElement for CanvasElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}
