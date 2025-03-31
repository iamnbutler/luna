#![allow(unused, dead_code)]
use gpui::{
    hsla, prelude::*, px, relative, App, ContentMask, DispatchPhase, ElementId, Entity, Focusable,
    Hitbox, Hsla, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Style,
    TextStyle, TextStyleRefinement, TransformationMatrix, Window,
};
use gpui::{point, size, Bounds, Point, Size};
use std::collections::HashSet;

/// Helper function to check if two bounds rectangles intersect
fn bounds_intersect(a: &Bounds<f32>, b: &Bounds<f32>) -> bool {
    // Check if one rectangle is to the left of the other
    if a.origin.x + a.size.width < b.origin.x || b.origin.x + b.size.width < a.origin.x {
        return false;
    }

    // Check if one rectangle is above the other
    if a.origin.y + a.size.height < b.origin.y || b.origin.y + b.size.height < a.origin.y {
        return false;
    }

    true
}

use crate::scene_graph::SceneGraph;
use crate::theme::Theme;
use crate::AppState;
use crate::{
    canvas::{register_canvas_action, ClearSelection, LunaCanvas},
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
            background: theme.tokens.background,
            cursor_color: theme.tokens.cursor,
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
    canvas: Entity<LunaCanvas>,
    style: CanvasStyle,
}

impl CanvasElement {
    pub fn new(
        canvas: &Entity<LunaCanvas>,
        scene_graph: &Entity<SceneGraph>,
        cx: &mut App,
    ) -> Self {
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

        register_canvas_action(canvas, window, LunaCanvas::clear_selection);
    }

    // handle_mouse_down, etc
    // Helper function to find the top node at a given point using scene graph for efficiency
    fn find_top_node_at_point(
        canvas: &LunaCanvas,
        window_point: Point<f32>,
        cx: &Context<LunaCanvas>,
    ) -> Option<NodeId> {
        // Convert window coordinate to canvas coordinate
        let canvas_point = canvas.window_to_canvas_point(window_point);

        // Instead of trying to access the private canvas_node directly,
        // we'll use the existing node methods to find a node at this point

        // Let's create a selection area of 1x1 pixels at the point
        let select_point_bounds = Bounds {
            origin: canvas_point,
            size: Size::new(1.0, 1.0),
        };

        // Test each node to see if it contains this point
        for node in &canvas.nodes {
            let node_bounds = node.bounds();
            if node_bounds.contains(&canvas_point) {
                return Some(node.id());
            }
        }

        None
    }

    fn handle_left_mouse_down(
        canvas: &mut LunaCanvas,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<LunaCanvas>,
    ) {
        if window.default_prevented() {
            return;
        }

        let position = event.position;
        let canvas_point = point(position.x.0, position.y.0);

        match canvas.active_tool {
            ToolKind::Selection => {
                // Attempt to find a node at the clicked point
                if let Some(node_id) = Self::find_top_node_at_point(canvas, canvas_point, cx) {
                    // Clicked on a node - select it

                    // If shift is not pressed, clear current selection first
                    let modifiers = event.modifiers;
                    if !modifiers.shift {
                        canvas.clear_selection(&ClearSelection, window, cx);
                    }

                    // If shift is pressed and node is already selected, deselect it
                    if modifiers.shift && canvas.is_node_selected(node_id) {
                        canvas.deselect_node(node_id);
                    } else {
                        // Otherwise select the node
                        canvas.select_node(node_id);
                    }

                    canvas.mark_dirty(cx);
                } else {
                    // Clicked on empty space - start a selection rectangle drag
                    // First clear selection if shift is not pressed
                    if !event.modifiers.shift {
                        canvas.clear_selection(&ClearSelection, window, cx);
                    }

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
        canvas: &mut LunaCanvas,
        event: &MouseUpEvent,
        window: &mut Window,
        cx: &mut Context<LunaCanvas>,
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
                        canvas.add_node(rect, cx);
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
        canvas: &mut LunaCanvas,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<LunaCanvas>,
    ) {
        let position = event.position;
        let canvas_point = point(position.x.0, position.y.0);

        // Handle selection box if we're dragging with the selection tool
        if let Some(active_drag) = canvas.active_drag.take() {
            // Update the drag with new position
            let new_drag = ActiveDrag {
                start_position: active_drag.start_position,
                current_position: position,
            };
            canvas.active_drag = Some(new_drag);

            // If we're using the selection tool, update the selection based on
            // which nodes intersect with the selection rectangle
            if canvas.active_tool == ToolKind::Selection {
                // Calculate the selection rectangle in canvas coordinates
                let start_pos = active_drag.start_position;
                let min_x = start_pos.x.0.min(position.x.0);
                let min_y = start_pos.y.0.min(position.y.0);
                let max_x = start_pos.x.0.max(position.x.0);
                let max_y = start_pos.y.0.max(position.y.0);

                // Convert to canvas coordinates
                let min_point = canvas.window_to_canvas_point(Point::new(min_x, min_y));
                let max_point = canvas.window_to_canvas_point(Point::new(max_x, max_y));

                // Create selection bounds
                let selection_bounds = Bounds {
                    origin: min_point,
                    size: Size::new(max_point.x - min_point.x, max_point.y - min_point.y),
                };

                // Pre-calculate all nodes that intersect with selection
                let nodes_in_selection: HashSet<NodeId> = canvas
                    .nodes
                    .iter()
                    .filter(|node| bounds_intersect(&selection_bounds, &node.bounds()))
                    .map(|node| node.id())
                    .collect();

                // Check if we want to add to existing selection (shift pressed)
                // or replace it (shift not pressed)
                if event.modifiers.shift {
                    // Add new nodes to selection
                    for node_id in nodes_in_selection {
                        canvas.select_node(node_id);
                    }
                } else {
                    // Replace selection
                    if nodes_in_selection != canvas.selected_nodes {
                        canvas.clear_selection(&ClearSelection, window, cx);
                        for node_id in nodes_in_selection {
                            canvas.select_node(node_id);
                        }
                    }
                }
            }

            canvas.mark_dirty(cx);
        }

        // Handle rectangle drawing
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
        canvas: &mut LunaCanvas,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<LunaCanvas>,
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
            let position = rounded_point(min_x, min_y);

            let rect_bounds = Bounds {
                origin: position,
                size: Size::new(width, height),
            };

            window.paint_quad(gpui::fill(rect_bounds, theme.tokens.overlay2.opacity(0.25)));
            window.paint_quad(gpui::outline(rect_bounds, theme.tokens.active_border));
            window.request_animation_frame();
        });
    }

    /// Paint a retangular element like a rectangle, square or frame
    /// as it is being created by clicking and dragging the tool
    fn paint_draw_rectangle(
        &self,
        new_node_id: NodeId,
        active_drag: &ActiveDrag,
        layout: &CanvasLayout,
        window: &mut Window,
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

        // Read canvas and app_state separately to avoid multiple borrows
        let canvas_read = self.canvas.read(cx);
        let app_state_entity = canvas_read.app_state().clone();

        let app_state = app_state_entity.read(cx);

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

    fn paint_nodes(&self, layout: &CanvasLayout, window: &mut Window, cx: &mut App) {
        let canvas = self.canvas.clone();

        // Collect ALL data we need up front to avoid any borrow issues
        struct NodeRenderInfo {
            node_id: NodeId,
            bounds: gpui::Bounds<Pixels>,
            fill_color: Option<Hsla>,
            border_color: Option<Hsla>,
            border_width: f32,
        }

        // Get all the data we need in one place
        let (nodes_to_render, theme, selected_node_ids) = canvas.update(cx, |canvas, cx| {
            let visible_nodes = canvas.visible_nodes(cx);
            let scene_graph = canvas.scene_graph().read(cx);
            let selected_nodes = canvas.selected_nodes.clone();
            let theme = canvas.theme.clone();

            // Collect all node rendering information into owned structures
            let mut nodes_to_render = Vec::new();

            for node in visible_nodes {
                let node_id = node.id();

                if let Some(scene_node_id) = scene_graph.get_scene_node_id(node_id) {
                    if let Some(world_bounds) = scene_graph.get_world_bounds(scene_node_id) {
                        nodes_to_render.push(NodeRenderInfo {
                            node_id,
                            bounds: gpui::Bounds {
                                origin: gpui::Point::new(
                                    gpui::Pixels(world_bounds.origin.x),
                                    gpui::Pixels(world_bounds.origin.y),
                                ),
                                size: gpui::Size::new(
                                    gpui::Pixels(world_bounds.size.width),
                                    gpui::Pixels(world_bounds.size.height),
                                ),
                            },
                            fill_color: node.fill(),
                            border_color: node.border_color(),
                            border_width: node.border_width(),
                        });
                    }
                }
            }

            (nodes_to_render, theme, selected_nodes)
        });

        window.paint_layer(layout.hitbox.bounds, |window| {
            // Paint each node with its transformation from the scene graph
            for node_info in &nodes_to_render {
                // Paint the fill if it exists
                if let Some(fill_color) = node_info.fill_color {
                    window.paint_quad(gpui::fill(node_info.bounds, fill_color));
                }

                // Paint the border if it exists
                if let Some(border_color) = node_info.border_color {
                    window.paint_quad(gpui::PaintQuad {
                        bounds: node_info.bounds,
                        corner_radii: (0.).into(),
                        background: gpui::transparent_black().into(),
                        border_widths: (node_info.border_width).into(),
                        border_color: border_color.into(),
                    });
                }

                // Draw selection indicator if the node is selected
                if selected_node_ids.contains(&node_info.node_id) {
                    // Create a slightly larger bounds for selection indicator
                    let selection_bounds = gpui::Bounds {
                        origin: gpui::Point::new(
                            node_info.bounds.origin.x - gpui::Pixels(2.0),
                            node_info.bounds.origin.y - gpui::Pixels(2.0),
                        ),
                        size: gpui::Size::new(
                            node_info.bounds.size.width + gpui::Pixels(4.0),
                            node_info.bounds.size.height + gpui::Pixels(4.0),
                        ),
                    };
                    // Use active_border for selection outlines per style guide
                    window.paint_quad(gpui::outline(selection_bounds, theme.tokens.active_border));
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
        // let key_context = self.canvas.update(cx, |canvas, cx| canvas.key_context());

        // window.set_key_context(key_context);

        // register_actions
        // register_key_listeners

        let text_style = TextStyleRefinement {
            font_size: Some(self.style.text.font_size),
            line_height: Some(self.style.text.line_height),
            ..Default::default()
        };

        window.with_text_style(Some(text_style), |window| {
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                // Clone the canvas to avoid multiple borrows of cx
                let canvas_clone = self.canvas.clone();
                self.paint_mouse_listeners(layout, window, cx);
                self.paint_canvas_background(layout, window, cx);
                self.paint_nodes(layout, window, cx);

                // Read canvas once to get all needed data
                let canvas_read = canvas_clone.read(cx);
                let active_drag = canvas_read.active_drag.clone();
                let active_element_draw = canvas_read.active_element_draw.clone();
                let active_tool = canvas_read.active_tool.clone();
                let theme = &canvas_read.theme;

                // Paint selection rectangle if dragging with selection tool
                if let Some(active_drag) = active_drag {
                    self.paint_selection(&active_drag, layout, window, theme);
                }

                // Paint rectangle preview if drawing with rectangle tool
                if let Some((node_id, node_type, drag)) = active_element_draw {
                    match active_tool {
                        ToolKind::Rectangle => {
                            self.paint_draw_rectangle(node_id, &drag, layout, window, cx);
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
