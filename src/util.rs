#![allow(unused, dead_code)]

use gpui::{Pixels, Point};

/// Round a Pixels value to the nearest pixel
pub fn round_to_pixel(value: Pixels) -> Pixels {
    Pixels(value.0.round())
}

/// Create a Point where both x and y coordinates are rounded to pixels
pub fn rounded_point(x: Pixels, y: Pixels) -> Point<Pixels> {
    Point::new(round_to_pixel(x), round_to_pixel(y))
}
