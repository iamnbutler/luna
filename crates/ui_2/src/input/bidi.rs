//! Bidirectional text support.

use unicode_bidi::{bidi_class, BidiClass};

/// Text direction for bidirectional text support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextDirection {
    /// Left-to-right text direction (default for Latin, Greek, Cyrillic, etc.)
    #[default]
    Ltr,
    /// Right-to-left text direction (for Arabic, Hebrew, etc.)
    Rtl,
}

impl TextDirection {
    /// Returns true if this is left-to-right direction.
    pub fn is_ltr(self) -> bool {
        matches!(self, TextDirection::Ltr)
    }

    /// Returns true if this is right-to-left direction.
    pub fn is_rtl(self) -> bool {
        matches!(self, TextDirection::Rtl)
    }
}

/// Detects the base direction of text using the first strong directional character.
pub fn detect_base_direction(text: &str) -> TextDirection {
    for c in text.chars() {
        match bidi_class(c) {
            BidiClass::L => return TextDirection::Ltr,
            BidiClass::R | BidiClass::AL => return TextDirection::Rtl,
            _ => continue,
        }
    }
    TextDirection::Ltr
}
