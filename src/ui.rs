//! # User Interface Components
//!
//! This module serves as the central organization point for Luna's user interface.
//! It provides components for inspector panels, layer management, property editing,
//! and other UI elements that surround the main canvas.
//!
//! ## UI Architecture
//!
//! Luna's UI is organized into several key components:
//! - **Inspector**: Properties panel for viewing and editing element attributes
//! - **Layer List**: Hierarchical view of elements in the document
//! - **Property**: Reusable property editing components
//! - **Sidebar**: Container for various panels and tools
//!
//! The UI system is built on GPUI's component model, with a focus on composability
//! and reactive updates based on application state changes.

#![allow(unused, dead_code)]
use crate::canvas_element::CanvasElement;
use crate::{canvas::LunaCanvas, theme::Theme};
use gpui::{
    actions, div, hsla, point, prelude::*, px, svg, App, Application, AssetSource, BoxShadow,
    ElementId, Entity, FocusHandle, Focusable, Global, Hsla, IntoElement, Keystroke, Menu,
    MenuItem, Modifiers, Pixels, Point, SharedString, TitlebarOptions, UpdateGlobal, WeakEntity,
    Window, WindowBackgroundAppearance, WindowOptions,
};
use std::{fs, path::PathBuf};
use strum::Display;

pub mod inspector;
pub mod layer_list;
mod property;
pub mod sidebar;

const TITLEBAR_HEIGHT: f32 = 31.;

/// SVG icon identifiers for UI elements
///
/// This enum provides a type-safe way to reference SVG icons used throughout
/// the application. It centralizes icon management and provides:
/// - A single point of truth for available icons
/// - Type checking for icon references
/// - An abstraction layer over the actual icon file paths
///
/// When displayed, icons are mapped to their corresponding SVG files
/// through the `src()` method, allowing icon files to be reorganized
/// without changing code that references them.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum Icon {
    /// Undo/history icon
    ArrowCounterClockwise,
    /// Resize/transform icon
    ArrowDownRight,
    /// Frame/artboard icon
    Frame,
    /// Image/media icon
    Image,
    /// Vector path icon
    Path,
    /// Rectangle shape icon 
    Square,
    /// Text tool icon
    Text,
}

impl Icon {
    /// Converts an icon enum variant to its corresponding SVG file path
    ///
    /// This method maps each icon to its actual file location, providing
    /// a layer of indirection that allows icon files to be reorganized
    /// without changing the code that references them.
    ///
    /// # Returns
    ///
    /// A SharedString containing the relative path to the SVG file
    pub fn src(self) -> SharedString {
        match self {
            Icon::ArrowCounterClockwise => "svg/arrow_counter_clockwise.svg".into(),
            Icon::ArrowDownRight => "svg/arrow_down_right.svg".into(),
            Icon::Frame => "svg/frame.svg".into(),
            Icon::Image => "svg/image.svg".into(),
            Icon::Path => "svg/pen_tool.svg".into(),
            Icon::Square => "svg/square.svg".into(),
            Icon::Text => "svg/text_cursor.svg".into(),
        }
    }
}
