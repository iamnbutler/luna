use gpui::{App, AppContext, Context, Entity, Global, IntoElement, Window};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// Unique identifier for canvas pages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanvasId(pub u64);

impl CanvasId {
    pub fn next() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        CanvasId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
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
    id: CanvasId,
    style: CanvasStyle,
}

impl CanvasElement {
    pub fn new(id: CanvasId, cx: &mut App) -> Self {
        let style = CanvasStyle::new(cx);
        Self { id, style }
    }

    pub fn id(&self) -> CanvasId {
        self.id
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
        window.paint_layer(layout.hitbox.bounds, |window| {
            // Calculate square position in the center of the canvas
            let bounds = layout.hitbox.bounds;
            let square_size = gpui::px(150.0);

            let square_bounds = gpui::Bounds {
                origin: gpui::Point::new(
                    bounds.origin.x + (bounds.size.width - square_size) / 2.0,
                    bounds.origin.y + (bounds.size.height - square_size) / 2.0,
                ),
                size: gpui::Size::new(square_size, square_size),
            };

            // Paint the square with fill
            window.paint_quad(gpui::fill(square_bounds, self.style.fill_color));

            // Paint the square border
            window.paint_quad(gpui::outline(
                square_bounds,
                self.style.border_color,
                gpui::BorderStyle::Solid,
            ));

            window.request_animation_frame();
        });
    }
}

impl gpui::Element for CanvasElement {
    type RequestLayoutState = ();
    type PrepaintState = CanvasLayout;

    fn id(&self) -> Option<gpui::ElementId> {
        None
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
                // Paint background and square
                self.paint_canvas_background(layout, window, cx);
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
}

impl CanvasData {
    pub fn new(id: CanvasId, _cx: &mut App) -> Self {
        Self { id }
    }
}

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


