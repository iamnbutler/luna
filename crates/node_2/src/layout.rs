//! Autolayout types for Luna frames.
//!
//! Provides flexbox-inspired layout for arranging children within frames.
//! Layout is opt-in: frames default to absolute positioning.

use serde::{Deserialize, Serialize};

/// Layout configuration for a frame.
///
/// When a frame has a `FrameLayout`, its children are automatically
/// positioned based on direction, alignment, gap, and padding settings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameLayout {
    /// Direction children are arranged.
    pub direction: LayoutDirection,

    /// Alignment along the main axis (direction of flow).
    pub main_axis_alignment: MainAxisAlignment,

    /// Alignment along the cross axis (perpendicular to flow).
    pub cross_axis_alignment: CrossAxisAlignment,

    /// Gap between children (in canvas units).
    pub gap: f32,

    /// Padding inside the frame.
    pub padding: Padding,
}

impl Default for FrameLayout {
    fn default() -> Self {
        Self {
            direction: LayoutDirection::Row,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            gap: 0.0,
            padding: Padding::default(),
        }
    }
}

impl FrameLayout {
    /// Create a new horizontal (row) layout.
    pub fn row() -> Self {
        Self::default()
    }

    /// Create a new vertical (column) layout.
    pub fn column() -> Self {
        Self {
            direction: LayoutDirection::Column,
            ..Default::default()
        }
    }

    /// Set the gap between children.
    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set uniform padding on all sides.
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = Padding::all(padding);
        self
    }

    /// Set the main axis alignment.
    pub fn with_main_axis(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }

    /// Set the cross axis alignment.
    pub fn with_cross_axis(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }
}

/// Direction children are laid out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutDirection {
    /// Children arranged horizontally (left to right).
    #[default]
    Row,
    /// Children arranged vertically (top to bottom).
    Column,
}

/// Alignment along the main axis (direction of flow).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MainAxisAlignment {
    /// Pack children at the start.
    #[default]
    Start,
    /// Center children.
    Center,
    /// Pack children at the end.
    End,
    /// Distribute space between children.
    SpaceBetween,
}

/// Alignment along the cross axis (perpendicular to flow).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CrossAxisAlignment {
    /// Align children to start of cross axis.
    #[default]
    Start,
    /// Center children on cross axis.
    Center,
    /// Align children to end of cross axis.
    End,
    /// Stretch children to fill cross axis.
    Stretch,
}

/// Padding (inset) for frame content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    /// Create padding with the same value on all sides.
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create padding with horizontal and vertical values.
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create padding with individual values.
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Total horizontal padding (left + right).
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Total vertical padding (top + bottom).
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// How a child determines its size along an axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SizingMode {
    /// Fixed size (use shape's explicit size).
    #[default]
    Fixed,
    /// Fill available space (stretch to fill remaining room).
    Fill,
    /// Hug content (shrink to fit - only meaningful for frames with children).
    Hug,
}

/// Child-specific layout settings.
///
/// Controls how a shape behaves when it's a child of a layout frame.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChildLayout {
    /// How width is determined in a layout.
    pub width_mode: SizingMode,
    /// How height is determined in a layout.
    pub height_mode: SizingMode,
}

impl ChildLayout {
    /// Create child layout with fixed sizing on both axes.
    pub fn fixed() -> Self {
        Self::default()
    }

    /// Create child layout that fills on main axis, fixed on cross axis.
    pub fn fill_main() -> Self {
        Self {
            width_mode: SizingMode::Fill,
            height_mode: SizingMode::Fixed,
        }
    }

    /// Create child layout that fills on both axes.
    pub fn fill() -> Self {
        Self {
            width_mode: SizingMode::Fill,
            height_mode: SizingMode::Fill,
        }
    }

    /// Set width sizing mode.
    pub fn with_width(mut self, mode: SizingMode) -> Self {
        self.width_mode = mode;
        self
    }

    /// Set height sizing mode.
    pub fn with_height(mut self, mode: SizingMode) -> Self {
        self.height_mode = mode;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_layout_defaults() {
        let layout = FrameLayout::default();
        assert_eq!(layout.direction, LayoutDirection::Row);
        assert_eq!(layout.main_axis_alignment, MainAxisAlignment::Start);
        assert_eq!(layout.cross_axis_alignment, CrossAxisAlignment::Start);
        assert_eq!(layout.gap, 0.0);
        assert_eq!(layout.padding, Padding::default());
    }

    #[test]
    fn test_frame_layout_builders() {
        let layout = FrameLayout::column()
            .with_gap(10.0)
            .with_padding(20.0)
            .with_main_axis(MainAxisAlignment::Center)
            .with_cross_axis(CrossAxisAlignment::Stretch);

        assert_eq!(layout.direction, LayoutDirection::Column);
        assert_eq!(layout.gap, 10.0);
        assert_eq!(layout.padding.top, 20.0);
        assert_eq!(layout.main_axis_alignment, MainAxisAlignment::Center);
        assert_eq!(layout.cross_axis_alignment, CrossAxisAlignment::Stretch);
    }

    #[test]
    fn test_padding_helpers() {
        let uniform = Padding::all(10.0);
        assert_eq!(uniform.horizontal(), 20.0);
        assert_eq!(uniform.vertical(), 20.0);

        let symmetric = Padding::symmetric(5.0, 15.0);
        assert_eq!(symmetric.left, 5.0);
        assert_eq!(symmetric.right, 5.0);
        assert_eq!(symmetric.top, 15.0);
        assert_eq!(symmetric.bottom, 15.0);
    }

    #[test]
    fn test_child_layout_defaults() {
        let child = ChildLayout::default();
        assert_eq!(child.width_mode, SizingMode::Fixed);
        assert_eq!(child.height_mode, SizingMode::Fixed);
    }

    #[test]
    fn test_serde_roundtrip() {
        let layout = FrameLayout::column()
            .with_gap(12.0)
            .with_main_axis(MainAxisAlignment::SpaceBetween);

        let json = serde_json::to_string(&layout).unwrap();
        let parsed: FrameLayout = serde_json::from_str(&json).unwrap();

        assert_eq!(layout, parsed);
    }
}
