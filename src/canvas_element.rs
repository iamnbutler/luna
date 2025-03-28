#[allow(unused, dead_code)]
use gpui::prelude::*;
use gpui::{
    hsla, px, relative, ContentMask, ElementId, Entity, Focusable, Hsla, Pixels, Style, TextStyle,
    TextStyleRefinement,
};

use crate::{canvas::Canvas, node::RootNodeLayout};

#[derive(Clone)]
pub struct CanvasStyle {
    pub background: Hsla,
    pub cursor_color: Hsla,
    pub scrollbar_thickness: Pixels,
    pub text: TextStyle,
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
    pub fn new(canvas: &Entity<Canvas>, style: CanvasStyle) -> Self {
        Self {
            canvas: canvas.clone(),
            style,
        }
    }

    // handle_mouse_down, etc
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

    // paint_canvas_background
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
        // we prepaint in
        self.canvas.update(cx, |canvas, cx| {
            let layout_id = {
                let mut style = Style::default();
                // TODO: impl actual size
                // style.size.height = relative(1.).into();
                // style.size.width = relative(1.).into();
                style.size.height = px(800.).into();
                style.size.width = px(800.).into();

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

                // let nodes = self.layout_nodes(..);
                let nodes = Vec::new();

                if !cx.has_active_drag() {
                    // anything that shouldn't be painted when
                    // dragging goes in here

                    // let context_menu = self.layout_context_menu(..
                    // );
                }

                CanvasLayout { root_nodes: nodes }
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

        // register_actions
        // register_key_listeners

        let text_style = TextStyleRefinement {
            font_size: Some(self.style.text.font_size),
            line_height: Some(self.style.text.line_height),
            ..Default::default()
        };

        window.with_text_style(Some(text_style), |window| {
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                // self.paint_canvas_background(..);

                if !layout.root_nodes.is_empty() {
                    // self.paint_nodes(..);
                }

                // paint_scrollbars
                // paint_context_menu
            })
        })
    }
}

impl IntoElement for CanvasElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}
