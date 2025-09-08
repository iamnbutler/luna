//! # Keyboard mapping utilities
//!
//! This module provides utilities for managing keyboard shortcuts and bindings
//! in the Luna application. Rather than defining specific bindings, it provides
//! helper types and functions that can be used by the main application to set up
//! its keyboard handling.

use gpui::Modifiers;

/// Represents a keyboard shortcut configuration
#[derive(Debug, Clone)]
pub struct KeyMap {
    pub key: &'static str,
    pub modifiers: Option<Modifiers>,
    pub description: &'static str,
}

impl KeyMap {
    /// Create a new keymap entry
    pub fn new(key: &'static str, description: &'static str) -> Self {
        Self {
            key,
            modifiers: None,
            description,
        }
    }

    /// Create a new keymap entry with modifiers
    pub fn with_modifiers(
        key: &'static str,
        modifiers: Modifiers,
        description: &'static str,
    ) -> Self {
        Self {
            key,
            modifiers: Some(modifiers),
            description,
        }
    }
}

/// Standard keyboard shortcuts used in design applications
pub struct StandardKeymaps;

impl StandardKeymaps {
    /// Tool selection keymaps
    pub fn tools() -> Vec<KeyMap> {
        vec![
            KeyMap::new("h", "Hand tool"),
            KeyMap::new("a", "Selection tool"),
            KeyMap::new("r", "Rectangle tool"),
            KeyMap::new("f", "Frame tool"),
        ]
    }

    /// Editing operation keymaps
    pub fn editing() -> Vec<KeyMap> {
        vec![
            KeyMap::with_modifiers("c", Modifiers::command(), "Copy"),
            KeyMap::with_modifiers("x", Modifiers::command(), "Cut"),
            KeyMap::with_modifiers("v", Modifiers::command(), "Paste"),
            KeyMap::with_modifiers("a", Modifiers::command(), "Select all"),
            KeyMap::new("delete", "Delete selected"),
            KeyMap::new("escape", "Cancel operation"),
        ]
    }

    /// UI control keymaps
    pub fn ui() -> Vec<KeyMap> {
        vec![
            KeyMap::with_modifiers("\\", Modifiers::command(), "Toggle UI visibility"),
            KeyMap::with_modifiers("q", Modifiers::command(), "Quit application"),
        ]
    }

    /// Color control keymaps
    pub fn colors() -> Vec<KeyMap> {
        vec![
            KeyMap::new("x", "Swap foreground/background colors"),
            KeyMap::new("d", "Reset to default colors"),
        ]
    }
}
