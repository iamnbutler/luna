//! # Image Node Implementation
//!
//! Implements the ImageNode type, a visual element that displays images with configurable
//! styling properties.

use crate::node::{NodeCommon, NodeId, NodeLayout, NodeType, NodeImage, Shadow};
use gpui::Hsla;
use smallvec::{smallvec, SmallVec};

/// Concrete implementation of an image visual element
///
/// ImageNode represents an element that displays an image with configurable:
/// - Position and dimensions via NodeLayout
/// - Source URL or path for the image
/// - Alt text for accessibility
/// - Fill color (optional background)
/// - Border properties (color and width)
/// - Corner radius for rounded corners
/// - Object fit styling
#[derive(Debug, Clone)]
pub struct ImageNode {
    pub id: NodeId,
    pub layout: NodeLayout,
    pub src: String,
    pub alt: Option<String>,
    pub object_fit: Option<String>,
    pub fill: Option<Hsla>,
    pub border_color: Option<Hsla>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub shadows: SmallVec<[Shadow; 1]>,
}

impl ImageNode {
    pub fn new(id: NodeId, src: String) -> Self {
        Self {
            id,
            layout: NodeLayout::new(0.0, 0.0, 100.0, 100.0),
            src,
            alt: None,
            object_fit: None,
            fill: None, // Images typically don't have a background fill
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            shadows: smallvec![],
        }
    }

    /// Create an image with specific dimensions and position
    pub fn with_rect(id: NodeId, src: String, x: f32, y: f32, width: f32, height: f32) -> Self {
        let mut node = Self::new(id, src);
        node.layout = NodeLayout::new(x, y, width, height);
        node
    }
}

impl NodeCommon for ImageNode {
    fn id(&self) -> NodeId {
        self.id
    }

    fn node_type(&self) -> NodeType {
        NodeType::Image
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

impl NodeImage for ImageNode {
    fn image_src(&self) -> &str {
        &self.src
    }
    
    fn set_image_src(&mut self, src: String) {
        self.src = src;
    }
    
    fn alt_text(&self) -> Option<&str> {
        self.alt.as_deref()
    }
    
    fn set_alt_text(&mut self, alt: Option<String>) {
        self.alt = alt;
    }
    
    fn object_fit(&self) -> Option<String> {
        self.object_fit.clone()
    }
    
    fn set_object_fit(&mut self, fit: Option<String>) {
        self.object_fit = fit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Point;

    #[test]
    fn test_image_node() {
        let id = NodeId::new(2);
        let src = "https://example.com/image.jpg".to_string();
        let image = ImageNode::new(id, src.clone());

        assert_eq!(image.node_type(), NodeType::Image);
        assert_eq!(image.id(), id);
        assert_eq!(image.image_src(), src);
        assert_eq!(image.corner_radius(), 0.0);
    }

    #[test]
    fn test_contains_point() {
        let id = NodeId::new(1);
        let src = "https://example.com/image.jpg".to_string();
        let image = ImageNode::with_rect(id, src, 10.0, 10.0, 100.0, 100.0);

        // Test points inside and outside
        let point_inside = Point::new(50.0, 50.0);
        let point_outside = Point::new(200.0, 200.0);

        assert!(image.contains_point(&point_inside));
        assert!(!image.contains_point(&point_outside));
    }
}