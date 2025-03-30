use gpui::{hsla, App, Global, Hsla};

#[derive(Debug, Clone)]
pub struct Theme {
    pub background_color: Hsla,
    pub canvas_color: Hsla,
    pub foreground: Hsla,
    pub foreground_muted: Hsla,
    pub foreground_disabled: Hsla,
    pub cursor_color: Hsla,
    pub selected: Hsla,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            background_color: hsla(222.0 / 360.0, 0.12, 0.2, 1.0),
            canvas_color: hsla(224.0 / 360., 0.12, 0.19, 1.0),
            foreground: hsla(0.0, 1.0, 1.0, 1.0),
            foreground_muted: hsla(0.0, 1.0, 1.0, 0.6),
            foreground_disabled: hsla(0.0, 1.0, 1.0, 0.3),
            cursor_color: hsla(0.0, 1.0, 1.0, 1.0),
            selected: hsla(210.0 / 360.0, 0.92, 0.65, 1.0),
        }
    }
}

impl Theme {
    pub fn get_global(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }
}

impl Global for Theme {}
