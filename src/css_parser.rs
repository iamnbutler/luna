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
/// Supports hex colors and rgb() format
fn parse_color(value: &str) -> Option<Hsla> {
    let value = value.trim().to_lowercase();
    
    // Handle hex colors
    if value.starts_with('#') {
        return parse_hex_color(&value);
    }
    
    // Handle rgb()
    if value.starts_with("rgb(") && value.ends_with(')') {
        let rgb = &value[4..value.len()-1];
        let parts: Vec<&str> = rgb.split(',').collect();
        
        if parts.len() >= 3 {
            let r = parts[0].trim().parse::<u8>().ok()? as f32 / 255.0;
            let g = parts[1].trim().parse::<u8>().ok()? as f32 / 255.0;
            let b = parts[2].trim().parse::<u8>().ok()? as f32 / 255.0;
            let a = if parts.len() > 3 {
                parts[3].trim().parse::<f32>().unwrap_or(1.0)
            } else {
                1.0
            };
            
            return Some(Hsla {
                h: 0.0, // We're not accurately converting RGB to HSL
                s: 0.0, // This is a simplification
                l: (r + g + b) / 3.0, // Just using average as lightness
                a,
            });
        }
    }
    
    // Handle named colors
    match value.as_str() {
        "black" => Some(Hsla { h: 0.0, s: 0.0, l: 0.0, a: 1.0 }),
        "white" => Some(Hsla { h: 0.0, s: 0.0, l: 1.0, a: 1.0 }),
        "red" => Some(Hsla { h: 0.0, s: 1.0, l: 0.5, a: 1.0 }),
        "green" => Some(Hsla { h: 0.33, s: 1.0, l: 0.5, a: 1.0 }),
        "blue" => Some(Hsla { h: 0.67, s: 1.0, l: 0.5, a: 1.0 }),
        _ => None,
    }
}

/// Parse a hex color into an Hsla
fn parse_hex_color(hex: &str) -> Option<Hsla> {
    let hex = hex.trim_start_matches('#');
    
    // Handle #RGB format
    if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1], 16).ok()? as f32 / 15.0;
        let g = u8::from_str_radix(&hex[1..2], 16).ok()? as f32 / 15.0;
        let b = u8::from_str_radix(&hex[2..3], 16).ok()? as f32 / 15.0;
        
        return Some(Hsla {
            h: 0.0, // Again, a simplification
            s: 0.0,
            l: (r + g + b) / 3.0,
            a: 1.0,
        });
    }
    
    // Handle #RRGGBB format
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        
        return Some(Hsla {
            h: 0.0, // This would ideally be a proper RGB to HSL conversion
            s: 0.0,
            l: (r + g + b) / 3.0,
            a: 1.0,
        });
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