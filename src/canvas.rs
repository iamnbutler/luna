use gpui::{App, AppContext, Context, ElementId, Entity, Global, IntoElement, Window};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanvasId(pub u64);

impl CanvasId {
    pub fn next() -> Self {
        static NEXT_CANVAS_ID: AtomicU64 = AtomicU64::new(1);
        CanvasId(NEXT_CANVAS_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanvasElementId(pub u64);

impl CanvasElementId {
    pub fn next() -> Self {
        static NEXT_CANVAS_ELEMENT_ID: AtomicU64 = AtomicU64::new(1);
        CanvasElementId(NEXT_CANVAS_ELEMENT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

// Canvas style definition
#[derive(Clone)]
pub struct CanvasStyle {
    pub background: gpui::Hsla,
    pub border_color: gpui::Hsla,
    pub fill_color: gpui::Hsla,
    pub scrollbar_thickness: gpui::Pixels,
    pub text: gpui::TextStyle,
}

impl CanvasStyle {
    pub fn new(cx: &App) -> Self {
        let theme = crate::Theme::get_global(cx);

        Self {
            background: theme.bg,
            border_color: gpui::hsla(0.0, 0.0, 0.0, 1.0),
            fill_color: theme.accent,
            ..Default::default()
        }
    }
}

impl Default for CanvasStyle {
    fn default() -> Self {
        Self {
            background: gpui::hsla(0.0, 0.0, 0.95, 1.0),
            border_color: gpui::hsla(0.0, 0.0, 0.0, 1.0),
            fill_color: gpui::hsla(0.6, 0.6, 0.6, 1.0),
            scrollbar_thickness: gpui::px(6.0),
            text: gpui::TextStyle::default(),
        }
    }
}

// Layout information for canvas painting
pub struct CanvasLayout {
    hitbox: gpui::Hitbox,
}

// The canvas element for rendering
pub struct CanvasElement {
    id: ElementId,
    style: CanvasStyle,
    data: Entity<CanvasData>,
}

impl CanvasElement {
    pub fn new(canvas_data: &Entity<CanvasData>, cx: &mut App) -> Self {
        Self {
            id: ElementId::Integer(CanvasElementId::next().0 as usize),
            style: CanvasStyle::new(cx),
            data: canvas_data.clone(),
        }
    }

    /// Helper method to paint a line using paint_quad
    fn paint_line(
        &self,
        start: gpui::Point<gpui::Pixels>,
        end: gpui::Point<gpui::Pixels>,
        thickness: gpui::Pixels,
        color: gpui::Hsla,
        window: &mut gpui::Window,
    ) {
        // For horizontal lines
        if start.y == end.y {
            let bounds = gpui::Bounds {
                origin: gpui::Point::new(
                    start.x.min(end.x),
                    start.y - thickness / 2.0,
                ),
                size: gpui::Size::new((end.x - start.x).abs(), thickness),
            };
            window.paint_quad(gpui::fill(bounds, color));
            return;
        }

        // For vertical lines
        if start.x == end.x {
            let bounds = gpui::Bounds {
                origin: gpui::Point::new(
                    start.x - thickness / 2.0,
                    start.y.min(end.y),
                ),
                size: gpui::Size::new(thickness, (end.y - start.y).abs()),
            };
            window.paint_quad(gpui::fill(bounds, color));
            return;
        }

        // For other lines, we would need more complex handling
        // but we're only using horizontal and vertical lines in our implementation
    }

    /// Paint the background layer of the canvas.
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

    /// Paint a simple square on the canvas
    pub fn paint_square(&self, layout: &CanvasLayout, window: &mut Window, cx: &mut App) {
        self.data.update(cx, |canvas, cx| {
            let scene_graph = canvas.scene_graph();
            scene_graph.paint_nodes(window, cx, layout.hitbox.bounds);
        });
    }

    /// Paint the origin marker and gridlines
    pub fn paint_origin_and_gridlines(&self, layout: &CanvasLayout, window: &mut Window, cx: &mut App) {
        window.paint_layer(layout.hitbox.bounds, |window| {
            // Calculate the center of the canvas as our origin
            let bounds = layout.hitbox.bounds;
            let center_x = bounds.origin.x + bounds.size.width / 2.0;
            let center_y = bounds.origin.y + bounds.size.height / 2.0;

            // Define grid spacing and grid color
            let grid_spacing = gpui::px(100.0);
            let grid_color = gpui::hsla(0.0, 0.0, 1.0, 0.1);
            let origin_color = gpui::hsla(0.0, 0.0, 0.0, 1.0);
            let grid_thickness = gpui::px(1.0);
            let origin_thickness = gpui::px(1.0);
            let origin_size = gpui::px(15.0); // Size of the plus sign

            // Paint gridlines in both directions
            // Horizontal gridlines
            let mut y_offset = grid_spacing;
            while y_offset < bounds.size.height / 2.0 {
                // Positive Y direction (up from origin)
                self.paint_line(
                    gpui::Point::new(bounds.origin.x, center_y - y_offset),
                    gpui::Point::new(bounds.origin.x + bounds.size.width, center_y - y_offset),
                    grid_thickness,
                    grid_color,
                    window,
                );

                // Negative Y direction (down from origin)
                self.paint_line(
                    gpui::Point::new(bounds.origin.x, center_y + y_offset),
                    gpui::Point::new(bounds.origin.x + bounds.size.width, center_y + y_offset),
                    grid_thickness,
                    grid_color,
                    window,
                );

                y_offset += grid_spacing;
            }

            // Vertical gridlines
            let mut x_offset = grid_spacing;
            while x_offset < bounds.size.width / 2.0 {
                // Positive X direction (right from origin)
                self.paint_line(
                    gpui::Point::new(center_x + x_offset, bounds.origin.y),
                    gpui::Point::new(center_x + x_offset, bounds.origin.y + bounds.size.height),
                    grid_thickness,
                    grid_color,
                    window,
                );

                // Negative X direction (left from origin)
                self.paint_line(
                    gpui::Point::new(center_x - x_offset, bounds.origin.y),
                    gpui::Point::new(center_x - x_offset, bounds.origin.y + bounds.size.height),
                    grid_thickness,
                    grid_color,
                    window,
                );

                x_offset += grid_spacing;
            }

            // Paint the origin marker (plus sign)
            // Horizontal line of the plus sign
            self.paint_line(
                gpui::Point::new(center_x - origin_size / 2.0, center_y),
                gpui::Point::new(center_x + origin_size / 2.0, center_y),
                origin_thickness,
                origin_color,
                window,
            );
            
            // Vertical line of the plus sign
            self.paint_line(
                gpui::Point::new(center_x, center_y - origin_size / 2.0),
                gpui::Point::new(center_x, center_y + origin_size / 2.0),
                origin_thickness,
                origin_color,
                window,
            );
            
            // Draw the main coordinate axes with a more subtle color
            let axes_color = gpui::hsla(0.0, 0.0, 0.5, 0.2);
            
            // X-axis
            self.paint_line(
                gpui::Point::new(bounds.origin.x, center_y),
                gpui::Point::new(bounds.origin.x + bounds.size.width, center_y),
                grid_thickness,
                axes_color,
                window,
            );
            
            // Y-axis
            self.paint_line(
                gpui::Point::new(center_x, bounds.origin.y),
                gpui::Point::new(center_x, bounds.origin.y + bounds.size.height),
                grid_thickness,
                axes_color,
                window,
            );
        });
    }
}

impl gpui::Element for CanvasElement {
    type RequestLayoutState = ();
    type PrepaintState = CanvasLayout;

    fn id(&self) -> Option<gpui::ElementId> {
        Some(self.id.clone())
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, ()) {
        // Request full available space for the canvas
        let mut style = gpui::Style::default();
        style.size.height = gpui::relative(1.).into();
        style.size.width = gpui::relative(1.).into();

        let layout_id = window.request_layout(style, None, cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> Self::PrepaintState {
        // Set up canvas styles
        let text_style = gpui::TextStyleRefinement {
            font_size: Some(self.style.text.font_size),
            line_height: Some(self.style.text.line_height),
            ..Default::default()
        };

        window.with_text_style(Some(text_style), |window| {
            window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
                let hitbox = window.insert_hitbox(bounds, false);
                CanvasLayout { hitbox }
            })
        })
    }

    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _: &mut Self::RequestLayoutState,
        layout: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        let text_style = gpui::TextStyleRefinement {
            font_size: Some(self.style.text.font_size),
            line_height: Some(self.style.text.line_height),
            ..Default::default()
        };

        window.with_text_style(Some(text_style), |window| {
            window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
                // Paint background layer
                self.paint_canvas_background(layout, window, cx);

                // Paint gridlines and origin marker
                self.paint_origin_and_gridlines(layout, window, cx);

                // Paint actual scene graph content (squares in quadrants)
                self.paint_square(layout, window, cx);
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

pub struct CanvasData {
    id: CanvasId,
    scene_graph: crate::scene_graph::SceneGraph,
}

impl CanvasData {
    pub fn new(id: CanvasId, _cx: &mut App) -> Self {
        // Create a new scene graph for this canvas
        let mut scene_graph = crate::scene_graph::SceneGraph::new(id);

        // Set up the demo scene with 4 squares
        scene_graph.setup_demo_scene();

        Self { id, scene_graph }
    }

    pub fn scene_graph(&self) -> &crate::scene_graph::SceneGraph {
        &self.scene_graph
    }

    pub fn id(&self) -> CanvasId {
        self.id
    }

    pub fn scene_graph_mut(&mut self) -> &mut crate::scene_graph::SceneGraph {
        &mut self.scene_graph
    }
}

impl Global for CanvasPages {}

// A collection of canvas pages
pub struct CanvasPages {
    pages: HashMap<CanvasId, Entity<CanvasData>>,
    active_id: Option<CanvasId>,
}

impl CanvasPages {
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
            active_id: None,
        }
    }

    pub fn add_canvas(&mut self, cx: &mut App) -> CanvasId {
        let id = CanvasId::next();
        let canvas = cx.new(|cx| CanvasData::new(id, cx));
        self.pages.insert(id, canvas);

        // If this is the first canvas, make it active
        if self.active_id.is_none() {
            self.active_id = Some(id);
        }

        id
    }

    pub fn remove_canvas(&mut self, id: CanvasId) -> Option<Entity<CanvasData>> {
        let canvas = self.pages.remove(&id);

        // If we removed the active canvas, select a new active one
        if self.active_id == Some(id) {
            self.active_id = self.pages.keys().next().copied();
        }

        canvas
    }

    pub fn get_canvas(&self, id: CanvasId) -> Option<&Entity<CanvasData>> {
        self.pages.get(&id)
    }

    pub fn active_canvas(&self) -> Option<&Entity<CanvasData>> {
        self.active_id.and_then(|id| self.pages.get(&id))
    }

    pub fn set_active_canvas(&mut self, id: CanvasId) -> bool {
        if self.pages.contains_key(&id) {
            self.active_id = Some(id);
            true
        } else {
            false
        }
    }

    pub fn active_id(&self) -> Option<CanvasId> {
        self.active_id
    }

    pub fn canvas_count(&self) -> usize {
        self.pages.len()
    }

    pub fn canvas_ids(&self) -> impl Iterator<Item = &CanvasId> {
        self.pages.keys()
    }
}
