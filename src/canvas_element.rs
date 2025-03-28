#[allow(unused, dead_code)]
use gpui::prelude::*;
use gpui::{
    hsla, px, relative, solid_background, App, ContentMask, DispatchPhase, ElementId,
    ElementInputHandler, Entity, Focusable, Hitbox, Hsla, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Style, TextStyle, TextStyleRefinement, Window,
};

use crate::{
    canvas::{register_canvas_action, Canvas},
    node::RootNodeLayout,
    Theme,
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

        let mut click_count = event.click_count;
        let mut modifiers = event.modifiers;

        // do stuff

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

        cx.stop_propagation();
    }

    fn handle_mouse_drag(
        &self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Canvas>,
    ) {
        println!("mouse drag");

        // if canvas.selection_pending()  {
        // handle selection
        // } else if canvas.dragging() {
        // handle dragging node
        // } else {
        // return
        // }
    }

    fn handle_mouse_move(
        &self,
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

    // handle_mouse_drag, etc
    // handle_key_down, etc

    // layout_scrollbars
    // layout_dimension_guides
    // layout_root_nodes - the top level nodes on the canvas
    //   - these will determine how big the canvas needs to be in each cardinal direction
    // layout_overlays
    //   - these are any elements that should be rendered on top of the canvas
    //   - these use fixed positions that don't move when the canvas pans
    // layout_context_menu

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

                // let nodes = self.layout_nodes(..);
                let root_nodes = Vec::new();

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
                    // self.paint_nodes(..);
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
