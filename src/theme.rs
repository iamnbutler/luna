//! # Theming System
//!
//! This module implements a comprehensive theming system based on the Catppuccin color palette.
//! It provides a type-safe and semantically meaningful approach to colors and visual styles
//! throughout the application.
//!
//! ## Architecture
//!
//! The theming system is built around several key components:
//!
//! - **Palette**: Raw color definitions for a specific theme variant
//! - **ThemeTokens**: Semantic mapping of UI elements to specific colors
//! - **Theme**: Main container combining a palette with semantic tokens
//! - **GlobalTheme**: Application-wide theming mechanism using GPUI's global state
//!
//! The system is designed to enable consistent styling across the application while
//! allowing for theme variants and potential future theme customization.

use gpui::{hsla, App, Global, Hsla, SharedString, UpdateGlobal};
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
        ThemeVariant::Mocha
    }
}

/// Catppuccin palette colors for all flavors
#[derive(Debug, Clone)]
pub struct Palette {
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

pub fn latte() -> Palette {
    Palette {
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

pub fn frappe() -> Palette {
    Palette {
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

pub fn macchiato() -> Palette {
    Palette {
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

pub fn mocha() -> Palette {
    Palette {
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

/// Semantic mapping of UI elements to specific colors
///
/// ThemeTokens provides a semantic layer between the raw color palette and 
/// application components. It categorizes colors by their functional role in the UI,
/// allowing components to reference colors by semantic meaning rather than
/// directly using palette colors.
///
/// This abstraction enables:
/// - Consistent styling across the application
/// - Easy theme switching while maintaining semantic relationships
/// - Separation between raw colors and their application-specific usage
/// - Centralized control over the application's visual language
#[derive(Debug, Clone)]
pub struct ThemeTokens {
    // Background category
    /// Panel background
    pub panel: Hsla,
    /// Canvas background
    pub canvas: Hsla,
    /// Main background (base)
    pub background: Hsla,
    /// Secondary panes (crust/mantle)
    pub background_secondary: Hsla,
    /// Surface element (first level)
    pub surface0: Hsla,
    /// Surface element (second level)
    pub surface1: Hsla,
    /// Surface element (third level)
    pub surface2: Hsla,
    /// Overlay element (first level)
    pub overlay0: Hsla,
    /// Overlay element (second level)
    pub overlay1: Hsla,
    /// Overlay element (third level)
    pub overlay2: Hsla,

    // Typography category
    /// Text/body copy
    pub text: Hsla,
    /// Muted text
    pub foreground_muted: Hsla,
    /// Disabled text
    pub foreground_disabled: Hsla,
    /// Sub-headlines, labels
    pub subtext0: Hsla,
    /// Subtext variant
    pub subtext1: Hsla,
    /// Links, URLs (blue)
    pub link: Hsla,
    /// Success messages (green)
    pub success: Hsla,
    /// Warnings (yellow)
    pub warning: Hsla,
    /// Errors (red)
    pub error: Hsla,
    /// Tags, pills (blue)
    pub tag: Hsla,

    // UI elements category
    /// Cursor (rosewater)
    pub cursor: Hsla,
    /// Selection background
    pub selected: Hsla,
    /// Active window border (lavender)
    pub active_border: Hsla,
    /// Inactive window border (overlay0)
    pub inactive_border: Hsla,
    /// Bell border (yellow)
    pub bell_border: Hsla,

    /// Syntax highlighting category

    /// Keywords (mauve)
    pub keyword: Hsla,
    /// Strings (green)
    pub string: Hsla,
    /// Symbols, atoms (red)
    pub symbol: Hsla,
    /// Escape sequences, regex (pink)
    pub escape: Hsla,
    /// Comments (overlay2)
    pub comment: Hsla,
    /// Constants, numbers (peach)
    pub constant: Hsla,
    /// Operators (sky)
    pub operator: Hsla,
    /// Braces, delimiters (overlay2)
    pub delimiter: Hsla,
    /// Methods, functions (blue)
    pub function: Hsla,
    /// Parameters (maroon)
    pub parameter: Hsla,
    /// Builtins (red)
    pub builtin: Hsla,
    /// Classes, interfaces, types (yellow)
    pub type_name: Hsla,
    /// Enum variants (teal)
    pub enum_variant: Hsla,
    /// Property e.g. JSON keys (blue)
    pub property: Hsla,
    /// XML-style attributes (yellow)
    pub attribute: Hsla,
    /// Macros (rosewater)
    pub macro_: Hsla,

    /// Line highlighting category

    /// Line numbers (overlay1)
    pub line_number: Hsla,
    /// Active line number (lavender)
    pub active_line_number: Hsla,
    /// Cursor line background
    pub cursor_line: Hsla,

    /// Diff & Merge category

    /// Diff header (blue)
    pub diff_header: Hsla,
    /// File path markers (pink)
    pub diff_file_path: Hsla,
    /// Hunk header (peach)
    pub diff_hunk_header: Hsla,
    /// Changed text/line
    pub diff_changed: Hsla,
    /// Inserted text/line
    pub diff_inserted: Hsla,
    /// Removed text/line
    pub diff_removed: Hsla,

    /// Debugging category

    /// Breakpoint icon (red)
    pub debug_breakpoint: Hsla,
    /// Breakpoint line during execution (yellow)
    pub debug_line: Hsla,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: SharedString,
    pub palette: Palette,
    pub tokens: ThemeTokens,
}

#[derive(Debug, Clone)]
pub struct ThemeRegistry {
    themes: Vec<Theme>,
    active_theme: Arc<Theme>,
}

/// Application-wide access point for the current theme
///
/// The ActiveTheme trait provides a standardized mechanism for components to access
/// the current theme from within any context that has access to the application state.
/// It serves as the primary interface through which components obtain styling information.
///
/// By implementing this trait for the App type, any component with an App context
/// can use consistent styling without explicitly passing theme references through
/// the component hierarchy.
pub trait ActiveTheme {
    /// Returns a reference to the currently active theme
    ///
    /// This method is the primary entry point for accessing theme-related styling
    /// information throughout the application.
    fn theme(&self) -> &Arc<Theme>;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Arc<Theme> {
        &self.global::<GlobalTheme>().0
    }
}

impl Theme {
    /// Create a new theme with the default variant
    pub fn default() -> Self {
        Self::from_palette("Cappucin Latte", mocha())
    }

    pub fn from_palette(name: &str, palette: Palette) -> Self {
        // Create tokens that map 1:1 with Catppuccin's style guide
        let tokens = ThemeTokens {
            // Background colors
            panel: palette.surface0,
            canvas: palette.surface0,
            background: palette.base,
            background_secondary: palette.mantle,
            surface0: palette.surface0,
            surface1: palette.surface1,
            surface2: palette.surface2,
            overlay0: palette.overlay0,
            overlay1: palette.overlay1,
            overlay2: palette.overlay2,

            // Typography
            text: palette.text,
            foreground_muted: palette.subtext0,
            foreground_disabled: palette.subtext1,
            subtext0: palette.subtext0,
            subtext1: palette.subtext1,
            link: palette.blue,
            success: palette.green,
            warning: palette.yellow,
            error: palette.red,
            tag: palette.blue,

            // UI elements
            cursor: palette.rosewater,
            selected: palette.overlay2.alpha(0.3),
            active_border: palette.lavender,
            inactive_border: palette.surface0,
            bell_border: palette.yellow,

            // Syntax highlighting
            keyword: palette.mauve,
            string: palette.green,
            symbol: palette.red,
            escape: palette.pink,
            comment: palette.overlay2,
            constant: palette.peach,
            operator: palette.sky,
            delimiter: palette.overlay2,
            function: palette.blue,
            parameter: palette.maroon,
            builtin: palette.red,
            type_name: palette.yellow,
            enum_variant: palette.teal,
            property: palette.blue,
            attribute: palette.yellow,
            macro_: palette.rosewater,

            // Line highlighting
            line_number: palette.overlay1,
            active_line_number: palette.lavender,
            cursor_line: palette.text.alpha(0.1), // Cursor line at 10% opacity

            // Diff & Merge
            diff_header: palette.blue,
            diff_file_path: palette.pink,
            diff_hunk_header: palette.peach,
            diff_changed: palette.blue.alpha(0.2),
            diff_inserted: palette.green.alpha(0.2),
            diff_removed: palette.red.alpha(0.2),

            // Debugging
            debug_breakpoint: palette.red,
            debug_line: palette.yellow.alpha(0.15),
        };

        Theme {
            name: SharedString::new(name),
            palette,
            tokens,
        }
    }

    /// Get the global theme instance
    pub fn get_global(cx: &App) -> &Arc<Theme> {
        &cx.global::<GlobalTheme>().0
    }
}

/// Global container for the application-wide theme instance
///
/// GlobalTheme implements GPUI's Global trait to provide application-wide
/// access to the current theme. It wraps an Arc<Theme> to enable efficient
/// sharing of theme data across components without unnecessary cloning.
///
/// The dereferencing implementations allow convenient access to the underlying
/// Theme while maintaining the reference-counting semantics needed for
/// efficient theme sharing.
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
