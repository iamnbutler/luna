//! # Utility Functions
//!
//! This module provides common utility functions used throughout the Luna application.
//! It contains helper functions for coordinate handling, input processing, and other
//! shared functionality that doesn't warrant its own dedicated module.
//!
//! The utilities fall into several categories:
//! - Pixel coordinate management (rounding, point creation)
//! - Input processing (keystroke parsing and creation)
//! - General-purpose helpers that are shared across multiple components

#![allow(unused, dead_code)]

use gpui::{Keystroke, Modifiers, Pixels, Point};

/// Rounds a floating-point pixel value to the nearest integer pixel
///
/// This function ensures pixel values align with the physical pixel grid,
/// which is essential for crisp rendering on displays. Without proper rounding,
/// elements can appear blurry due to anti-aliasing across pixel boundaries.
pub fn round_to_pixel(value: Pixels) -> Pixels {
    Pixels(value.0.round())
}

/// Creates a Point with both x and y coordinates rounded to integer pixels
///
/// This convenience function applies pixel rounding to both coordinates
/// of a point simultaneously, ensuring elements are positioned on the pixel
/// grid for crisp rendering.
pub fn rounded_point(x: Pixels, y: Pixels) -> Point<Pixels> {
    Point::new(round_to_pixel(x), round_to_pixel(y))
}

/// Parses a string representation of a keyboard shortcut into a GPUI Keystroke
///
/// This function converts human-readable keyboard shortcut notation into GPUI's
/// internal Keystroke representation. It supports standard modifier key names and
/// handles platform-specific terminology.
///
/// The format follows a dash-separated convention:
/// - Modifiers come first (ctrl, alt, shift, cmd, fn)
/// - The target key comes last
/// - Case is ignored for modifiers
///
/// # Examples
///
/// ```
/// use core::util::keystroke_builder;
///
/// // Creates Cmd+S keystroke
/// let save = keystroke_builder("cmd-s");
///
/// // Creates Ctrl+Shift+P keystroke
/// let palette = keystroke_builder("ctrl-shift-p");
///
/// // Creates simple 'a' keystroke (no modifiers)
/// let type_a = keystroke_builder("a");
/// ```
pub fn keystroke_builder(str: &str) -> Keystroke {
    let parts: Vec<&str> = str.split('-').collect();

    let mut modifiers = Modifiers {
        control: false,
        alt: false,
        shift: false,
        platform: false,
        function: false,
    };

    let mut key_char = None;

    // The last part is the key, everything before it is a modifier
    let key = if parts.is_empty() {
        ""
    } else {
        parts[parts.len() - 1]
    };

    for i in 0..parts.len() - 1 {
        match parts[i].to_lowercase().as_str() {
            "ctrl" | "control" => modifiers.control = true,
            "alt" | "option" => modifiers.alt = true,
            "shift" => modifiers.shift = true,
            "cmd" | "meta" | "command" | "platform" => modifiers.platform = true,
            "fn" | "function" => modifiers.function = true,
            _ => (),
        }
    }

    if !modifiers.control
        && !modifiers.alt
        && !modifiers.shift
        && !modifiers.platform
        && !modifiers.function
    {
        key_char = Some(key.to_string());
    }

    Keystroke {
        modifiers,
        key: key.into(),
        key_char,
    }
}
