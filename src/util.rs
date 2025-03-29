use gpui::{px, Pixels, Point};

// Helper function to round pixel values to whole numbers
pub fn round_to_pixel(value: Pixels) -> Pixels {
    px(value.0.round())
}

// Helper to create a point with rounded pixel values
pub fn rounded_point(x: Pixels, y: Pixels) -> Point<Pixels> {
    Point::new(round_to_pixel(x), round_to_pixel(y))
}
