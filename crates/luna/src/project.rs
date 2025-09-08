use anyhow::{anyhow, Result};
use gpui::{point, Hsla, Point};
use node::{AnyNode, NodeId, NodeLayout};
use scene_graph::SceneGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Version of the Luna project file format
pub const PROJECT_FORMAT_VERSION: u32 = 1;

/// A Luna project that can be saved to and loaded from disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LunaProject {
    /// Version of the file format for future compatibility
    pub version: u32,

    /// Project metadata
    pub metadata: ProjectMetadata,

    /// Pages in the project (currently we'll only use one)
    pub pages: Vec<Page>,

    /// Index of the currently active page
    pub active_page: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// When the project was created
    pub created_at: Option<String>,

    /// When the project was last modified
    pub modified_at: Option<String>,

    /// Optional project name
    pub name: Option<String>,

    /// Optional project description
    pub description: Option<String>,
}

/// A page within a Luna project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// Unique identifier for the page
    pub id: String,

    /// Name of the page
    pub name: String,

    /// Canvas state for this page
    pub canvas: CanvasState,

    /// All nodes on this page
    pub nodes: Vec<SerializedNode>,

    /// Parent-child relationships between nodes
    pub hierarchy: Vec<NodeRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasState {
    /// Viewport position
    pub viewport_x: f32,
    pub viewport_y: f32,

    /// Zoom level
    pub zoom: f32,

    /// Canvas background color
    pub background_color: Option<SerializedColor>,

    /// Currently selected node IDs
    pub selected_nodes: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRelationship {
    pub parent_id: usize,
    pub child_ids: Vec<usize>,
}

/// Serializable representation of a node
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerializedNode {
    Frame {
        id: usize,
        layout: SerializedLayout,
        fill: Option<SerializedColor>,
        border_color: Option<SerializedColor>,
        border_width: f32,
        corner_radius: f32,
        shadows: Vec<SerializedShadow>,
    },
    Shape {
        id: usize,
        layout: SerializedLayout,
        shape_type: String,
        fill: Option<SerializedColor>,
        border_color: Option<SerializedColor>,
        border_width: f32,
        corner_radius: f32,
        shadows: Vec<SerializedShadow>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedColor {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedShadow {
    pub color: SerializedColor,
    pub x_offset: f32,
    pub y_offset: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
}

impl LunaProject {
    /// Creates a new empty project with a single page
    pub fn new() -> Self {
        Self {
            version: PROJECT_FORMAT_VERSION,
            metadata: ProjectMetadata {
                created_at: Some(chrono::Utc::now().to_rfc3339()),
                modified_at: Some(chrono::Utc::now().to_rfc3339()),
                name: None,
                description: None,
            },
            pages: vec![Page {
                id: uuid::Uuid::new_v4().to_string(),
                name: "Page 1".to_string(),
                canvas: CanvasState {
                    viewport_x: 0.0,
                    viewport_y: 0.0,
                    zoom: 1.0,
                    background_color: None,
                    selected_nodes: Vec::new(),
                },
                nodes: Vec::new(),
                hierarchy: Vec::new(),
            }],
            active_page: 0,
        }
    }

    /// Saves the project to a file
    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        smol::fs::write(path, json).await?;
        Ok(())
    }

    /// Loads a project from a file
    pub async fn load_from_file(path: &Path) -> Result<Self> {
        let contents = smol::fs::read_to_string(path).await?;
        let project: LunaProject = serde_json::from_str(&contents)?;

        // Validate version compatibility
        if project.version > PROJECT_FORMAT_VERSION {
            return Err(anyhow!(
                "Project file version {} is newer than supported version {}",
                project.version,
                PROJECT_FORMAT_VERSION
            ));
        }

        Ok(project)
    }

    /// Gets the active page
    pub fn active_page(&self) -> Option<&Page> {
        self.pages.get(self.active_page)
    }

    /// Gets the active page mutably
    pub fn active_page_mut(&mut self) -> Option<&mut Page> {
        self.pages.get_mut(self.active_page)
    }
}

impl Default for LunaProject {
    fn default() -> Self {
        Self::new()
    }
}

// Conversion helpers

impl From<Hsla> for SerializedColor {
    fn from(color: Hsla) -> Self {
        Self {
            h: color.h,
            s: color.s,
            l: color.l,
            a: color.a,
        }
    }
}

impl From<SerializedColor> for Hsla {
    fn from(color: SerializedColor) -> Self {
        Hsla {
            h: color.h,
            s: color.s,
            l: color.l,
            a: color.a,
        }
    }
}

impl From<NodeLayout> for SerializedLayout {
    fn from(layout: NodeLayout) -> Self {
        Self {
            x: layout.x,
            y: layout.y,
            width: layout.width,
            height: layout.height,
        }
    }
}

impl From<SerializedLayout> for NodeLayout {
    fn from(layout: SerializedLayout) -> Self {
        Self {
            x: layout.x,
            y: layout.y,
            width: layout.width,
            height: layout.height,
        }
    }
}

/// Manages the current project state including file path and dirty state
pub struct ProjectState {
    /// The current project
    pub project: LunaProject,

    /// Path to the current project file (None for unsaved projects)
    pub file_path: Option<PathBuf>,

    /// Whether the project has unsaved changes
    pub is_dirty: bool,
}

impl ProjectState {
    /// Creates a new unsaved project state
    pub fn new() -> Self {
        Self {
            project: LunaProject::new(),
            file_path: None,
            is_dirty: false,
        }
    }

    /// Marks the project as having unsaved changes
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
        self.project.metadata.modified_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Clears the dirty flag (typically after saving)
    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    /// Gets the display name for the project (filename or "Untitled")
    pub fn display_name(&self) -> String {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    /// Checks if this is a new unsaved project
    pub fn is_new(&self) -> bool {
        self.file_path.is_none()
    }
}

impl Default for ProjectState {
    fn default() -> Self {
        Self::new()
    }
}
