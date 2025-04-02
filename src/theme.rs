//! # Theming System
//!
//! This module implements a comprehensive theming system based on the Atom One themes.
//! It provides a type-safe and semantically meaningful approach to colors and visual styles
//! throughout the application.
//!
//! ## Architecture
//!
//! The theming system is built around several key components:
//!
//! - **Palette**: Raw color definitions for a specific theme variant (One Dark or One Light)
//! - **ThemeTokens**: Semantic mapping of UI elements to specific colors
//! - **Theme**: Main container combining a palette with semantic tokens
//! - **GlobalTheme**: Application-wide theming mechanism using GPUI's global state
//!
//! The system is designed to enable consistent styling across the application while
//! allowing for theme variants and potential future theme customization.

use gpui::{hsla, App, Global, Hsla, SharedString, UpdateGlobal};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Represents the available theme variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    OneDark,
    OneLight,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        ThemeVariant::OneDark
    }
}

/// Atom One palette colors for both themes
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

pub fn one_dark() -> Palette {
    Palette {
        // Accent colors
        rosewater: hsla(0.0 / 360.0, 0.50, 0.70, 1.0),     // Cursor color
        flamingo: hsla(5.0 / 360.0, 0.48, 0.67, 1.0),      // #e06c75 (red variant)
        pink: hsla(300.0 / 360.0, 0.60, 0.70, 1.0),        // #c678dd (purple)
        mauve: hsla(300.0 / 360.0, 0.60, 0.70, 1.0),       // #c678dd (purple)
        red: hsla(5.0 / 360.0, 0.48, 0.67, 1.0),           // #e06c75
        maroon: hsla(5.0 / 360.0, 0.54, 0.60, 1.0),        // Red variant
        peach: hsla(29.0 / 360.0, 0.54, 0.61, 1.0),        // #d19a66 (orange)
        yellow: hsla(39.0 / 360.0, 0.67, 0.69, 1.0),       // #e5c07b
        green: hsla(93.0 / 360.0, 0.48, 0.62, 1.0),        // #98c379
        teal: hsla(180.0 / 360.0, 0.43, 0.55, 1.0),        // #56b6c2 (cyan)
        sky: hsla(180.0 / 360.0, 0.43, 0.55, 1.0),         // #56b6c2 (cyan)
        sapphire: hsla(207.0 / 360.0, 0.82, 0.66, 1.0),    // #61afef (blue)
        blue: hsla(207.0 / 360.0, 0.82, 0.66, 1.0),        // #61afef
        lavender: hsla(207.0 / 360.0, 1.00, 0.70, 1.0),    // #528bff (cursor)

        // Base colors
        text: hsla(220.0 / 360.0, 0.14, 0.75, 1.0),        // #abb2bf
        subtext1: hsla(220.0 / 360.0, 0.10, 0.70, 1.0),    // Lighter text variant
        subtext0: hsla(220.0 / 360.0, 0.08, 0.65, 1.0),    // Lighter text variant
        overlay2: hsla(220.0 / 360.0, 0.10, 0.55, 1.0),    // #5c6370 (comment color)
        overlay1: hsla(220.0 / 360.0, 0.08, 0.50, 1.0),    // Darker comment variant
        overlay0: hsla(220.0 / 360.0, 0.06, 0.45, 1.0),    // Darker comment variant
        surface2: hsla(220.0 / 360.0, 0.10, 0.35, 1.0),    // #3e4451 (selection)
        surface1: hsla(222.0 / 360.0, 0.14, 0.25, 1.0),    // Darker background
        surface0: hsla(222.0 / 360.0, 0.14, 0.22, 1.0),    // Darker background
        base: hsla(220.0 / 360.0, 0.13, 0.18, 1.0),        // #282c34 (background)
        mantle: hsla(222.0 / 360.0, 0.14, 0.16, 1.0),      // #21252b (UI background)
        crust: hsla(222.0 / 360.0, 0.15, 0.14, 1.0),       // Darkest background
    }
}

pub fn one_light() -> Palette {
    Palette {
        // Accent colors
        rosewater: hsla(220.0 / 360.0, 1.00, 0.66, 1.0),   // #526eff (cursor)
        flamingo: hsla(5.0 / 360.0, 0.70, 0.59, 1.0),      // #e45649 (red)
        pink: hsla(322.0 / 360.0, 0.72, 0.40, 1.0),        // #a626a4 (purple)
        mauve: hsla(322.0 / 360.0, 0.72, 0.40, 1.0),       // #a626a4 (purple)
        red: hsla(5.0 / 360.0, 0.70, 0.59, 1.0),           // #e45649
        maroon: hsla(5.0 / 360.0, 0.60, 0.50, 1.0),        // Red variant
        peach: hsla(35.0 / 360.0, 0.99, 0.36, 1.0),        // #986801 (orange)
        yellow: hsla(41.0 / 360.0, 0.99, 0.40, 1.0),       // #c18401
        green: hsla(104.0 / 360.0, 0.46, 0.38, 1.0),       // #50a14f
        teal: hsla(189.0 / 360.0, 0.99, 0.37, 1.0),        // #0184bc (cyan)
        sky: hsla(189.0 / 360.0, 0.99, 0.37, 1.0),         // #0184bc (cyan)
        sapphire: hsla(224.0 / 360.0, 0.88, 0.60, 1.0),    // #4078f2 (blue)
        blue: hsla(224.0 / 360.0, 0.88, 0.60, 1.0),        // #4078f2
        lavender: hsla(220.0 / 360.0, 1.00, 0.66, 1.0),    // #526eff (cursor)

        // Base colors
        text: hsla(230.0 / 360.0, 0.11, 0.25, 1.0),        // #383a42
        subtext1: hsla(230.0 / 360.0, 0.08, 0.30, 1.0),    // Slightly lighter text
        subtext0: hsla(230.0 / 360.0, 0.06, 0.35, 1.0),    // Slightly lighter text
        overlay2: hsla(230.0 / 360.0, 0.05, 0.45, 1.0),    // #a0a1a7 (comments)
        overlay1: hsla(230.0 / 360.0, 0.04, 0.50, 1.0),    // Slightly lighter comments
        overlay0: hsla(230.0 / 360.0, 0.03, 0.55, 1.0),    // Slightly lighter comments
        surface2: hsla(230.0 / 360.0, 0.02, 0.90, 1.0),    // #e5e5e6 (selection)
        surface1: hsla(230.0 / 360.0, 0.01, 0.94, 1.0),    // Slightly darker than background
        surface0: hsla(230.0 / 360.0, 0.01, 0.96, 1.0),    // Slightly darker than background
        base: hsla(0.0 / 360.0, 0.00, 0.98, 1.0),          // #fafafa (background)
        mantle: hsla(0.0 / 360.0, 0.00, 0.94, 1.0),        // #f0f0f0 (UI background)
        crust: hsla(0.0 / 360.0, 0.00, 0.90, 1.0),         // Slightly darker UI background
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
        Self::from_palette("Atom One Dark", one_dark())
    }

    pub fn from_palette(name: &str, palette: Palette) -> Self {
        // Create tokens that map to Atom One theme colors
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
