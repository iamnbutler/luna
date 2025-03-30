use gpui::{hsla, App, Global, Hsla};

/// Represents the available theme variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    Latte,
    Frappe,
    Macchiato,
    Mocha,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        ThemeVariant::Mocha
    }
}

/// Represents our application theme
#[derive(Debug, Clone)]
pub struct Theme {
    pub variant: ThemeVariant,
    pub background_color: Hsla,
    pub canvas_color: Hsla,
    pub foreground: Hsla,
    pub foreground_muted: Hsla,
    pub foreground_disabled: Hsla,
    pub cursor_color: Hsla,
    pub selected: Hsla,
    
    // Accent colors
    pub rosewater: Hsla,
    pub flamingo: Hsla,
    pub pink: Hsla,
    pub mauve: Hsla,
    pub red: Hsla,
    pub maroon: Hsla,
    pub peach: Hsla,
    pub yellow: Hsla,
    pub green: Hsla,
    pub teal: Hsla,
    pub sky: Hsla,
    pub sapphire: Hsla,
    pub blue: Hsla,
    pub lavender: Hsla,
}

impl Theme {
    /// Create a new theme with the specified variant
    pub fn new(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Latte => Theme::latte(),
            ThemeVariant::Frappe => Theme::frappe(),
            ThemeVariant::Macchiato => Theme::macchiato(),
            ThemeVariant::Mocha => Theme::mocha(),
        }
    }

    /// Create a new theme with the default variant (Mocha)
    pub fn default() -> Self {
        Theme::mocha()
    }

    /// Create the Latte (light) variant
    pub fn latte() -> Self {
        Self {
            variant: ThemeVariant::Latte,
            // Base colors
            background_color: hsla(220.0 / 360.0, 0.23, 0.95, 1.0), // Base
            canvas_color: hsla(220.0 / 360.0, 0.22, 0.92, 1.0),     // Mantle
            foreground: hsla(234.0 / 360.0, 0.16, 0.35, 1.0),       // Text
            foreground_muted: hsla(233.0 / 360.0, 0.13, 0.41, 1.0), // Subtext1
            foreground_disabled: hsla(233.0 / 360.0, 0.10, 0.47, 1.0), // Subtext0
            cursor_color: hsla(220.0 / 360.0, 0.91, 0.54, 1.0),     // Blue
            selected: hsla(231.0 / 360.0, 0.97, 0.72, 1.0),         // Lavender
            
            // Accent colors
            rosewater: hsla(11.0 / 360.0, 0.59, 0.67, 1.0),
            flamingo: hsla(0.0 / 360.0, 0.60, 0.67, 1.0),
            pink: hsla(316.0 / 360.0, 0.73, 0.69, 1.0),
            mauve: hsla(266.0 / 360.0, 0.85, 0.58, 1.0),
            red: hsla(347.0 / 360.0, 0.87, 0.44, 1.0),
            maroon: hsla(355.0 / 360.0, 0.76, 0.59, 1.0),
            peach: hsla(22.0 / 360.0, 0.99, 0.52, 1.0),
            yellow: hsla(35.0 / 360.0, 0.77, 0.49, 1.0),
            green: hsla(109.0 / 360.0, 0.58, 0.40, 1.0),
            teal: hsla(183.0 / 360.0, 0.74, 0.35, 1.0),
            sky: hsla(197.0 / 360.0, 0.97, 0.46, 1.0),
            sapphire: hsla(189.0 / 360.0, 0.70, 0.42, 1.0),
            blue: hsla(220.0 / 360.0, 0.91, 0.54, 1.0),
            lavender: hsla(231.0 / 360.0, 0.97, 0.72, 1.0),
        }
    }

    /// Create the Frappé variant
    pub fn frappe() -> Self {
        Self {
            variant: ThemeVariant::Frappe,
            // Base colors
            background_color: hsla(229.0 / 360.0, 0.19, 0.23, 1.0), // Base
            canvas_color: hsla(231.0 / 360.0, 0.19, 0.20, 1.0),     // Mantle
            foreground: hsla(227.0 / 360.0, 0.70, 0.87, 1.0),       // Text
            foreground_muted: hsla(227.0 / 360.0, 0.44, 0.80, 1.0), // Subtext1
            foreground_disabled: hsla(228.0 / 360.0, 0.29, 0.73, 1.0), // Subtext0
            cursor_color: hsla(222.0 / 360.0, 0.74, 0.74, 1.0),     // Blue
            selected: hsla(239.0 / 360.0, 0.66, 0.84, 1.0),         // Lavender
            
            // Accent colors
            rosewater: hsla(10.0 / 360.0, 0.57, 0.88, 1.0),
            flamingo: hsla(0.0 / 360.0, 0.59, 0.84, 1.0),
            pink: hsla(316.0 / 360.0, 0.73, 0.84, 1.0),
            mauve: hsla(277.0 / 360.0, 0.59, 0.76, 1.0),
            red: hsla(359.0 / 360.0, 0.68, 0.71, 1.0),
            maroon: hsla(358.0 / 360.0, 0.66, 0.76, 1.0),
            peach: hsla(20.0 / 360.0, 0.79, 0.70, 1.0),
            yellow: hsla(40.0 / 360.0, 0.62, 0.73, 1.0),
            green: hsla(96.0 / 360.0, 0.44, 0.68, 1.0),
            teal: hsla(172.0 / 360.0, 0.39, 0.65, 1.0),
            sky: hsla(189.0 / 360.0, 0.48, 0.73, 1.0),
            sapphire: hsla(199.0 / 360.0, 0.55, 0.69, 1.0),
            blue: hsla(222.0 / 360.0, 0.74, 0.74, 1.0),
            lavender: hsla(239.0 / 360.0, 0.66, 0.84, 1.0),
        }
    }

    /// Create the Macchiato variant
    pub fn macchiato() -> Self {
        Self {
            variant: ThemeVariant::Macchiato,
            // Base colors
            background_color: hsla(232.0 / 360.0, 0.23, 0.18, 1.0), // Base
            canvas_color: hsla(233.0 / 360.0, 0.23, 0.15, 1.0),     // Mantle
            foreground: hsla(227.0 / 360.0, 0.68, 0.88, 1.0),       // Text
            foreground_muted: hsla(228.0 / 360.0, 0.39, 0.80, 1.0), // Subtext1
            foreground_disabled: hsla(227.0 / 360.0, 0.27, 0.72, 1.0), // Subtext0
            cursor_color: hsla(220.0 / 360.0, 0.83, 0.75, 1.0),     // Blue
            selected: hsla(234.0 / 360.0, 0.82, 0.85, 1.0),         // Lavender
            
            // Accent colors
            rosewater: hsla(10.0 / 360.0, 0.58, 0.90, 1.0),
            flamingo: hsla(0.0 / 360.0, 0.58, 0.86, 1.0),
            pink: hsla(316.0 / 360.0, 0.74, 0.85, 1.0),
            mauve: hsla(267.0 / 360.0, 0.83, 0.80, 1.0),
            red: hsla(351.0 / 360.0, 0.74, 0.73, 1.0),
            maroon: hsla(355.0 / 360.0, 0.71, 0.77, 1.0),
            peach: hsla(21.0 / 360.0, 0.86, 0.73, 1.0),
            yellow: hsla(40.0 / 360.0, 0.70, 0.78, 1.0),
            green: hsla(105.0 / 360.0, 0.48, 0.72, 1.0),
            teal: hsla(171.0 / 360.0, 0.47, 0.69, 1.0),
            sky: hsla(189.0 / 360.0, 0.59, 0.73, 1.0),
            sapphire: hsla(199.0 / 360.0, 0.66, 0.69, 1.0),
            blue: hsla(220.0 / 360.0, 0.83, 0.75, 1.0),
            lavender: hsla(234.0 / 360.0, 0.82, 0.85, 1.0),
        }
    }

    /// Create the Mocha variant
    pub fn mocha() -> Self {
        Self {
            variant: ThemeVariant::Mocha,
            // Base colors
            background_color: hsla(240.0 / 360.0, 0.21, 0.15, 1.0), // Base
            canvas_color: hsla(240.0 / 360.0, 0.21, 0.12, 1.0),     // Mantle
            foreground: hsla(226.0 / 360.0, 0.64, 0.88, 1.0),       // Text
            foreground_muted: hsla(227.0 / 360.0, 0.35, 0.80, 1.0), // Subtext1
            foreground_disabled: hsla(228.0 / 360.0, 0.24, 0.72, 1.0), // Subtext0
            cursor_color: hsla(217.0 / 360.0, 0.92, 0.76, 1.0),     // Blue
            selected: hsla(232.0 / 360.0, 0.97, 0.85, 1.0),         // Lavender
            
            // Accent colors
            rosewater: hsla(10.0 / 360.0, 0.56, 0.91, 1.0),
            flamingo: hsla(0.0 / 360.0, 0.59, 0.88, 1.0),
            pink: hsla(316.0 / 360.0, 0.72, 0.86, 1.0),
            mauve: hsla(267.0 / 360.0, 0.84, 0.81, 1.0),
            red: hsla(343.0 / 360.0, 0.81, 0.75, 1.0),
            maroon: hsla(350.0 / 360.0, 0.65, 0.77, 1.0),
            peach: hsla(23.0 / 360.0, 0.92, 0.75, 1.0),
            yellow: hsla(41.0 / 360.0, 0.86, 0.83, 1.0),
            green: hsla(115.0 / 360.0, 0.54, 0.76, 1.0),
            teal: hsla(170.0 / 360.0, 0.57, 0.73, 1.0),
            sky: hsla(189.0 / 360.0, 0.71, 0.73, 1.0),
            sapphire: hsla(199.0 / 360.0, 0.76, 0.69, 1.0),
            blue: hsla(217.0 / 360.0, 0.92, 0.76, 1.0),
            lavender: hsla(232.0 / 360.0, 0.97, 0.85, 1.0),
        }
    }

    /// Get the global theme instance
    pub fn get_global(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }

    /// Get a theme with a specific variant
    pub fn with_variant(variant: ThemeVariant) -> Self {
        Theme::new(variant)
    }
}

impl Global for Theme {}

impl Default for Theme {
    fn default() -> Self {
        Theme::default()
    }
}