Long range goal: Luna is software design tool in the vein of figma, sketchapp, and old Framer. It can swap between rendering nodes on a canvas, and editing specific nodes and their children in code directly.

Use these points and style guide to inform your plan an implementation:

- Eventually nodes and code will be interchangable. For example, a `FrameNode` would be interchangable with a html `div` with styles on it. A given frame and it's children will be serializable in some data format, or as html.
- Prefer single files for a single concept. Example: color.rs (handles color), node.rs (defines common node properties.), node/frame.rs (implements node properties and frame-speciifc behavior.)
- Don't be afraid of large files. As long as it is well organized, a 3000 line file isn't a problem to me. More files doesn't equal better organization.
- Respect the roles of core components: The data structure for nodes is a flat list. The scene graph manages  spatial relationships between nodes for efficient transformations. Tools represent top level intents, like selecting and moving nodes, creating specific types of nodes, etc. The canvas stores all canvas-related data, while the CanvasElement renders it and is the layer that handles interactions.
- Avoid rewriting things that already have solid libraries (outside of the scene graph, that should be fullly custom.) For example, the rust Palette crate could handle color operations (we just need to map to gpui colors when we render them.)
- Remember you can fetch information with your fetch tool. For example, if your knowledge of the Palette crate is out of date, and you need to know about the SRGB type, you could fetch info from docs.rs (https://docs.rs/palette/latest/palette/type.Srgb.html)
