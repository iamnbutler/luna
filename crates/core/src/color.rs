//! Color parsing and manipulation utilities.
//!
//! This module provides functions for parsing color strings in various formats
//! and converting between color spaces.

use gpui::Hsla;

/// Parse a color string into an HSLA color.
///
/// Supports the following formats:
/// - Hex colors: #RGB, #RGBA, #RRGGBB, #RRGGBBAA (with or without # prefix)
/// - RGB/RGBA: rgb(r, g, b), rgba(r, g, b, a)
/// - HSLA: hsla(h, s%, l%, a)
/// - Named colors: black, white, red, etc.
///
/// # Examples
///
/// ```
/// use luna_core::color::parse_color;
///
/// // Parse a hex color
/// let red = parse_color("#ff0000").unwrap();
///
/// // Parse an RGB color
/// let green = parse_color("rgb(0, 255, 0)").unwrap();
///
/// // Parse an HSLA color
/// let blue = parse_color("hsla(240, 100%, 50%, 1.0)").unwrap();
///
/// // Parse a named color
/// let black = parse_color("black").unwrap();
/// ```
pub fn parse_color(value: &str) -> Option<Hsla> {
    let value = value.trim();

    // Handle transparent special case
    if value.eq_ignore_ascii_case("transparent") {
        return Some(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.0,
        });
    }

    // Try using the built-in functionality from gpui
    if let Some(rgba) = parse_hex_color(value) {
        return Some(rgba);
    }

    // Handle RGB/RGBA format
    if value.starts_with("rgb") {
        return parse_rgb_color(value);
    }

    // Handle HSLA format
    if value.starts_with("hsla") {
        return parse_hsla_color(value);
    }

    // Handle named colors
    match value.to_lowercase().as_str() {
        "black" => Some(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 1.0,
        }),
        "white" => Some(Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 1.0,
        }),
        "red" => Some(Hsla {
            h: 0.0,
            s: 1.0,
            l: 0.5,
            a: 1.0,
        }),
        "green" => Some(Hsla {
            h: 0.33,
            s: 1.0,
            l: 0.5,
            a: 1.0,
        }),
        "blue" => Some(Hsla {
            h: 0.67,
            s: 1.0,
            l: 0.5,
            a: 1.0,
        }),
        "yellow" => Some(Hsla {
            h: 0.17,
            s: 1.0,
            l: 0.5,
            a: 1.0,
        }),
        "cyan" => Some(Hsla {
            h: 0.5,
            s: 1.0,
            l: 0.5,
            a: 1.0,
        }),
        "magenta" => Some(Hsla {
            h: 0.83,
            s: 1.0,
            l: 0.5,
            a: 1.0,
        }),
        "gray" | "grey" => Some(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.5,
            a: 1.0,
        }),
        _ => None,
    }
}

/// Parse a hex color into an Hsla
fn parse_hex_color(value: &str) -> Option<Hsla> {
    // Try using built-in TryFrom for Rgba first
    if let Ok(rgba) = gpui::Rgba::try_from(value) {
        return Some(rgba.into());
    }

    // If that fails, try our own hex parsing implementation
    let value = value.trim();

    // Normalize the hex string
    let hex = if value.starts_with('#') {
        &value[1..]
    } else if !value.starts_with("rgb") && !value.starts_with("hsl") {
        value
    } else {
        return None;
    };

    // Parse based on the hex format
    if hex.len() == 3 {
        // Handle #RGB format
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()? as f32 / 255.0;

        let rgba = gpui::Rgba { r, g, b, a: 1.0 };
        return Some(rgba.into());
    } else if hex.len() == 6 {
        // Handle #RRGGBB format
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;

        let rgba = gpui::Rgba { r, g, b, a: 1.0 };
        return Some(rgba.into());
    } else if hex.len() == 8 {
        // Handle #RRGGBBAA format
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;

        let rgba = gpui::Rgba { r, g, b, a };
        return Some(rgba.into());
    }

    None
}

/// Parse RGB/RGBA color format
fn parse_rgb_color(value: &str) -> Option<Hsla> {
    let value = value.trim();

    // Get the components inside the parentheses
    let components = if value.starts_with("rgba(") && value.ends_with(')') {
        value.strip_prefix("rgba(")?.strip_suffix(")")?
    } else if value.starts_with("rgb(") && value.ends_with(')') {
        value.strip_prefix("rgb(")?.strip_suffix(")")?
    } else {
        return None;
    };

    let parts: Vec<&str> = components.split(',').collect();

    if parts.len() >= 3 {
        // Parse RGB components
        let r = parse_rgb_component(parts[0])?;
        let g = parse_rgb_component(parts[1])?;
        let b = parse_rgb_component(parts[2])?;

        // Parse alpha component if present
        let a = if parts.len() > 3 {
            parts[3].trim().parse::<f32>().ok().unwrap_or(1.0)
        } else {
            1.0
        };

        let rgba = gpui::Rgba { r, g, b, a };
        return Some(rgba.into());
    }

    None
}

/// Parse HSLA color format
fn parse_hsla_color(value: &str) -> Option<Hsla> {
    let value = value.trim();

    // Get the components inside the parentheses
    let content = value.strip_prefix("hsla(")?.strip_suffix(")")?;
    let hsla_parts: Vec<&str> = content.split(',').collect();

    // Need exactly 4 parts for hsla
    if hsla_parts.len() == 4 {
        // Try to parse the values
        // h: hue (0-360 degrees)
        // s: saturation (0-100%)
        // l: lightness (0-100%)
        // a: alpha (0-1)
        if let Ok(h) = hsla_parts[0].trim().parse::<f32>() {
            // Parse saturation (remove % sign)
            let s_str = hsla_parts[1].trim();
            let s = if let Some(s_val) = s_str.strip_suffix("%") {
                match s_val.parse::<f32>() {
                    Ok(val) => val / 100.0, // Convert percentage to 0-1 range
                    Err(_) => return None,
                }
            } else {
                match s_str.parse::<f32>() {
                    Ok(val) => val / 100.0, // Assume it's a percentage without %
                    Err(_) => return None,
                }
            };

            // Parse lightness (remove % sign)
            let l_str = hsla_parts[2].trim();
            let l = if let Some(l_val) = l_str.strip_suffix("%") {
                match l_val.parse::<f32>() {
                    Ok(val) => val / 100.0, // Convert percentage to 0-1 range
                    Err(_) => return None,
                }
            } else {
                match l_str.parse::<f32>() {
                    Ok(val) => val / 100.0, // Assume it's a percentage without %
                    Err(_) => return None,
                }
            };

            // Parse alpha
            if let Ok(a) = hsla_parts[3].trim().parse::<f32>() {
                return Some(Hsla {
                    h: (h / 360.0).clamp(0.0, 1.0), // Convert degrees to 0-1 range
                    s: s.clamp(0.0, 1.0),
                    l: l.clamp(0.0, 1.0),
                    a: a.clamp(0.0, 1.0),
                });
            }
        }
    }

    None
}

/// Parse a single RGB component which can be a number (0-255) or percentage
fn parse_rgb_component(value: &str) -> Option<f32> {
    let value = value.trim();

    if value.ends_with('%') {
        // Handle percentage value
        value[..value.len() - 1]
            .parse::<f32>()
            .ok()
            .map(|v| v / 100.0)
    } else {
        // Handle numeric value (0-255)
        value.parse::<u8>().ok().map(|v| v as f32 / 255.0)
    }
}
