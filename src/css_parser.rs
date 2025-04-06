use crate::node::{NodeCommon, NodeFactory, RectangleNode};
use gpui::Hsla;
use std::collections::HashMap;

/// Parses a CSS string and creates a RectangleNode with the properties defined in the CSS.
///
/// # Arguments
///
/// * `css` - A string containing CSS declarations
/// * `factory` - A NodeFactory to generate a unique ID for the new node
///
/// # Returns
///
/// * `Some(RectangleNode)` if parsing was successful
/// * `None` if there was a critical error during parsing
///
/// # Example
///
/// ```
/// let css = r#"
///     width: 100px;
///     height: 50px;
///     left: 10px;
///     top: 20px;
///     background-color: #ff0000;
///     border-color: #000000;
///     border-width: 2px;
///     border-radius: 5px;
/// "#;
///
/// let mut factory = NodeFactory::default();
/// let rect = parse_rectangle_from_css(css, &mut factory).unwrap();
/// ```
pub fn parse_rectangle_from_css(css: &str, factory: &mut NodeFactory) -> Option<RectangleNode> {
    let mut rect = RectangleNode::new(factory.next_id());
    let properties = parse_css_declarations(css);
    
    // Apply properties to the rectangle
    if let Some(width) = properties.get("width").and_then(|v| parse_length(v)) {
        rect.layout_mut().width = width;
    }
    
    if let Some(height) = properties.get("height").and_then(|v| parse_length(v)) {
        rect.layout_mut().height = height;
    }
    
    if let Some(x) = properties.get("left").and_then(|v| parse_length(v)) {
        rect.layout_mut().x = x;
    }
    
    if let Some(y) = properties.get("top").and_then(|v| parse_length(v)) {
        rect.layout_mut().y = y;
    }
    
    if let Some(color) = properties.get("background-color").and_then(|v| parse_color(v)) {
        rect.set_fill(Some(color));
    }
    
    if let Some(color) = properties.get("border-color").and_then(|v| parse_color(v)) {
        rect.set_border(Some(color), rect.border_width());
    }
    
    if let Some(width) = properties.get("border-width").and_then(|v| parse_length(v)) {
        rect.set_border(rect.border_color(), width);
    }
    
    if let Some(radius) = properties.get("border-radius").and_then(|v| parse_length(v)) {
        rect.set_corner_radius(radius);
    }
    
    Some(rect)
}

/// Parse CSS declarations into a map of property names to values
fn parse_css_declarations(css: &str) -> HashMap<String, String> {
    let mut properties = HashMap::new();
    
    for line in css.lines() {
        let line = line.trim();
        if line.is_empty() || !line.contains(':') {
            continue;
        }
        
        if let Some((property, value)) = line.split_once(':') {
            let property = property.trim().to_lowercase();
            // Remove any trailing semicolons
            let value = value.trim().trim_end_matches(';').trim().to_string();
            properties.insert(property, value);
        }
    }
    
    properties
}

/// Parse a CSS length value into a float
///
/// Handles units like 'px' and unitless numbers
fn parse_length(value: &str) -> Option<f32> {
    let value = value.trim();
    
    // Handle pixel units (most common case)
    if value.ends_with("px") {
        value[..value.len() - 2].parse::<f32>().ok()
    } 
    // Handle percentage (convert to 0-1 range)
    else if value.ends_with('%') {
        value[..value.len() - 1].parse::<f32>().ok().map(|v| v / 100.0)
    }
    // Handle unitless values
    else {
        value.parse::<f32>().ok()
    }
}

/// Parse a CSS color value into an Hsla
///
/// Supports:
/// - Hex colors (#RGB, #RRGGBB)
/// - RGB/RGBA format (rgb(r,g,b), rgba(r,g,b,a))
/// - HSLA format (hsla(h,s%,l%,a))
/// - Named colors (red, green, blue, transparent, etc.)
fn parse_color(value: &str) -> Option<Hsla> {
    crate::color::parse_color(value)
}

/// Parse RGB or RGBA components from a string
fn parse_rgba_components(components: &str) -> Option<Hsla> {
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
        
        // Convert RGB to HSL (more accurate conversion)
        let (h, s, l) = rgb_to_hsl(r, g, b);
        
        return Some(Hsla { h, s, l, a });
    }
    
    None
}

/// Parse a single RGB component which can be a number (0-255) or percentage
fn parse_rgb_component(value: &str) -> Option<f32> {
    let value = value.trim();
    
    if value.ends_with('%') {
        // Handle percentage value
        value[..value.len() - 1].parse::<f32>().ok().map(|v| v / 100.0)
    } else {
        // Handle numeric value (0-255)
        value.parse::<u8>().ok().map(|v| v as f32 / 255.0)
    }
}

/// Convert RGB to HSL colorspace
/// 
/// Returns (hue, saturation, lightness) tuple with values in the range [0, 1]
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;
    
    // Calculate lightness
    let l = (max + min) / 2.0;
    
    // Calculate saturation
    let s = if delta.abs() < f32::EPSILON {
        0.0 // Achromatic (gray)
    } else {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    };
    
    // Calculate hue
    let h = if delta.abs() < f32::EPSILON {
        0.0 // Achromatic (gray)
    } else if max == r {
        let segment = (g - b) / delta;
        let shift = if segment < 0.0 { 6.0 } else { 0.0 };
        (segment + shift) / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };
    
    (h, s, l)
}

/// Parse a hex color into an Hsla
fn parse_hex_color(hex: &str) -> Option<Hsla> {
    let hex = hex.trim_start_matches('#');
    
    // Handle #RGB format
    if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1], 16).ok()? as f32 / 15.0;
        let g = u8::from_str_radix(&hex[1..2], 16).ok()? as f32 / 15.0;
        let b = u8::from_str_radix(&hex[2..3], 16).ok()? as f32 / 15.0;
        
        let (h, s, l) = rgb_to_hsl(r, g, b);
        return Some(Hsla { h, s, l, a: 1.0 });
    }
    
    // Handle #RRGGBB format
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        
        let (h, s, l) = rgb_to_hsl(r, g, b);
        return Some(Hsla { h, s, l, a: 1.0 });
    }
    
    // Handle #RRGGBBAA format
    if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()? as f32 / 255.0;
        
        let (h, s, l) = rgb_to_hsl(r, g, b);
        return Some(Hsla { h, s, l, a });
    }
    
    None
}

/// Parse a CSS file and extract multiple rectangle nodes
///
/// Each CSS rule with a selector will create a separate RectangleNode
pub fn parse_rectangles_from_css_file(css: &str, factory: &mut NodeFactory) -> Vec<RectangleNode> {
    let mut result = Vec::new();
    
    // Simple parsing - split by rule blocks
    let mut in_block = false;
    let mut current_block = String::new();
    
    for line in css.lines() {
        let line = line.trim();
        
        if line.contains('{') {
            in_block = true;
            current_block.clear();
            continue;
        }
        
        if line.contains('}') {
            in_block = false;
            if !current_block.is_empty() {
                if let Some(rect) = parse_rectangle_from_css(&current_block, factory) {
                    result.push(rect);
                }
            }
            continue;
        }
        
        if in_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rectangle() {
        let css = r#"
            width: 100px;
            height: 50px;
            left: 10px;
            top: 20px;
            background-color: #ff0000;
            border-color: #000000;
            border-width: 2px;
            border-radius: 5px;
        "#;

        let mut factory = NodeFactory::default();
        let rect = parse_rectangle_from_css(css, &mut factory).unwrap();

        assert_eq!(rect.layout().width, 100.0);
        assert_eq!(rect.layout().height, 50.0);
        assert_eq!(rect.layout().x, 10.0);
        assert_eq!(rect.layout().y, 20.0);
        assert_eq!(rect.border_width(), 2.0);
        assert_eq!(rect.corner_radius(), 5.0);
    }
    
    #[test]
    fn test_parse_multiple_rectangles() {
        let css = r#"
        .rect1 {
            width: 100px;
            height: 50px;
            background-color: #ff0000;
        }
        
        .rect2 {
            width: 200px;
            height: 150px;
            background-color: #00ff00;
        }
        "#;
        
        let mut factory = NodeFactory::default();
        let rects = parse_rectangles_from_css_file(css, &mut factory);
        
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].layout().width, 100.0);
        assert_eq!(rects[0].layout().height, 50.0);
        assert_eq!(rects[1].layout().width, 200.0);
        assert_eq!(rects[1].layout().height, 150.0);
    }
}