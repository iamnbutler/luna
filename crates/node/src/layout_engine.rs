//! Pure layout computation engine.
//!
//! Computes positions and sizes for children in a layout frame.
//! Designed for testability - no GPUI or external dependencies.

use crate::coords::{CanvasPoint, CanvasSize};
use crate::layout::{
    CrossAxisAlignment, FrameLayout, LayoutDirection, MainAxisAlignment, SizingMode,
};
use crate::ShapeId;

/// Input for layout computation.
#[derive(Clone, Debug)]
pub struct LayoutInput {
    /// Shape identifier.
    pub id: ShapeId,
    /// Current size of the shape.
    pub size: CanvasSize,
    /// How width is determined in layout.
    pub width_mode: SizingMode,
    /// How height is determined in layout.
    pub height_mode: SizingMode,
}

/// Output from layout computation.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutOutput {
    /// Shape identifier.
    pub id: ShapeId,
    /// Computed position relative to frame origin.
    pub position: CanvasPoint,
    /// Computed size (may differ from input if Fill mode).
    pub size: CanvasSize,
}

/// Compute layout for children within a frame.
///
/// Returns new positions and sizes for each child, relative to frame origin
/// (accounting for padding).
///
/// # Arguments
/// * `frame_size` - Size of the containing frame
/// * `layout` - Layout configuration (direction, alignment, gap, padding)
/// * `children` - Child shapes with their current sizes and sizing modes
///
/// # Returns
/// Vector of `LayoutOutput` with computed position and size for each child,
/// in the same order as the input.
pub fn compute_layout(
    frame_size: CanvasSize,
    layout: &FrameLayout,
    children: &[LayoutInput],
) -> Vec<LayoutOutput> {
    if children.is_empty() {
        return vec![];
    }

    // Calculate content area (frame minus padding)
    let content_width = (frame_size.width() - layout.padding.horizontal()).max(0.0);
    let content_height = (frame_size.height() - layout.padding.vertical()).max(0.0);

    // Determine main and cross axis sizes based on direction
    let (main_size, cross_size) = match layout.direction {
        LayoutDirection::Row => (content_width, content_height),
        LayoutDirection::Column => (content_height, content_width),
    };

    // Calculate sizes first (handle Fill children)
    let child_sizes = compute_child_sizes(children, layout, main_size, cross_size);

    // Calculate positions
    compute_positions(layout, &child_sizes, main_size, cross_size, children)
}

/// Compute final sizes for all children, handling Fill mode.
fn compute_child_sizes(
    children: &[LayoutInput],
    layout: &FrameLayout,
    main_size: f32,
    cross_size: f32,
) -> Vec<CanvasSize> {
    // Determine which sizing mode applies to which axis based on direction
    let (main_mode_fn, cross_mode_fn): (fn(&LayoutInput) -> SizingMode, fn(&LayoutInput) -> SizingMode) =
        match layout.direction {
            LayoutDirection::Row => (|c| c.width_mode, |c| c.height_mode),
            LayoutDirection::Column => (|c| c.height_mode, |c| c.width_mode),
        };

    let (main_size_fn, cross_size_fn): (fn(&CanvasSize) -> f32, fn(&CanvasSize) -> f32) =
        match layout.direction {
            LayoutDirection::Row => (|s| s.width(), |s| s.height()),
            LayoutDirection::Column => (|s| s.height(), |s| s.width()),
        };

    // Count fill children and calculate fixed space usage
    let mut fill_count = 0;
    let mut fixed_main = 0.0;

    for child in children {
        match main_mode_fn(child) {
            SizingMode::Fill => fill_count += 1,
            SizingMode::Fixed | SizingMode::Hug => {
                fixed_main += main_size_fn(&child.size);
            }
        }
    }

    // Calculate total gap space
    let total_gap = if children.len() > 1 {
        layout.gap * (children.len() - 1) as f32
    } else {
        0.0
    };

    // Calculate space available for fill children
    let available_for_fill = (main_size - fixed_main - total_gap).max(0.0);
    let fill_size = if fill_count > 0 {
        available_for_fill / fill_count as f32
    } else {
        0.0
    };

    // Compute final sizes
    children
        .iter()
        .map(|child| {
            let main = match main_mode_fn(child) {
                SizingMode::Fill => fill_size,
                SizingMode::Fixed | SizingMode::Hug => main_size_fn(&child.size),
            };

            let cross = match cross_mode_fn(child) {
                SizingMode::Fill => cross_size,
                SizingMode::Fixed | SizingMode::Hug => cross_size_fn(&child.size),
            };

            // Handle Stretch on cross axis
            let cross = if layout.cross_axis_alignment == CrossAxisAlignment::Stretch {
                cross_size
            } else {
                cross
            };

            match layout.direction {
                LayoutDirection::Row => CanvasSize::new(main, cross),
                LayoutDirection::Column => CanvasSize::new(cross, main),
            }
        })
        .collect()
}

/// Compute positions for all children based on alignment.
fn compute_positions(
    layout: &FrameLayout,
    child_sizes: &[CanvasSize],
    main_size: f32,
    cross_size: f32,
    children: &[LayoutInput],
) -> Vec<LayoutOutput> {
    let main_size_fn: fn(&CanvasSize) -> f32 = match layout.direction {
        LayoutDirection::Row => |s| s.width(),
        LayoutDirection::Column => |s| s.height(),
    };

    let cross_size_fn: fn(&CanvasSize) -> f32 = match layout.direction {
        LayoutDirection::Row => |s| s.height(),
        LayoutDirection::Column => |s| s.width(),
    };

    // Calculate total children size on main axis
    let total_children: f32 = child_sizes.iter().map(|s| main_size_fn(s)).sum();
    let total_gap = if children.len() > 1 {
        layout.gap * (children.len() - 1) as f32
    } else {
        0.0
    };
    let total = total_children + total_gap;

    // Calculate starting position and gap based on main axis alignment
    // When children overflow (total > main_size), clamp positions to prevent negative values
    let (start_main, effective_gap) = match layout.main_axis_alignment {
        MainAxisAlignment::Start => (0.0, layout.gap),
        MainAxisAlignment::Center => {
            // When overflowing, start at 0 instead of negative
            (((main_size - total) / 2.0).max(0.0), layout.gap)
        }
        MainAxisAlignment::End => {
            // When overflowing, start at 0 instead of negative
            ((main_size - total).max(0.0), layout.gap)
        }
        MainAxisAlignment::SpaceBetween => {
            if children.len() <= 1 {
                (0.0, 0.0)
            } else {
                let space = main_size - total_children;
                // When children overflow, fall back to Start alignment with 0 gap
                if space < 0.0 {
                    (0.0, 0.0)
                } else {
                    (0.0, space / (children.len() - 1) as f32)
                }
            }
        }
    };

    let mut current_main = start_main;

    children
        .iter()
        .zip(child_sizes.iter())
        .map(|(child, size)| {
            let child_main = main_size_fn(size);
            let child_cross = cross_size_fn(size);

            // Cross-axis position based on alignment
            let cross_pos = match layout.cross_axis_alignment {
                CrossAxisAlignment::Start => 0.0,
                CrossAxisAlignment::Center => (cross_size - child_cross) / 2.0,
                CrossAxisAlignment::End => cross_size - child_cross,
                CrossAxisAlignment::Stretch => 0.0, // Size already stretched
            };

            // Create position based on direction
            let position = match layout.direction {
                LayoutDirection::Row => CanvasPoint::new(
                    layout.padding.left + current_main,
                    layout.padding.top + cross_pos,
                ),
                LayoutDirection::Column => CanvasPoint::new(
                    layout.padding.left + cross_pos,
                    layout.padding.top + current_main,
                ),
            };

            current_main += child_main + effective_gap;

            LayoutOutput {
                id: child.id,
                position,
                size: *size,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::Padding;

    // === Test Helpers ===
    // These helpers create LayoutInput with explicit sizing modes.
    // The naming convention makes the sizing mode clear:
    // - fixed_child: both width and height are Fixed (won't stretch)
    // - fill_width_child: width is Fill (stretches horizontally), height is Fixed
    // - fill_height_child: width is Fixed, height is Fill (stretches vertically)

    /// Create a child with Fixed sizing on both axes.
    /// The child will maintain its specified size regardless of layout alignment.
    fn fixed_child(id: u128, width: f32, height: f32) -> LayoutInput {
        LayoutInput {
            id: ShapeId::from_u128(id),
            size: CanvasSize::new(width, height),
            width_mode: SizingMode::Fixed,
            height_mode: SizingMode::Fixed,
        }
    }

    /// Create a child that fills available width (stretches horizontally).
    /// Height remains fixed at the specified value.
    fn fill_width_child(id: u128, width: f32, height: f32) -> LayoutInput {
        LayoutInput {
            id: ShapeId::from_u128(id),
            size: CanvasSize::new(width, height),
            width_mode: SizingMode::Fill,
            height_mode: SizingMode::Fixed,
        }
    }

    /// Create a child that fills available height (stretches vertically).
    /// Width remains fixed at the specified value.
    #[allow(dead_code)]
    fn fill_height_child(id: u128, width: f32, height: f32) -> LayoutInput {
        LayoutInput {
            id: ShapeId::from_u128(id),
            size: CanvasSize::new(width, height),
            width_mode: SizingMode::Fixed,
            height_mode: SizingMode::Fill,
        }
    }

    #[test]
    fn test_row_layout_start() {
        let layout = FrameLayout::default();
        let children = vec![fixed_child(1, 50.0, 30.0), fixed_child(2, 50.0, 30.0)];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].position, CanvasPoint::new(0.0, 0.0));
        assert_eq!(result[1].position, CanvasPoint::new(50.0, 0.0));
    }

    #[test]
    fn test_row_layout_with_gap() {
        let layout = FrameLayout::row().with_gap(10.0);
        let children = vec![fixed_child(1, 50.0, 30.0), fixed_child(2, 50.0, 30.0)];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        assert_eq!(result[0].position, CanvasPoint::new(0.0, 0.0));
        assert_eq!(result[1].position, CanvasPoint::new(60.0, 0.0)); // 50 + 10 gap
    }

    #[test]
    fn test_row_layout_center() {
        let layout = FrameLayout::row().with_main_axis(MainAxisAlignment::Center);
        let children = vec![fixed_child(1, 50.0, 30.0), fixed_child(2, 50.0, 30.0)];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        // Total width = 100, centered in 200 means start at 50
        assert_eq!(result[0].position, CanvasPoint::new(50.0, 0.0));
        assert_eq!(result[1].position, CanvasPoint::new(100.0, 0.0));
    }

    #[test]
    fn test_row_layout_end() {
        let layout = FrameLayout::row().with_main_axis(MainAxisAlignment::End);
        let children = vec![fixed_child(1, 50.0, 30.0), fixed_child(2, 50.0, 30.0)];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        // Total width = 100, end-aligned in 200 means start at 100
        assert_eq!(result[0].position, CanvasPoint::new(100.0, 0.0));
        assert_eq!(result[1].position, CanvasPoint::new(150.0, 0.0));
    }

    #[test]
    fn test_row_layout_space_between() {
        let layout = FrameLayout::row().with_main_axis(MainAxisAlignment::SpaceBetween);
        let children = vec![
            fixed_child(1, 50.0, 30.0),
            fixed_child(2, 50.0, 30.0),
            fixed_child(3, 50.0, 30.0),
        ];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        // Total children = 150, remaining = 50, divided by 2 gaps = 25
        assert_eq!(result[0].position, CanvasPoint::new(0.0, 0.0));
        assert_eq!(result[1].position, CanvasPoint::new(75.0, 0.0)); // 50 + 25
        assert_eq!(result[2].position, CanvasPoint::new(150.0, 0.0)); // 50 + 25 + 50 + 25
    }

    #[test]
    fn test_column_layout() {
        let layout = FrameLayout::column().with_gap(10.0);
        let children = vec![fixed_child(1, 50.0, 30.0), fixed_child(2, 50.0, 40.0)];

        let result = compute_layout(CanvasSize::new(200.0, 200.0), &layout, &children);

        assert_eq!(result[0].position, CanvasPoint::new(0.0, 0.0));
        assert_eq!(result[1].position, CanvasPoint::new(0.0, 40.0)); // 30 + 10 gap
    }

    #[test]
    fn test_cross_axis_center() {
        let layout = FrameLayout::row().with_cross_axis(CrossAxisAlignment::Center);
        let children = vec![fixed_child(1, 50.0, 30.0)];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        // Height 30 centered in 100 means y = 35
        assert_eq!(result[0].position, CanvasPoint::new(0.0, 35.0));
    }

    #[test]
    fn test_cross_axis_end() {
        let layout = FrameLayout::row().with_cross_axis(CrossAxisAlignment::End);
        let children = vec![fixed_child(1, 50.0, 30.0)];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        // Height 30 at end of 100 means y = 70
        assert_eq!(result[0].position, CanvasPoint::new(0.0, 70.0));
    }

    #[test]
    fn test_cross_axis_stretch() {
        let layout = FrameLayout::row().with_cross_axis(CrossAxisAlignment::Stretch);
        let children = vec![fixed_child(1, 50.0, 30.0)];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        // Child should be stretched to fill cross axis
        assert_eq!(result[0].position, CanvasPoint::new(0.0, 0.0));
        assert_eq!(result[0].size, CanvasSize::new(50.0, 100.0));
    }

    #[test]
    fn test_with_padding() {
        let layout = FrameLayout {
            padding: Padding::all(20.0),
            ..Default::default()
        };
        let children = vec![fixed_child(1, 50.0, 30.0)];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        // Position should be offset by padding
        assert_eq!(result[0].position, CanvasPoint::new(20.0, 20.0));
    }

    #[test]
    fn test_fill_children() {
        let layout = FrameLayout::row().with_gap(10.0);
        let children = vec![
            fixed_child(1, 50.0, 30.0),       // Fixed 50
            fill_width_child(2, 50.0, 30.0),  // Fill
            fixed_child(3, 50.0, 30.0),       // Fixed 50
        ];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        // Fixed = 100, gaps = 20, remaining for fill = 80
        assert_eq!(result[0].size, CanvasSize::new(50.0, 30.0));
        assert_eq!(result[1].size, CanvasSize::new(80.0, 30.0));
        assert_eq!(result[2].size, CanvasSize::new(50.0, 30.0));

        // Positions
        assert_eq!(result[0].position, CanvasPoint::new(0.0, 0.0));
        assert_eq!(result[1].position, CanvasPoint::new(60.0, 0.0)); // 50 + 10
        assert_eq!(result[2].position, CanvasPoint::new(150.0, 0.0)); // 60 + 80 + 10
    }

    #[test]
    fn test_multiple_fill_children() {
        let layout = FrameLayout::row();
        let children = vec![
            fill_width_child(1, 0.0, 30.0),
            fill_width_child(2, 0.0, 30.0),
        ];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        // Each fill child gets half
        assert_eq!(result[0].size, CanvasSize::new(100.0, 30.0));
        assert_eq!(result[1].size, CanvasSize::new(100.0, 30.0));
    }

    #[test]
    fn test_empty_children() {
        let layout = FrameLayout::default();
        let children: Vec<LayoutInput> = vec![];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        assert!(result.is_empty());
    }

    #[test]
    fn test_single_child_space_between() {
        let layout = FrameLayout::row().with_main_axis(MainAxisAlignment::SpaceBetween);
        let children = vec![fixed_child(1, 50.0, 30.0)];

        let result = compute_layout(CanvasSize::new(200.0, 100.0), &layout, &children);

        // Single child should be at start
        assert_eq!(result[0].position, CanvasPoint::new(0.0, 0.0));
    }

    #[test]
    fn test_overflow_space_between() {
        // When children are larger than frame, should not have negative positions
        let layout = FrameLayout::column().with_main_axis(MainAxisAlignment::SpaceBetween);
        let children = vec![
            fixed_child(1, 50.0, 300.0),  // 300 tall
            fixed_child(2, 50.0, 300.0),  // 300 tall - total 600, frame only 100
        ];

        let result = compute_layout(CanvasSize::new(100.0, 100.0), &layout, &children);

        // Children overflow, should fall back to start alignment with 0 gap
        assert!(result[0].position.y() >= 0.0, "First child should not have negative Y");
        assert!(result[1].position.y() >= 0.0, "Second child should not have negative Y");
        assert_eq!(result[0].position, CanvasPoint::new(0.0, 0.0));
        assert_eq!(result[1].position, CanvasPoint::new(0.0, 300.0)); // Stacked, no gap
    }

    #[test]
    fn test_overflow_center() {
        // When children are larger than frame, should clamp to 0
        let layout = FrameLayout::row().with_main_axis(MainAxisAlignment::Center);
        let children = vec![fixed_child(1, 300.0, 30.0)]; // 300 wide, frame only 100

        let result = compute_layout(CanvasSize::new(100.0, 100.0), &layout, &children);

        // Should clamp to 0, not negative
        assert_eq!(result[0].position, CanvasPoint::new(0.0, 0.0));
    }

    #[test]
    fn test_overflow_end() {
        // When children are larger than frame, should clamp to 0
        let layout = FrameLayout::row().with_main_axis(MainAxisAlignment::End);
        let children = vec![fixed_child(1, 300.0, 30.0)]; // 300 wide, frame only 100

        let result = compute_layout(CanvasSize::new(100.0, 100.0), &layout, &children);

        // Should clamp to 0, not negative
        assert_eq!(result[0].position, CanvasPoint::new(0.0, 0.0));
    }
}
