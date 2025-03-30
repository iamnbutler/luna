use gpui::{hsla, App, Global, Hsla};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Represents the available theme variants (flavors)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    Latte,
    Frappe,
    Macchiato,
    Mocha,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        ThemeVariant::Latte
    }
}

/// Catppuccin palette colors for all flavors
#[derive(Debug, Clone)]
pub struct ThemePalette {
    // Flavor (variant)
    pub variant: ThemeVariant,

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

    // Base colors
    pub text: Hsla,
    pub subtext1: Hsla,
    pub subtext0: Hsla,
    pub overlay2: Hsla,
    pub overlay1: Hsla,
    pub overlay0: Hsla,
    pub surface2: Hsla,
    pub surface1: Hsla,
    pub surface0: Hsla,
    pub base: Hsla,
    pub mantle: Hsla,
    pub crust: Hsla,
}

impl ThemePalette {
    /// Create a new palette with the specified variant
    pub fn new(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Latte => ThemePalette::latte(),
            ThemeVariant::Frappe => ThemePalette::frappe(),
            ThemeVariant::Macchiato => ThemePalette::macchiato(),
            ThemeVariant::Mocha => ThemePalette::mocha(),
        }
    }

    /// Create the Latte (light) variant
    pub fn latte() -> Self {
        Self {
            variant: ThemeVariant::Latte,

            // Accent colors
            rosewater: hsla(11.0 / 360.0, 0.59, 0.67, 1.0), // #dc8a78
            flamingo: hsla(0.0 / 360.0, 0.60, 0.67, 1.0),   // #dd7878
            pink: hsla(316.0 / 360.0, 0.73, 0.69, 1.0),     // #ea76cb
            mauve: hsla(266.0 / 360.0, 0.85, 0.58, 1.0),    // #8839ef
            red: hsla(347.0 / 360.0, 0.87, 0.44, 1.0),      // #d20f39
            maroon: hsla(355.0 / 360.0, 0.76, 0.59, 1.0),   // #e64553
            peach: hsla(22.0 / 360.0, 0.99, 0.52, 1.0),     // #fe640b
            yellow: hsla(35.0 / 360.0, 0.77, 0.49, 1.0),    // #df8e1d
            green: hsla(109.0 / 360.0, 0.58, 0.40, 1.0),    // #40a02b
            teal: hsla(183.0 / 360.0, 0.74, 0.35, 1.0),     // #179299
            sky: hsla(197.0 / 360.0, 0.97, 0.46, 1.0),      // #04a5e5
            sapphire: hsla(189.0 / 360.0, 0.70, 0.42, 1.0), // #209fb5
            blue: hsla(220.0 / 360.0, 0.91, 0.54, 1.0),     // #1e66f5
            lavender: hsla(231.0 / 360.0, 0.97, 0.72, 1.0), // #7287fd

            // Base colors
            text: hsla(234.0 / 360.0, 0.16, 0.35, 1.0), // #4c4f69
            subtext1: hsla(233.0 / 360.0, 0.13, 0.41, 1.0), // #5c5f77
            subtext0: hsla(233.0 / 360.0, 0.10, 0.47, 1.0), // #6c6f85
            overlay2: hsla(232.0 / 360.0, 0.10, 0.53, 1.0), // #7c7f93
            overlay1: hsla(231.0 / 360.0, 0.10, 0.59, 1.0), // #8c8fa1
            overlay0: hsla(228.0 / 360.0, 0.11, 0.65, 1.0), // #9ca0b0
            surface2: hsla(227.0 / 360.0, 0.12, 0.71, 1.0), // #acb0be
            surface1: hsla(225.0 / 360.0, 0.14, 0.77, 1.0), // #bcc0cc
            surface0: hsla(223.0 / 360.0, 0.16, 0.83, 1.0), // #ccd0da
            base: hsla(220.0 / 360.0, 0.23, 0.95, 1.0), // #eff1f5
            mantle: hsla(220.0 / 360.0, 0.22, 0.92, 1.0), // #e6e9ef
            crust: hsla(220.0 / 360.0, 0.21, 0.89, 1.0), // #dce0e8
        }
    }

    /// Create the Frappé variant
    pub fn frappe() -> Self {
        Self {
            variant: ThemeVariant::Frappe,

            // Accent colors
            rosewater: hsla(10.0 / 360.0, 0.57, 0.88, 1.0), // #f2d5cf
            flamingo: hsla(0.0 / 360.0, 0.59, 0.84, 1.0),   // #eebebe
            pink: hsla(316.0 / 360.0, 0.73, 0.84, 1.0),     // #f4b8e4
            mauve: hsla(277.0 / 360.0, 0.59, 0.76, 1.0),    // #ca9ee6
            red: hsla(359.0 / 360.0, 0.68, 0.71, 1.0),      // #e78284
            maroon: hsla(358.0 / 360.0, 0.66, 0.76, 1.0),   // #ea999c
            peach: hsla(20.0 / 360.0, 0.79, 0.70, 1.0),     // #ef9f76
            yellow: hsla(40.0 / 360.0, 0.62, 0.73, 1.0),    // #e5c890
            green: hsla(96.0 / 360.0, 0.44, 0.68, 1.0),     // #a6d189
            teal: hsla(172.0 / 360.0, 0.39, 0.65, 1.0),     // #81c8be
            sky: hsla(189.0 / 360.0, 0.48, 0.73, 1.0),      // #99d1db
            sapphire: hsla(199.0 / 360.0, 0.55, 0.69, 1.0), // #85c1dc
            blue: hsla(222.0 / 360.0, 0.74, 0.74, 1.0),     // #8caaee
            lavender: hsla(239.0 / 360.0, 0.66, 0.84, 1.0), // #babbf1

            // Base colors
            text: hsla(227.0 / 360.0, 0.70, 0.87, 1.0), // #c6d0f5
            subtext1: hsla(227.0 / 360.0, 0.44, 0.80, 1.0), // #b5bfe2
            subtext0: hsla(228.0 / 360.0, 0.29, 0.73, 1.0), // #a5adce
            overlay2: hsla(228.0 / 360.0, 0.22, 0.66, 1.0), // #949cbb
            overlay1: hsla(227.0 / 360.0, 0.17, 0.58, 1.0), // #838ba7
            overlay0: hsla(229.0 / 360.0, 0.13, 0.52, 1.0), // #737994
            surface2: hsla(228.0 / 360.0, 0.13, 0.44, 1.0), // #626880
            surface1: hsla(227.0 / 360.0, 0.15, 0.37, 1.0), // #51576d
            surface0: hsla(230.0 / 360.0, 0.16, 0.30, 1.0), // #414559
            base: hsla(229.0 / 360.0, 0.19, 0.23, 1.0), // #303446
            mantle: hsla(231.0 / 360.0, 0.19, 0.20, 1.0), // #292c3c
            crust: hsla(229.0 / 360.0, 0.20, 0.17, 1.0), // #232634
        }
    }

    /// Create the Macchiato variant
    pub fn macchiato() -> Self {
        Self {
            variant: ThemeVariant::Macchiato,

            // Accent colors
            rosewater: hsla(10.0 / 360.0, 0.58, 0.90, 1.0), // #f4dbd6
            flamingo: hsla(0.0 / 360.0, 0.58, 0.86, 1.0),   // #f0c6c6
            pink: hsla(316.0 / 360.0, 0.74, 0.85, 1.0),     // #f5bde6
            mauve: hsla(267.0 / 360.0, 0.83, 0.80, 1.0),    // #c6a0f6
            red: hsla(351.0 / 360.0, 0.74, 0.73, 1.0),      // #ed8796
            maroon: hsla(355.0 / 360.0, 0.71, 0.77, 1.0),   // #ee99a0
            peach: hsla(21.0 / 360.0, 0.86, 0.73, 1.0),     // #f5a97f
            yellow: hsla(40.0 / 360.0, 0.70, 0.78, 1.0),    // #eed49f
            green: hsla(105.0 / 360.0, 0.48, 0.72, 1.0),    // #a6da95
            teal: hsla(171.0 / 360.0, 0.47, 0.69, 1.0),     // #8bd5ca
            sky: hsla(189.0 / 360.0, 0.59, 0.73, 1.0),      // #91d7e3
            sapphire: hsla(199.0 / 360.0, 0.66, 0.69, 1.0), // #7dc4e4
            blue: hsla(220.0 / 360.0, 0.83, 0.75, 1.0),     // #8aadf4
            lavender: hsla(234.0 / 360.0, 0.82, 0.85, 1.0), // #b7bdf8

            // Base colors
            text: hsla(227.0 / 360.0, 0.68, 0.88, 1.0), // #cad3f5
            subtext1: hsla(228.0 / 360.0, 0.39, 0.80, 1.0), // #b8c0e0
            subtext0: hsla(227.0 / 360.0, 0.27, 0.72, 1.0), // #a5adcb
            overlay2: hsla(228.0 / 360.0, 0.20, 0.65, 1.0), // #939ab7
            overlay1: hsla(228.0 / 360.0, 0.15, 0.57, 1.0), // #8087a2
            overlay0: hsla(230.0 / 360.0, 0.12, 0.49, 1.0), // #6e738d
            surface2: hsla(230.0 / 360.0, 0.14, 0.41, 1.0), // #5b6078
            surface1: hsla(231.0 / 360.0, 0.16, 0.34, 1.0), // #494d64
            surface0: hsla(230.0 / 360.0, 0.19, 0.26, 1.0), // #363a4f
            base: hsla(232.0 / 360.0, 0.23, 0.18, 1.0), // #24273a
            mantle: hsla(233.0 / 360.0, 0.23, 0.15, 1.0), // #1e2030
            crust: hsla(236.0 / 360.0, 0.23, 0.12, 1.0), // #181926
        }
    }

    /// Create the Mocha variant
    pub fn mocha() -> Self {
        Self {
            variant: ThemeVariant::Mocha,

            // Accent colors
            rosewater: hsla(10.0 / 360.0, 0.56, 0.91, 1.0), // #f5e0dc
            flamingo: hsla(0.0 / 360.0, 0.59, 0.88, 1.0),   // #f2cdcd
            pink: hsla(316.0 / 360.0, 0.72, 0.86, 1.0),     // #f5c2e7
            mauve: hsla(267.0 / 360.0, 0.84, 0.81, 1.0),    // #cba6f7
            red: hsla(343.0 / 360.0, 0.81, 0.75, 1.0),      // #f38ba8
            maroon: hsla(350.0 / 360.0, 0.65, 0.77, 1.0),   // #eba0ac
            peach: hsla(23.0 / 360.0, 0.92, 0.75, 1.0),     // #fab387
            yellow: hsla(41.0 / 360.0, 0.86, 0.83, 1.0),    // #f9e2af
            green: hsla(115.0 / 360.0, 0.54, 0.76, 1.0),    // #a6e3a1
            teal: hsla(170.0 / 360.0, 0.57, 0.73, 1.0),     // #94e2d5
            sky: hsla(189.0 / 360.0, 0.71, 0.73, 1.0),      // #89dceb
            sapphire: hsla(199.0 / 360.0, 0.76, 0.69, 1.0), // #74c7ec
            blue: hsla(217.0 / 360.0, 0.92, 0.76, 1.0),     // #89b4fa
            lavender: hsla(232.0 / 360.0, 0.97, 0.85, 1.0), // #b4befe

            // Base colors
            text: hsla(226.0 / 360.0, 0.64, 0.88, 1.0), // #cdd6f4
            subtext1: hsla(227.0 / 360.0, 0.35, 0.80, 1.0), // #bac2de
            subtext0: hsla(228.0 / 360.0, 0.24, 0.72, 1.0), // #a6adc8
            overlay2: hsla(228.0 / 360.0, 0.17, 0.64, 1.0), // #9399b2
            overlay1: hsla(230.0 / 360.0, 0.13, 0.55, 1.0), // #7f849c
            overlay0: hsla(231.0 / 360.0, 0.11, 0.47, 1.0), // #6c7086
            surface2: hsla(233.0 / 360.0, 0.12, 0.39, 1.0), // #585b70
            surface1: hsla(234.0 / 360.0, 0.13, 0.31, 1.0), // #45475a
            surface0: hsla(237.0 / 360.0, 0.16, 0.23, 1.0), // #313244
            base: hsla(240.0 / 360.0, 0.21, 0.15, 1.0), // #1e1e2e
            mantle: hsla(240.0 / 360.0, 0.21, 0.12, 1.0), // #181825
            crust: hsla(240.0 / 360.0, 0.23, 0.09, 1.0), // #11111b
        }
    }
}

/// Represents semantic tokens that reference the theme palette
#[derive(Debug, Clone)]
pub struct Theme {
    // Underlying palette
    pub palette: ThemePalette,

    // Semantic tokens
    pub background_color: Hsla,
    pub canvas_color: Hsla,
    pub foreground: Hsla,
    pub foreground_muted: Hsla,
    pub foreground_disabled: Hsla,
    pub cursor_color: Hsla,
    pub selected: Hsla,
}

/// Implementing this trait allows accessing the active theme.
pub trait ActiveTheme {
    /// Returns the active theme.
    fn theme(&self) -> &Arc<Theme>;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Arc<Theme> {
        &self.global::<GlobalTheme>().0
    }
}

impl Theme {
    /// Create a new theme with the default variant (Mocha)
    pub fn new() -> Self {
        Self::create_theme(ThemeVariant::Mocha)
    }

    /// Private helper to create a theme with the specified variant
    fn create_theme(variant: ThemeVariant) -> Self {
        let palette = ThemePalette::new(variant);

        Self {
            // Semantic tokens that reference palette colors
            background_color: palette.base,
            canvas_color: palette.mantle,
            foreground: palette.text,
            foreground_muted: palette.subtext1,
            foreground_disabled: palette.subtext0,
            cursor_color: palette.blue,
            selected: palette.lavender,

            // Store the full palette for access to all colors
            palette,
        }
    }

    /// Create a new theme with the default variant (Mocha)
    pub fn default() -> Self {
        Self::create_theme(ThemeVariant::Mocha)
    }

    /// Get the theme variant
    pub fn variant(&self) -> ThemeVariant {
        self.palette.variant
    }

    /// Get the global theme instance
    pub fn get_global(cx: &App) -> &Arc<Theme> {
        &cx.global::<GlobalTheme>().0
    }

    /// Get a theme with a specific variant
    pub fn with_variant(variant: ThemeVariant) -> Self {
        Self::create_theme(variant)
    }
}

#[derive(Clone, Debug)]
pub struct GlobalTheme(pub Arc<Theme>);

impl Deref for GlobalTheme {
    type Target = Arc<Theme>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GlobalTheme {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Global for GlobalTheme {}

impl Default for Theme {
    fn default() -> Self {
        Theme::default()
    }
}
