//! # Text Node Implementation
//!
//! Implements the TextNode type, a visual element that displays text with configurable
//! styling properties.

use crate::node::{NodeCommon, NodeId, NodeLayout, NodeType, NodeTypography, Shadow};
use gpui::Hsla;
use smallvec::{smallvec, SmallVec};

/// Concrete implementation of a text visual element
///
/// TextNode represents an element that displays text with configurable:
/// - Position and dimensions via NodeLayout
/// - Text content to display
/// - Typography properties (font family, size, weight, etc.)
/// - Fill color (background)
/// - Border properties (color and width)
/// - Corner radius for rounded background corners
#[derive(Debug, Clone)]
pub struct TextNode {
    pub id: NodeId,
    pub layout: NodeLayout,
    pub content: String,
    pub font_family: Option<String>,
    pub font_size: Option<String>,
    pub font_weight: Option<String>,
    pub text_color: Option<Hsla>,
    pub text_align: Option<String>,
    pub fill: Option<Hsla>,
    pub border_color: Option<Hsla>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub shadows: SmallVec<[Shadow; 1]>,
}

impl TextNode {
    pub fn new(id: NodeId, content: String) -> Self {
        Self {
            id,
            layout: NodeLayout::new(0.0, 0.0, 100.0, 100.0),
            content,
            font_family: None,
            font_size: None,
            font_weight: None,
            text_color: Some(Hsla::black()),
            text_align: None,
            fill: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            shadows: smallvec![],
        }
    }

    /// Create a text node with specific dimensions and position
    pub fn with_rect(id: NodeId, content: String, x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut node = Self::new(id, content);
        node.layout = NodeLayout::new(x, y, width, height);
        node
    }

    /// Get the text content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Set the text content
    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }
}

impl NodeCommon for TextNode {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Text
    }

    fn layout(&self) -> &NodeLayout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut NodeLayout {
        &mut self.layout
    }

    fn fill(&self) -> Option<Hsla> {
        self.fill
    }

    fn set_fill(&mut self, color: Option<Hsla>) {
        self.fill = color;
    }

    fn border_color(&self) -> Option<Hsla> {
        self.border_color
    }

    fn border_width(&self) -> f32 {
        self.border_width
    }

    fn set_border(&mut self, color: Option<Hsla>, width: f32) {
        self.border_color = color;
        self.border_width = width;
    }

    fn corner_radius(&self) -> f32 {
        self.corner_radius
    }

    fn set_corner_radius(&mut self, radius: f32) {
        self.corner_radius = radius;
    }

    fn shadows(&self) -> SmallVec<[Shadow; 1]> {
        self.shadows.clone()
    }

    fn set_shadows(&mut self, shadows: SmallVec<[Shadow; 1]>) {
        self.shadows = shadows
    }
}

impl NodeTypography for TextNode {
    fn font_family(&self) -> Option<String> {
        self.font_family.clone()
    }
    
    fn set_font_family(&mut self, family: Option<String>) {
        self.font_family = family;
    }
    
    fn font_size(&self) -> Option<String> {
        self.font_size.clone()
    }
    
    fn set_font_size(&mut self, size: Option<String>) {
        self.font_size = size;
    }
    
    fn font_weight(&self) -> Option<String> {
        self.font_weight.clone()
    }
    
    fn set_font_weight(&mut self, weight: Option<String>) {
        self.font_weight = weight;
    }
    
    fn text_color(&self) -> Option<Hsla> {
        self.text_color
    }
    
    fn set_text_color(&mut self, color: Option<Hsla>) {
        self.text_color = color;
    }
    
    fn text_align(&self) -> Option<String> {
        self.text_align.clone()
    }
    
    fn set_text_align(&mut self, align: Option<String>) {
        self.text_align = align;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Point;

    #[test]
    fn test_text_node() {
        let id = NodeId::new(2);
        let content = "Hello, world!".to_string();
        let text = TextNode::new(id, content.clone());

        assert_eq!(text.node_type(), NodeType::Text);
        assert_eq!(text.id(), id);
        assert_eq!(text.content(), content);
        assert_eq!(text.corner_radius(), 0.0);
    }

    #[test]
    fn test_contains_point() {
        let id = NodeId::new(1);
        let content = "Hello, world!".to_string();
        let text = TextNode::with_rect(id, content, 10.0, 10.0, 100.0, 100.0);

        // Test points inside and outside
        let point_inside = Point::new(50.0, 50.0);
        let point_outside = Point::new(200.0, 200.0);

        assert!(text.contains_point(&point_inside));
        assert!(!text.contains_point(&point_outside));
    }
}