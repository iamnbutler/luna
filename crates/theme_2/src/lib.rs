//! Minimal theming for Luna.
//!
//! Provides colors for the canvas, selection, and shape defaults.

use gpui::Hsla;

/// Theme colors for the canvas editor.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Canvas background
    pub canvas_background: Hsla,

    /// Selection indicator color
    pub selection: Hsla,

    /// Hover indicator color
    pub hover: Hsla,

    /// Default stroke color for new shapes
    pub default_stroke: Hsla,

    /// Grid lines (if shown)
    pub grid: Hsla,

    /// UI background
    pub ui_background: Hsla,

    /// UI border
    pub ui_border: Hsla,

    /// UI text
    pub ui_text: Hsla,

    /// UI text muted
    pub ui_text_muted: Hsla,
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

impl Theme {
    pub fn light() -> Self {
        Self {
            canvas_background: gpui::white(),
            selection: hsla(0.58, 0.9, 0.5, 1.0),  // Blue
            hover: hsla(0.58, 0.9, 0.5, 0.3),      // Blue transparent
            default_stroke: gpui::black(),
            grid: hsla(0.0, 0.0, 0.9, 1.0),        // Light gray
            ui_background: hsla(0.0, 0.0, 0.98, 1.0),
            ui_border: hsla(0.0, 0.0, 0.9, 1.0),
            ui_text: hsla(0.0, 0.0, 0.1, 1.0),
            ui_text_muted: hsla(0.0, 0.0, 0.5, 1.0),
        }
    }

    pub fn dark() -> Self {
        Self {
            canvas_background: hsla(0.0, 0.0, 0.1, 1.0),
            selection: hsla(0.58, 0.9, 0.5, 1.0),
            hover: hsla(0.58, 0.9, 0.5, 0.3),
            default_stroke: gpui::white(),
            grid: hsla(0.0, 0.0, 0.2, 1.0),
            ui_background: hsla(0.0, 0.0, 0.12, 1.0),
            ui_border: hsla(0.0, 0.0, 0.2, 1.0),
            ui_text: hsla(0.0, 0.0, 0.9, 1.0),
            ui_text_muted: hsla(0.0, 0.0, 0.5, 1.0),
        }
    }
}

/// Color palette for shape fills (inspired by tldraw).
pub struct Palette;

impl Palette {
    pub fn black() -> Hsla {
        gpui::black()
    }

    pub fn gray() -> Hsla {
        hsla(0.0, 0.0, 0.6, 1.0)
    }

    pub fn white() -> Hsla {
        gpui::white()
    }

    pub fn red() -> Hsla {
        hsla(0.0, 0.8, 0.5, 1.0)
    }

    pub fn orange() -> Hsla {
        hsla(0.08, 0.9, 0.55, 1.0)
    }

    pub fn yellow() -> Hsla {
        hsla(0.13, 0.9, 0.55, 1.0)
    }

    pub fn green() -> Hsla {
        hsla(0.35, 0.7, 0.45, 1.0)
    }

    pub fn blue() -> Hsla {
        hsla(0.58, 0.8, 0.5, 1.0)
    }

    pub fn purple() -> Hsla {
        hsla(0.75, 0.6, 0.55, 1.0)
    }

    pub fn pink() -> Hsla {
        hsla(0.9, 0.7, 0.6, 1.0)
    }

    /// Returns all palette colors in order.
    pub fn all() -> [Hsla; 10] {
        [
            Self::black(),
            Self::gray(),
            Self::white(),
            Self::red(),
            Self::orange(),
            Self::yellow(),
            Self::green(),
            Self::blue(),
            Self::purple(),
            Self::pink(),
        ]
    }
}

/// Helper to create Hsla from h, s, l, a values.
pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Hsla {
    Hsla { h, s, l, a }
}
