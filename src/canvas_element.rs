#[allow(unused, dead_code)]
use gpui::{
    hsla, prelude::*, px, relative, solid_background, App, ContentMask, DispatchPhase, ElementId,
    ElementInputHandler, Entity, Focusable, Hitbox, Hsla, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Style, TextStyle, TextStyleRefinement, Window,
};
use gpui::{point, Bounds, Size, Point};

use crate::{
    canvas::{register_canvas_action, Canvas},
    interactivity::ActiveDrag,
    node::RootNodeLayout,
    util::{round_to_pixel, rounded_point},
    Theme, ToolKind,
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

        // End of drag, clean up selection
        if let Some(active_drag) = canvas.active_drag.take() {
            match canvas.active_tool {
                // handle per-tool post selection actions
                ToolKind::Rectangle => {
                    // canvas.create_rectangle_node(..);
                }
                _ => {}
            }

            canvas.active_drag = None;
        }

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
            for node_id in canvas.get_root_nodes() {
                if let Some(node) = canvas.nodes.get(&node_id) {
                    if let Some(bounds) = node.common().bounds() {
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

    fn paint_selection_bounds(
        &self,
        active_drag: &ActiveDrag,
        layout: &CanvasLayout,
        window: &mut Window,
        cx: &mut App,
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
            let theme = Theme::get_global(cx);
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

    pub fn paint_root_nodes(
        &self,
        layout: &CanvasLayout,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.paint_layer(layout.hitbox.bounds, |window| {
            for node in &layout.root_nodes {
                let bounds = Bounds {
                    origin: Point::new(Pixels(node.x), Pixels(node.y)),
                    size: Size::new(Pixels(node.width), Pixels(node.height)),
                };

                // Paint background
                window.paint_quad(gpui::fill(bounds, node.background_color));

                // Paint border if it exists
                if let Some(border_color) = node.border_color {
                    if node.border_width > 0.0 {
                        window.paint_quad(gpui::outline(bounds, border_color));
                    }
                }
            }
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
                // style.size.height = relative(1.).into();
                // style.size.width = relative(1.).into();
                style.size.height = px(500.).into();
                style.size.width = px(700.).into();

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
        window.set_view_id(self.canvas.entity_id());
        window.set_focus_handle(&focus_handle, cx);

        window.with_text_style(Some(text_style), |window| {
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                // todo: we probably need somethink like zed::editor::EditorSnapshot here

                let style = self.style.clone();
                let hitbox = window.insert_hitbox(bounds, false);

                let root_nodes = self.layout_root_nodes(window, cx);

                if !cx.has_active_drag() {
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

                canvas.update(cx, |canvas, cx| {
                    if let Some(active_drag) = canvas.active_drag.clone() {
                        self.paint_selection_bounds(&active_drag, layout, window, cx);
                    }
                });

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
