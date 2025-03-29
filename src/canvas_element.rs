#![allow(unused_variables)]
use gpui::{
    hsla, prelude::*, px, relative, App, ContentMask, DispatchPhase, ElementId, Entity, Focusable,
    Hitbox, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Style,
    TextStyle, TextStyleRefinement, Window,
};
use gpui::{point, Bounds, Point, Size};

use crate::{
    canvas::{register_canvas_action, Canvas},
    interactivity::ActiveDrag,
    node::{CanvasNode, NodeId, NodeType, RectangleNode, RootNodeLayout},
    util::{round_to_pixel, rounded_point},
    GlobalState, Theme, ToolKind,
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
    root_nodes: Vec<RootNodeLayout>,
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
    pub fn new(canvas: &Entity<Canvas>, cx: &mut App) -> Self {
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

                        // Convert window coordinates to canvas coordinates using our standard method
                        let canvas_point = Self::window_to_canvas_coordinates(
                            window,
                            cx,
                            Point::new(min_x, min_y),
                        );
                        let rel_x = canvas_point.x;
                        let rel_y = canvas_point.y;

                        println!(
                            "Window coords: ({}, {}), Converted to canvas coords: ({}, {})",
                            min_x, min_y, rel_x, rel_y
                        );

                        // Create a new rectangle node
                        let mut rect = RectangleNode::new(node_id);

                        // Set position and size using canvas coordinates
                        rect.common_mut().set_position(rel_x, rel_y);
                        rect.common_mut().set_size(width, height);

                        // Set styles from global state
                        rect.common_mut()
                            .set_fill(Some(state.current_background_color));
                        rect.common_mut()
                            .set_border(Some(state.current_border_color), 1.0);

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

    fn layout_root_nodes(&self, window: &mut Window, cx: &mut App) -> Vec<RootNodeLayout> {
        let mut root_layouts = Vec::new();

        self.canvas.update(cx, |canvas, _cx| {
            (&mut *canvas).update_layout();

            println!("Found {} root nodes", canvas.get_root_nodes().len());
            for node_id in canvas.get_root_nodes() {
                if let Some(node) = canvas.nodes.get(&node_id) {
                    if let Some(bounds) = node.common().bounds() {
                        println!(
                            "Root node at ({}, {}) with size {}x{}",
                            bounds.origin.x, bounds.origin.y, bounds.size.width, bounds.size.height
                        );
                        root_layouts.push(RootNodeLayout {
                            id: node_id,
                            x: bounds.origin.x,
                            y: bounds.origin.y,
                            width: bounds.size.width,
                            height: bounds.size.height,
                            background_color: node.common().fill.unwrap_or(Hsla::white()),
                            border_color: node.common().border_color,
                            border_width: node.common().border_width,
                            border_radius: node.common().corner_radius,
                        });
                    }
                }
            }
        });

        root_layouts
    }

    // handle_mouse_drag, etc
    // handle_key_down, etc

    // layout_scrollbars
    // layout_dimension_guides
    // layout_overlays
    //   - these are any elements that should be rendered on top of the canvas
    //   - these use fixed positions that don't move when the canvas pans
    // layout_context_menu

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

        window.paint_layer(layout.hitbox.bounds, |window| {
            // When painting in a layer, coordinates are relative to the layer's origin
            // Convert from window coordinates to layer coordinates
            // by subtracting the layer origin from the window coordinates
            let layer_x = min_x.0 - layout.hitbox.bounds.origin.x.0;
            let layer_y = min_y.0 - layout.hitbox.bounds.origin.y.0;

            // Round once after all coordinate conversions for pixel-perfect rendering
            let position = rounded_point(Pixels(layer_x), Pixels(layer_y));

            let rect_bounds = Bounds {
                origin: position,
                size: Size::new(width, height),
            };

            let new_node_id = NodeId::new(new_node_id);

            let new_node = RootNodeLayout {
                id: new_node_id,
                x: position.x.0,
                y: position.y.0,
                width: width.0,
                height: height.0,
                background_color: state.current_background_color,
                border_color: Some(state.current_border_color),
                border_width: 1.0,
                border_radius: 0.0,
            };

            window.paint_quad(gpui::fill(rect_bounds, new_node.background_color));
            if let Some(border_color) = new_node.border_color {
                window.paint_quad(gpui::outline(rect_bounds, border_color));
            }
            window.request_animation_frame();
        });
    }

    // render_scrollbars
    // render_dimension_guides
    // render_root_nodes
    // render_context_menu

    // paint_canvas_background might also include any features like:
    // - canvas grids
    // - background images or textures
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

    pub fn paint_root_nodes(&self, layout: &CanvasLayout, window: &mut Window, cx: &mut App) {
        self.canvas.update(cx, |canvas, _cx| {
            window.paint_layer(layout.hitbox.bounds, |window| {
                // Calculate the center of the canvas in window coordinates
                let canvas_center_x =
                    layout.hitbox.bounds.origin.x.0 + layout.hitbox.bounds.size.width.0 / 2.0;
                let canvas_center_y =
                    layout.hitbox.bounds.origin.y.0 + layout.hitbox.bounds.size.height.0 / 2.0;

                for node in &layout.root_nodes {
                    // Since (0,0) is at the center of the canvas according to the comment,
                    // We need to offset by the canvas center to position correctly
                    let adjusted_bounds = Bounds {
                        origin: Point::new(
                            Pixels(canvas_center_x + node.x),
                            Pixels(canvas_center_y + node.y),
                        ),
                        size: Size::new(
                            Pixels(node.width * canvas.zoom()),
                            Pixels(node.height * canvas.zoom()),
                        ),
                    };

                    // Paint background
                    window.paint_quad(gpui::fill(adjusted_bounds, node.background_color));

                    // Paint border if it exists
                    if let Some(border_color) = node.border_color {
                        if node.border_width > 0.0 {
                            window.paint_quad(gpui::outline(adjusted_bounds, border_color));
                        }
                    }
                }
            });
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

    // paint_scrollbars
    // paint_dimension_guides
    // paint_root_nodes
    // paint_context_menu

    // data_furthest_node_positions

    /// Convert window coordinates to canvas-centered coordinates accounting for UI elements
    /// that offset the canvas (like sidebars)
    ///
    /// This is a static method that can be called from anywhere, including event handlers
    pub fn window_to_canvas_coordinates(
        window: &Window,
        cx: &App,
        window_point: Point<f32>,
    ) -> Point<f32> {
        // Get the GlobalState to check sidebar visibility
        let state = GlobalState::get(cx);

        // Step 1: Adjust for UI element offsets (like the sidebar)
        let mut adjusted_point = window_point;

        // Only adjust for sidebar if it's visible
        if !state.hide_sidebar {
            // Subtract sidebar width to get coordinates relative to the canvas area
            adjusted_point.x -= state.sidebar_width.0;
        }

        // Step 2: Calculate center of the actual canvas area (not full viewport)
        let canvas_width = window.viewport_size().width.0
            - (if !state.hide_sidebar {
                state.sidebar_width.0
            } else {
                0.0
            });
        let canvas_height = window.viewport_size().height.0;

        let canvas_center_x = canvas_width / 2.0;
        let canvas_center_y = canvas_height / 2.0;

        // Step 3: Convert to canvas-centered coordinates
        let canvas_x = adjusted_point.x - canvas_center_x;
        let canvas_y = adjusted_point.y - canvas_center_y;

        Point::new(canvas_x, canvas_y)
    }

    /// Convert canvas-centered coordinates to window coordinates accounting for UI elements
    /// that offset the canvas (like sidebars)
    #[allow(unused)]
    pub fn canvas_to_window_coordinates(
        window: &Window,
        cx: &App,
        canvas_point: Point<f32>,
    ) -> Point<f32> {
        // Get the GlobalState to check sidebar visibility
        let state = GlobalState::get(cx);

        // Step 1: Calculate center of the actual canvas area (not full viewport)
        let canvas_width = window.viewport_size().width.0
            - (if !state.hide_sidebar {
                state.sidebar_width.0
            } else {
                0.0
            });
        let canvas_height = window.viewport_size().height.0;

        let canvas_center_x = canvas_width / 2.0;
        let canvas_center_y = canvas_height / 2.0;

        // Step 2: Convert from canvas-centered to window coordinates
        let window_x = canvas_point.x + canvas_center_x;
        let window_y = canvas_point.y + canvas_center_y;

        let mut adjusted_point = Point::new(window_x, window_y);

        // Step 3: Add sidebar width if visible
        if !state.hide_sidebar {
            adjusted_point.x += state.sidebar_width.0;
        }

        adjusted_point
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
        let focus_handle = self.canvas.focus_handle(cx);
        window.set_focus_handle(&focus_handle, cx);

        window.with_text_style(Some(text_style), |window| {
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                // todo: we probably need somethink like zed::editor::EditorSnapshot here

                let style = self.style.clone();
                let hitbox = window.insert_hitbox(bounds, false);

                let root_nodes = self.layout_root_nodes(window, cx);

                // Check for active drags in the canvas itself instead of using cx.has_active_drag()
                let has_active_drag = self
                    .canvas
                    .update(cx, |canvas, _| canvas.active_drag.is_some());

                if !has_active_drag {
                    // anything that shouldn't be painted when
                    // dragging goes in here

                    // let context_menu = self.layout_context_menu(..
                    // );
                }

                CanvasLayout { hitbox, root_nodes }
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
        let focus_handle = self.canvas.focus_handle(cx);
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

                if !layout.root_nodes.is_empty() {
                    self.paint_root_nodes(layout, window, cx);
                }

                // Get all canvas data we'll need for painting first (this needs a mutable borrow)
                let (active_drag, active_element_draw, active_tool) =
                    canvas.update(cx, |canvas, _| {
                        (
                            canvas.active_drag.clone(),
                            canvas.active_element_draw.clone(),
                            canvas.active_tool.clone(),
                        )
                    });

                // Then get theme and global state (this only needs immutable borrow)
                let theme = Theme::get_global(cx);
                let state = GlobalState::get(cx);

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
                            );
                        }
                        _ => {}
                    }
                }

                // paint_scrollbars
                // paint_context_menu
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
