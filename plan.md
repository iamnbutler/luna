# Luna – Description & Development Plan

Our goal is to build a highly performant, scalable canvas application—akin to Figma—that supports complex interactions (drag-and-drop, reparenting, scaling, rotation) even with many levels of nested elements. We’ll use an ECS-inspired architecture with a strict type system that clearly distinguishes between local and world coordinates. Testing and test-driven development (TDD) are central to our approach, and we’ll integrate visual debugging tools to aid development and troubleshooting.

---

## Key Design Principles

### 1. Performance
- **Efficient Layout and Rendering:**
  - Use spatial indexing (e.g., quadtrees) so that only visible elements are processed for hit testing and rendering.
  - Cache world transforms computed by composing local transforms to minimize unnecessary recalculations.
- **Scalability:**
  - Design to support a theoretically infinite canvas and thousands of nested elements without performance degradation.
  - Optimize critical paths in transform composition, event propagation, and rendering.

### 2. Testing & Test-Driven Development
- **TDD Approach:**
  - Write comprehensive unit tests for core math operations (vector arithmetic, transform composition), ECS functions (entity management, component updates), and collision detection (AABB computations, hit testing).
  - Develop integration tests for user flows (e.g., dragging, reparenting, scaling) using a virtual canvas (TestCanvas) that simulates actions and verifies state transitions.
- **Visual Debugging:**
  - Implement debug overlays that render bounding boxes, wireframes for clipping, and transformation controls.
  - Use these visual tools to quickly identify discrepancies in layout or transform computations.

### 3. Strict Type System & ECS Principles
- **Local vs. World Coordinates:**
  - Define explicit types (`LocalPosition`, `WorldPosition`, `LocalTransform`, `WorldTransform`) to prevent mixing coordinate spaces.
  - Provide helper methods for converting between these types.
- **ECS Architecture:**
  - **Entities:** Represented by unique IDs (e.g., `LunaEntityId`), with no intrinsic data.
  - **Components:** Store all data in dedicated structures (TransformComponent, HierarchyComponent, RenderComponent, etc.).
  - **Systems:** Process component data (e.g., TransformSystem to compute world transforms, RenderSystem to draw elements, HitTestSystem for efficient interaction).

---

## Short-Range Needs and Goals
- **MVP Features:**
  - Create a basic canvas that supports the creation and movement of elements.
  - Implement a simple scene graph with local-to-world transform composition.
  - Integrate a basic quadtree for spatial indexing and hit testing.
- **Testing & Debugging:**
  - Develop unit tests for vector math, transform composition, and AABB hit testing.
  - Implement visual debug overlays (bounding boxes, wireframe views) to validate layout and transform logic.

---

## Long-Range Needs and Goals
- **Complex Interactions:**
  - Support advanced interactions such as reparenting elements, interactive scaling, rotation, and drag-and-drop.
  - Build a rich UI including a layer list (to manage hierarchy), transform controls (for scaling/rotation), and property inspectors.
- **High Performance at Scale:**
  - Ensure the system remains responsive even with thousands of elements and deep nesting.
  - Optimize update flows, caching of transforms, and event handling.
- **Extensibility and Maintainability:**
  - Use an ECS framework with a flat data structure (e.g., hash maps keyed by EntityId) to store components.
  - Maintain a single source of truth for positions and layout so that every update (via events like `request_layout_update`) flows through a well-defined pipeline.
  - Provide clear, self-documenting types and conversion methods to minimize errors.

---

## Important Notes
- **Adhere Strictly to the Type System:**
  - Always use `LocalPosition` for data relative to a parent and `WorldPosition` for absolute positions.
  - Ensure that all transforms are composed via explicit methods (e.g., `LocalTransform::to_world(&parent_transform)`) to avoid double-application.
- **Follow ECS Patterns:**
  - Treat entities as minimal IDs, with all data stored in separate, flat components (e.g., TransformComponent, HierarchyComponent, RenderComponent).
  - Build systems that process these components in a decoupled manner (e.g., TransformSystem, RenderSystem, HitTestSystem).
- **Prioritize Testing and Visual Debugging:**
  - Use TDD to drive development, writing tests for every core function before implementation.
  - Integrate visual debugging tools to render overlays (bounding boxes, wireframe clipping, transform controls) to assist in diagnosing issues.
- **Plan for Both Short and Long Term:**
  - In the short term, focus on a robust, testable MVP that includes basic element manipulation and rendering.
  - In the long term, extend the system to support complex UI interactions (reparenting, multi-selection, property editing) and optimize for performance at scale.
- **Data Storage Considerations:**
  - Consider a centralized state (using hash maps keyed by entity IDs) for fast lookups and updates.
  - Evaluate whether a flat structure or rebuilding the scene graph from a flat map is best for your performance needs.

---

## ECS Organized Components

### Entities
- **LunaEntityId**
  - Unique identifier for each canvas element.
  - *Role:* Acts as a handle; no data is stored in the entity itself, only in its components.

### Components
- **TransformComponent**
  - **LocalTransform:** Contains local position (e.g., using `LocalPosition` or `Vec2`), scale, and rotation (relative to the parent).
  - **WorldTransform:** The composed transform (from local and parent transforms); either computed on the fly or cached.
  - *Role:* Determines an element's position in the scene and supports conversion between local and world space.

- **HierarchyComponent**
  - Stores parent–child relationships:
    - `parent: Option<LunaEntityId>`
    - `children: Vec<LunaEntityId>`
  - *Role:* Represents the scene graph in a flat data structure.

- **RenderComponent (or StyleComponent)**
  - Contains visual properties: width, height, corner radius, colors, etc.
  - May include computed bounding boxes (from the transform and style).
  - *Role:* Determines how an element is drawn on the canvas.

- **HitboxComponent (Optional)**
  - Contains a bounding box (or similar structure) for hit-testing.
  - *Role:* Optimizes interaction queries.

- **LayoutComponent (Optional)**
  - Contains layout constraints and sizing rules.
  - *Role:* Allows the system to recompute positions and sizes based on layout updates.

- **DebugComponent (Optional)**
  - Flags or additional data for rendering visual debugging information:
    - Render bounding boxes, wireframes for clipping, etc.
  - *Role:* Helps visually debug layout, transforms, and hitboxes.

### Systems
- **TransformSystem**
  - Iterates over entities and computes/updates their WorldTransform by traversing the hierarchy.
  - *Role:* Ensures that every element’s world position is current.

- **RenderSystem**
  - Reads RenderComponent and WorldTransform to draw elements.
  - *Role:* Produces the final visual representation on the canvas.
  - *Enhancement:* Integrate visual debugging (e.g., drawing bounding boxes, clipping wireframes).

- **HitTestSystem**
  - Uses bounding boxes (or HitboxComponents) and a spatial index (e.g., a quadtree) to determine which element is under a given point.
  - *Role:* Supports efficient hit testing and user interactions.

- **LayoutSystem**
  - Processes layout updates (e.g., via `request_layout_update`) to adjust TransformComponents and LayoutComponents.
  - *Role:* Centralizes and enforces layout constraints and updates.

- **HierarchySystem (Optional)**
  - Maintains and updates the scene graph (parent–child relationships) stored in HierarchyComponents.
  - *Role:* Provides an easy way to traverse or rebuild the tree when necessary.

- **DebugRenderSystem (Optional)**
  - A specialized system to render additional debug overlays:
    - Bounding boxes (wireframes)
    - Clipped element outlines
    - Transform control visuals
  - *Role:* Speeds up debugging of layout, transform, and hit-test issues.

---

## Data Storage Strategy

- **Centralized State:**
  Use flat hash maps keyed by LunaEntityId for each component:
  - `HashMap<LunaEntityId, TransformComponent>`
  - `HashMap<LunaEntityId, RenderComponent>`
  - `HashMap<LunaEntityId, HierarchyComponent>`
- **Separation of Concerns:**
  Possibly maintain a separate spatial index (e.g., a quadtree) built from world transforms for hit testing.
- **TestCanvas:**
  Create a specialized module (or a variant of the Canvas) that can simulate a virtual canvas. This module should:
  - Allow scripted series of actions (like reparenting, moving, scaling, rotating, and unparenting).
  - Expose APIs to query state changes, making it possible to write automated tests against user flows.

---

## Ordered To-Do List

1. **Define Core Types and Primitives**
   - Create type definitions for:
     - [x] `LunaLunaEntityId`
     - [x] `Vector2D`
     - [x] `LocalPosition` and `WorldPosition`
     - [x] `LocalTransform` (and optionally a `WorldTransform` alias)
     - [x] `BoundingBox` (or AABB) with helper methods (intersection, containment, etc.)
   - Write tests to verify basic arithmetic and conversion functions.
   - **Visual Debugging:**
     - Add helper methods to output or draw the BoundingBox as a wireframe for debugging.

2. **Implement Core Components**
   - Define components: TransformComponent, HierarchyComponent, RenderComponent.
   - Decide on a data structure (e.g., hash maps) to store these components by LunaEntityId.
   - Write unit tests for component CRUD operations.
   - **Visual Debugging:**
     - Introduce a DebugComponent flag to optionally render extra info (bounding boxes, transform markers).

3. **Build the ECS Framework / Manager**
   - Create a manager that registers entities and attaches components.
   - Implement basic functions for adding, removing, and updating entities.
   - Write tests to ensure entities and components are correctly managed.
   - **TestCanvas:**
     - Create a TestCanvas module that simulates a canvas, allowing scripted actions and state queries.

4. **Implement the Transform System**
   - Write a system that computes WorldTransform from LocalTransform, traversing the hierarchy using HierarchyComponents.
   - Ensure proper composition of transforms (local-to-parent → world).
   - Write tests for various hierarchy configurations, including deep nesting.
   - **Visual Debugging:**
     - Render world transform axes or control points to verify proper transformation.

5. **Implement the Render System**
   - Create a rendering pipeline that reads RenderComponent and WorldTransform.
   - Start with simple rendering (basic shapes) to verify positions.
   - Write tests or manual test cases to validate element drawing positions.
   - **Visual Debugging:**
     - Implement overlay options to render bounding boxes and clipping wireframes.

6. **Implement the HitTest System with Spatial Indexing**
   - Integrate a quad tree (or similar spatial index) to index bounding boxes.
   - Develop functions to quickly determine which element is under a given screen point.
   - Write unit tests for hit testing under various scenarios.
   - **Visual Debugging:**
     - Optionally render the quad tree bounds as an overlay for verification.

7. **Implement the Layout System**
   - Design a LayoutComponent to hold constraints and sizing rules.
   - Implement a LayoutSystem that processes `request_layout_update` events and updates TransformComponents accordingly.
   - Write tests to validate layout updates and constraint resolutions.

8. **Implement Input and Interaction Handling**
   - Add event listeners for mouse and keyboard events.
   - Implement dragging for moving elements.
   - Implement drag-and-drop for reparenting elements (and update HierarchyComponents accordingly).
   - Write integration tests for user interactions (dragging, selection, reparenting).
   - **Visual Elements:**
     - Develop UI elements for:
       - Drag-to-select (a marquee selection tool)
       - Visual feedback for selection (highlighting selected elements)
       - Transform controls (handles for scaling and rotating)

9. **Implement Scaling and Rotation**
   - Extend TransformComponent to fully support scaling and rotation.
   - Update TransformSystem to correctly handle these transformations.
   - Write tests that ensure scaled and rotated elements have correct world transforms and bounding boxes.
   - Verify that hit testing and rendering remain accurate under these transformations.
   - **Visual Debugging:**
     - Render transform controls (rotation circles, scale handles) over selected elements.

10. **Develop Additional Visual UI Components**
    - **Layer List Panel:**
      - A UI component that shows the hierarchy of elements (like layers in Photoshop).
      - Supports reordering, visibility toggling, and selection.
    - **Property Inspector:**
      - A panel for editing the properties of a selected element (position, scale, rotation, style).
    - **Drag-to-Select UI:**
      - A marquee tool for selecting multiple elements.
    - **Transform Controls UI:**
      - Handles or gizmos for interactive scaling, rotation, and moving.
    - Write integration tests and manual tests for these UI components.

11. **Integrate Everything into a Canvas Module**
    - Build a Canvas module that wraps the ECS state and provides a public API.
    - Support canvas-level operations (panning, zooming, etc.).
    - Write integration tests simulating typical user flows (creating elements, moving, reparenting, scaling/rotating).
    - **Visual Debugging:**
      - Ensure that debug overlays (bounding boxes, wireframes) can be toggled on/off.

12. **Optimize, Profile, and Finalize**
    - Profile the system with many nested elements.
    - Optimize update flows (e.g., cache world transforms, reduce unnecessary tree rebuilds).
    - Write regression tests and benchmarks for performance.
    - Finalize documentation and clean up code for consistency in naming and explicit coordinate space handling.

---

This updated plan provides a clear, ordered roadmap from core ECS primitives through interaction systems and visual debugging aids, culminating in a robust and scalable canvas app. Each step builds upon the previous, and the added visual tools (bounding box overlays, debug wireframes, and UI components like layer lists and transform controls) will greatly assist in testing and debugging complex scenarios.
