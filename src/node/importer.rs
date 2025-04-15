//! # Node Importer
//!
//! Provides functionality to convert SerializedNode structures to internal node types
//! for rendering on the Luna canvas.

use crate::node::schema::{SerializedNode, Style, Color};
use crate::node::{NodeFactory, NodeId, NodeTypography, NodeImage, NodeCommon};
use crate::node::frame::FrameNode;
use crate::node::image::ImageNode;
use crate::node::text::TextNode;
use std::collections::HashMap;
use gpui::Hsla;
use palette::Srgb;
use smallvec::smallvec;

/// Converts color values from schema format to internal representation
fn convert_color(color: &Option<Color>) -> Option<Hsla> {
    if let Some(color) = color {
        let rgb = color.0;
        // Convert RGB to HSL (simplified approach, could be improved)
        Some(Hsla {
            h: 0.0,      // Placeholder - needs proper conversion
            s: 0.0, // Placeholder
            l: rgb.red,  // Simplified - using red as lightness
            a: 1.0,
        })
    } else {
        None
    }
}

/// Maps font weight from schema to internal representation
fn map_font_weight(weight: &Option<crate::node::schema::FontWeight>) -> Option<String> {
    weight.as_ref().map(|w| {
        match w {
            crate::node::schema::FontWeight::Normal => "normal".to_string(),
            crate::node::schema::FontWeight::Bold => "bold".to_string(),
            crate::node::schema::FontWeight::Bolder => "bolder".to_string(),
            crate::node::schema::FontWeight::Lighter => "lighter".to_string(),
            crate::node::schema::FontWeight::W100 => "100".to_string(),
            crate::node::schema::FontWeight::W200 => "200".to_string(),
            crate::node::schema::FontWeight::W300 => "300".to_string(),
            crate::node::schema::FontWeight::W400 => "400".to_string(),
            crate::node::schema::FontWeight::W500 => "500".to_string(),
            crate::node::schema::FontWeight::W600 => "600".to_string(),
            crate::node::schema::FontWeight::W700 => "700".to_string(),
            crate::node::schema::FontWeight::W800 => "800".to_string(),
            crate::node::schema::FontWeight::W900 => "900".to_string(),
        }
    })
}

/// Maps text align from schema to internal representation
fn map_text_align(align: &Option<crate::node::schema::TextAlign>) -> Option<String> {
    align.as_ref().map(|a| {
        match a {
            crate::node::schema::TextAlign::Left => "left".to_string(),
            crate::node::schema::TextAlign::Right => "right".to_string(),
            crate::node::schema::TextAlign::Center => "center".to_string(),
            crate::node::schema::TextAlign::Justify => "justify".to_string(),
        }
    })
}

/// Maps style properties from schema to node
fn apply_common_style<T: crate::node::NodeCommon>(node: &mut T, style: &Option<Style>) {
    if let Some(style) = style {
        // Background color
        node.set_fill(convert_color(&style.background_color));
        
        // Border properties
        let border_color = convert_color(&style.border_color);
        let border_width = style.border_width.as_ref()
            .and_then(|w| w.parse::<f32>().ok())
            .unwrap_or(0.0);
        node.set_border(border_color, border_width);
        
        // Corner radius
        let corner_radius = style.border_radius.as_ref()
            .and_then(|r| r.parse::<f32>().ok())
            .unwrap_or(0.0);
        node.set_corner_radius(corner_radius);
        
        // TODO: Shadow handling
    }
}

/// Maps typography style properties from schema to node
fn apply_typography_style<T: NodeTypography>(node: &mut T, style: &Option<Style>) {
    if let Some(style) = style {
        // Font family
        node.set_font_family(style.font_family.clone());
        
        // Font size
        node.set_font_size(style.font_size.clone());
        
        // Font weight
        node.set_font_weight(map_font_weight(&style.font_weight));
        
        // Text color
        node.set_text_color(convert_color(&style.color));
        
        // Text align
        node.set_text_align(map_text_align(&style.text_align));
    }
}

/// Importer for converting serialized nodes to internal node representations
pub struct NodeImporter {
    factory: NodeFactory,
    node_map: HashMap<String, NodeId>, // Maps id strings to NodeIds
}

impl NodeImporter {
    pub fn new(factory: NodeFactory) -> Self {
        Self {
            factory,
            node_map: HashMap::new(),
        }
    }
    
    /// Import a SerializedNode and all its children
    pub fn import(&mut self, serialized: &SerializedNode) -> NodeId {
        match serialized {
            SerializedNode::Div { id, classes, style, children } => {
                self.import_div(id, classes, style, children)
            },
            SerializedNode::Image { id, classes, style, src, alt } => {
                self.import_image(id, classes, style, src, alt)
            },
            SerializedNode::Text { id, classes, style, content } => {
                self.import_text(id, classes, style, content)
            },
        }
    }
    
    /// Import a Div node (maps to Frame)
    fn import_div(
        &mut self, 
        id: &Option<String>, 
        classes: &Option<Vec<String>>, 
        style: &Option<Style>, 
        children: &Vec<SerializedNode>
    ) -> NodeId {
        // Create frame node
        let mut frame = self.factory.create_frame();
        
        // Apply styles
        apply_common_style(&mut frame, style);
        apply_typography_style(&mut frame, style);
        
        // Import children and add them to the frame
        for child in children {
            let child_id = self.import(child);
            frame.add_child(child_id);
        }
        
        // Store node and return its ID
        let node_id = frame.id();
        if let Some(id_str) = id {
            self.node_map.insert(id_str.clone(), node_id);
        }
        
        node_id
    }
    
    /// Import an Image node
    fn import_image(
        &mut self, 
        id: &Option<String>, 
        classes: &Option<Vec<String>>, 
        style: &Option<Style>, 
        src: &String, 
        alt: &Option<String>
    ) -> NodeId {
        // Create image node
        let mut image = self.factory.create_image(src.clone());
        
        // Apply styles
        apply_common_style(&mut image, style);
        
        // Set alt text
        image.set_alt_text(alt.clone());
        
        // Set object fit if specified
        if let Some(style) = style {
            image.set_object_fit(style.object_fit.clone());
        }
        
        // Store node and return its ID
        let node_id = image.id();
        if let Some(id_str) = id {
            self.node_map.insert(id_str.clone(), node_id);
        }
        
        node_id
    }
    
    /// Import a Text node
    fn import_text(
        &mut self, 
        id: &Option<String>, 
        classes: &Option<Vec<String>>, 
        style: &Option<Style>, 
        content: &String
    ) -> NodeId {
        // Create text node
        let mut text = self.factory.create_text(content.clone());
        
        // Apply styles
        apply_common_style(&mut text, style);
        apply_typography_style(&mut text, style);
        
        // Store node and return its ID
        let node_id = text.id();
        if let Some(id_str) = id {
            self.node_map.insert(id_str.clone(), node_id);
        }
        
        node_id
    }
    
    /// Get the node mappings (ID string to NodeId)
    pub fn node_map(&self) -> &HashMap<String, NodeId> {
        &self.node_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_import_simple_text() {
        let mut factory = NodeFactory::new();
        let mut importer = NodeImporter::new(factory);
        
        let text_node = SerializedNode::Text {
            id: Some("test-text".to_string()),
            classes: None,
            style: None,
            content: "Hello, world!".to_string()
        };
        
        let node_id = importer.import(&text_node);
        assert!(importer.node_map().contains_key("test-text"));
    }
}