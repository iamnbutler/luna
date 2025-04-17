//! # Luna: A software design tool without compromises.
//!
//! Luna is local, files on disk first. Own your own data,
//! collaborate, design on the canvas or write code.
//!
//! It's not a design tool, or a code editor, it's a tool
//! for designing software:
//!
//! That means not just pixels, but representative screens and flows
//! using an abstractionless design and editing experience.
//!
//! If Luna is a design tool for everyone I'll have failed in my
//! goals – it should be as complex as it needs to be without
//! trade-offs in service of making it easier to use.

struct Luna2 {
    /// The main canvas where elements are rendered and manipulated
    active_canvas: Entity<LunaCanvas>,
    /// Focus handle for keyboard event routing
    focus_handle: FocusHandle,
    /// Scene graph for managing spatial relationships between nodes
    // scene_graph: Entity<SceneGraph>,
    /// Modify elements and their properties
    // inspector: Entity<Inspector>,
    /// Choose tools, layers, and more.
    // sidebar: Entity<Sidebar>,
}
