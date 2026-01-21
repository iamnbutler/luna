//! Luna project package format.
//!
//! A `.luna` project is a folder containing:
//! - `manifest.kdl` - Project metadata and page list
//! - `pages/` - One .kdl file per page/canvas
//! - `assets/` - Linked images, fonts, etc. (future)
//! - `shaders/` - Custom shaders (future)
//!
//! # Example structure
//!
//! ```text
//! my_project.luna/
//! ├── manifest.kdl
//! └── pages/
//!     └── canvas.kdl
//! ```
//!
//! # Manifest format
//!
//! ```kdl
//! project version="0.1" {
//!   name "My Project"
//!   pages {
//!     page "canvas" file="pages/canvas.kdl"
//!   }
//! }
//! ```

use crate::{Document, InterchangeError, FORMAT_VERSION};
use kdl::{KdlDocument, KdlEntry, KdlNode};
use std::path::Path;

/// A Luna project (folder-based package).
#[derive(Debug, Clone)]
pub struct Project {
    /// Project name
    pub name: String,
    /// Format version
    pub version: String,
    /// Pages in the project (name -> document)
    pub pages: Vec<(String, Document)>,
}

impl Project {
    /// Create a new project with a single canvas.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: FORMAT_VERSION.to_string(),
            pages: vec![("canvas".to_string(), Document::new(vec![]))],
        }
    }

    /// Create a project from a single document.
    pub fn from_document(name: impl Into<String>, doc: Document) -> Self {
        Self {
            name: name.into(),
            version: FORMAT_VERSION.to_string(),
            pages: vec![("canvas".to_string(), doc)],
        }
    }

    /// Get the default/first page.
    pub fn default_page(&self) -> Option<&Document> {
        self.pages.first().map(|(_, doc)| doc)
    }

    /// Get a mutable reference to the default/first page.
    pub fn default_page_mut(&mut self) -> Option<&mut Document> {
        self.pages.first_mut().map(|(_, doc)| doc)
    }

    /// Save the project to a .luna folder.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), InterchangeError> {
        let path = path.as_ref();

        // Ensure path ends with .luna
        let path = if path.extension().map(|e| e == "luna").unwrap_or(false) {
            path.to_path_buf()
        } else {
            path.with_extension("luna")
        };

        // Create directory structure
        std::fs::create_dir_all(&path)
            .map_err(|e| InterchangeError::Parse(format!("Failed to create directory: {}", e)))?;

        let pages_dir = path.join("pages");
        std::fs::create_dir_all(&pages_dir)
            .map_err(|e| InterchangeError::Parse(format!("Failed to create pages directory: {}", e)))?;

        // Write manifest
        let manifest = self.to_manifest_kdl();
        std::fs::write(path.join("manifest.kdl"), manifest)
            .map_err(|e| InterchangeError::Parse(format!("Failed to write manifest: {}", e)))?;

        // Write each page
        for (name, doc) in &self.pages {
            let page_path = pages_dir.join(format!("{}.kdl", name));
            std::fs::write(&page_path, doc.to_kdl())
                .map_err(|e| InterchangeError::Parse(format!("Failed to write page {}: {}", name, e)))?;
        }

        Ok(())
    }

    /// Load a project from a .luna folder.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, InterchangeError> {
        let path = path.as_ref();

        // Read manifest
        let manifest_path = path.join("manifest.kdl");
        let manifest_str = std::fs::read_to_string(&manifest_path)
            .map_err(|e| InterchangeError::Parse(format!("Failed to read manifest: {}", e)))?;

        let manifest: KdlDocument = manifest_str
            .parse()
            .map_err(|e| InterchangeError::Parse(format!("Failed to parse manifest: {}", e)))?;

        // Parse manifest
        let project_node = manifest
            .get("project")
            .ok_or_else(|| InterchangeError::InvalidStructure("Missing 'project' node in manifest".into()))?;

        let version = project_node
            .get("version")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| FORMAT_VERSION.to_string());

        let name = project_node
            .children()
            .and_then(|c| c.get("name"))
            .and_then(|n| n.entries().first())
            .and_then(|e| e.value().as_string())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        // Parse pages from manifest
        let mut pages = Vec::new();
        if let Some(children) = project_node.children() {
            if let Some(pages_node) = children.get("pages") {
                if let Some(pages_children) = pages_node.children() {
                    for page_node in pages_children.nodes() {
                        if page_node.name().value() == "page" {
                            let page_name = page_node
                                .entries()
                                .iter()
                                .find(|e| e.name().is_none())
                                .and_then(|e| e.value().as_string())
                                .unwrap_or("canvas");

                            let file = page_node
                                .get("file")
                                .and_then(|v| v.as_string())
                                .unwrap_or("pages/canvas.kdl");

                            // Load the page file
                            let page_path = path.join(file);
                            let page_str = std::fs::read_to_string(&page_path)
                                .map_err(|e| InterchangeError::Parse(format!("Failed to read page {}: {}", page_name, e)))?;

                            let doc = Document::from_kdl(&page_str)?;
                            pages.push((page_name.to_string(), doc));
                        }
                    }
                }
            }
        }

        // If no pages found, try loading default
        if pages.is_empty() {
            let default_path = path.join("pages/canvas.kdl");
            if default_path.exists() {
                let page_str = std::fs::read_to_string(&default_path)
                    .map_err(|e| InterchangeError::Parse(format!("Failed to read default page: {}", e)))?;
                let doc = Document::from_kdl(&page_str)?;
                pages.push(("canvas".to_string(), doc));
            }
        }

        Ok(Self { name, version, pages })
    }

    /// Generate manifest KDL.
    fn to_manifest_kdl(&self) -> String {
        let mut doc = KdlDocument::new();

        let mut project_node = KdlNode::new("project");
        project_node.push(KdlEntry::new_prop("version", self.version.clone()));

        let children = project_node.children_mut().get_or_insert_with(KdlDocument::new);

        // Add name
        let mut name_node = KdlNode::new("name");
        name_node.push(KdlEntry::new(self.name.clone()));
        children.nodes_mut().push(name_node);

        // Add pages
        let mut pages_node = KdlNode::new("pages");
        let pages_children = pages_node.children_mut().get_or_insert_with(KdlDocument::new);

        for (name, _) in &self.pages {
            let mut page_node = KdlNode::new("page");
            page_node.push(KdlEntry::new(name.clone()));
            page_node.push(KdlEntry::new_prop("file", format!("pages/{}.kdl", name)));
            pages_children.nodes_mut().push(page_node);
        }

        children.nodes_mut().push(pages_node);

        doc.nodes_mut().push(project_node);
        doc.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use node_2::Shape;

    #[test]
    fn test_project_roundtrip() {
        let temp_dir = std::env::temp_dir().join("luna_test_project");
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean up any previous test

        // Create a project with some shapes
        let shapes = vec![
            Shape::rectangle(Vec2::new(100.0, 100.0), Vec2::new(150.0, 100.0))
                .with_fill(gpui::Hsla { h: 0.6, s: 0.8, l: 0.5, a: 1.0 }),
            Shape::ellipse(Vec2::new(300.0, 150.0), Vec2::new(120.0, 120.0))
                .with_stroke(gpui::Hsla { h: 0.0, s: 0.0, l: 0.0, a: 1.0 }, 2.0),
        ];
        let doc = Document::new(shapes);
        let project = Project::from_document("Test Project", doc);

        // Save it
        let project_path = temp_dir.join("test.luna");
        project.save(&project_path).expect("Failed to save");

        // Print the files for inspection
        println!("\n=== Project structure ===");
        println!("test.luna/");
        println!("├── manifest.kdl");
        println!("└── pages/");
        println!("    └── canvas.kdl");

        println!("\n=== manifest.kdl ===");
        println!("{}", std::fs::read_to_string(project_path.join("manifest.kdl")).unwrap());

        println!("=== pages/canvas.kdl ===");
        println!("{}", std::fs::read_to_string(project_path.join("pages/canvas.kdl")).unwrap());

        // Verify files exist
        assert!(project_path.join("manifest.kdl").exists());
        assert!(project_path.join("pages/canvas.kdl").exists());

        // Load it back
        let loaded = Project::load(&project_path).expect("Failed to load");

        assert_eq!(loaded.name, "Test Project");
        assert_eq!(loaded.pages.len(), 1);
        assert_eq!(loaded.pages[0].0, "canvas");
        assert_eq!(loaded.pages[0].1.shapes.len(), 2);

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
