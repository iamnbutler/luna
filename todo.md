# Scene Graph Implementation TODO List

## Overview

This document outlines the steps needed to implement a scene graph on top of GPUI for canvas positioning, transformations, and hit testing while maintaining:
- A flat list for the data layer
- GPUI's Elements and painter's algorithm for rendering

## 1. Create Scene Graph Data Structure

- [x] Create a new file `src/scene_graph.rs` with:
  - [x] Define `SceneNodeId` type for unique scene node identifiers
  - [x] Implement `SceneGraph` struct with node hierarchy management
  - [x] Implement `SceneNode` struct with parent-child relationships and transforms

## 2. Core Scene Graph Operations

- [x] Implement node creation and hierarchy management
  - [x] `create_node(parent_id, data_node_id)` to create new scene nodes
  - [x] `add_child(parent_id, child_id)` to build hierarchy
  - [x] `remove_node(node_id)` to remove nodes
- [x] Implement transformation operations
  - [x] `set_local_transform(node_id, transform)` to set node transforms
  - [x] `update_world_transform(node_id)` to propagate transforms down the tree
  - [x] `update_world_bounds(node_id)` to update transformed bounds

## 3. Scene Graph Utility Operations

- [ ] Implement visibility culling
  - [ ] `collect_visible_nodes(viewport)` to get nodes visible in viewport
  - [ ] `is_node_visible(node_id, viewport)` to check visibility
- [ ] Implement hit testing
  - [ ] `hit_test(point)` for efficient hierarchical hit testing
  - [ ] `hit_test_node(node_id, point)` for recursive hit testing

## 4. Integrate Scene Graph with Canvas

- [ ] Modify `Canvas` struct to include the scene graph
```rust
pub struct Canvas {
    // Existing fields...
    scene_graph: SceneGraph,
    canvas_root_node: SceneNodeId,
}
```
- [ ] Initialize scene graph in `Canvas::new()`
```rust
pub fn new(window: &Window, cx: &mut Context<Self>) -> Self {
    // Existing initialization...
    let mut scene_graph = SceneGraph::new();
    let canvas_root_node = scene_graph.create_node(None, None);

    Self {
        // Existing fields...
        scene_graph,
        canvas_root_node,
    }
}
```

## 5. Node Management Integration

- [ ] Update `Canvas::add_node()` to:
  - [ ] Add the node to the flat data structure (keep this)
  - [ ] Create a corresponding scene node in the scene graph
  - [ ] Set initial bounds based on node layout
- [ ] Update `Canvas::remove_node()` to:
  - [ ] Remove the node from the flat data structure (keep this)
  - [ ] Remove the corresponding scene node from the scene graph

## 6. Replace Direct Transformations with Scene Graph

- [ ] Modify `Canvas::set_zoom()` to update canvas root node transform
```rust
pub fn set_zoom(&mut self, zoom: f32) {
    self.zoom = zoom.max(0.1).min(10.0);

    // Update canvas root transform
    self.update_canvas_transform();

    self.dirty = true;
}
```

- [ ] Modify `Canvas::set_scroll_position()` to update canvas root node transform
```rust
pub fn set_scroll_position(&mut self, position: Point<f32>) {
    self.scroll_position = position;

    // Update canvas root transform
    self.update_canvas_transform();

    self.dirty = true;
}
```

- [ ] Create a combined transform update method
```rust
fn update_canvas_transform(&mut self) {
    let transform = TransformationMatrix::unit()
        .scale(Size::new(self.zoom, self.zoom))
        .translate(Point::new(-self.scroll_position.x, -self.scroll_position.y));

    self.scene_graph.set_transform(self.canvas_root_node, transform);
}
```

## 7. Replace Hit Testing with Scene Graph

- [ ] Replace current hit testing with scene graph-based testing
```rust
pub fn nodes_at_point(&self, point: Point<f32>) -> Vec<NodeId> {
    // No need to convert point, scene graph handles transformations
    self.scene_graph.hit_test(point)
}

pub fn top_node_at_point(&self, point: Point<f32>) -> Option<NodeId> {
    self.nodes_at_point(point).first().copied()
}
```

## 8. Replace Visibility Culling with Scene Graph

- [ ] Replace manual culling with scene graph-based culling
```rust
pub fn visible_nodes(&self) -> Vec<&RectangleNode> {
    let viewport = Bounds {
        origin: Point::new(0.0, 0.0),
        size: self.viewport.size,
    };

    let visible_ids = self.scene_graph.collect_visible_nodes(viewport)
        .into_iter()
        .filter_map(|node_id| {
            self.scene_graph.get_data_node_id(node_id)
        })
        .collect::<Vec<_>>();

    self.nodes.iter()
        .filter(|node| visible_ids.contains(&node.id()))
        .collect()
}
```

## 9. Update Rendering in CanvasElement

- [ ] Modify `CanvasElement::paint_nodes()` to use scene graph transforms
```rust
fn paint_nodes(&self, layout: &CanvasLayout, window: &mut Window, theme: &Theme, cx: &App) {
    let canvas = self.canvas.read(cx);

    // Get viewport in window space
    let viewport = Bounds {
        origin: Point::new(0.0, 0.0),
        size: canvas.viewport.size,
    };

    // Get visible nodes with their transforms from scene graph
    let visible_nodes_with_transforms = canvas.scene_graph.get_visible_nodes(viewport);
    let selected_nodes = &canvas.selected_nodes;

    window.paint_layer(layout.hitbox.bounds, |window| {
        // Paint each node with its transformation
        for (node_id, transform) in visible_nodes_with_transforms {
            if let Some(node) = canvas.nodes.iter().find(|n| n.id() == node_id) {
                let layout = node.layout();

                // Create bounds in local space
                let bounds = Bounds {
                    origin: Point::new(
                        gpui::Pixels(0.0),
                        gpui::Pixels(0.0)
                    ),
                    size: Size::new(
                        gpui::Pixels(layout.width),
                        gpui::Pixels(layout.height)
                    ),
                };

                // Apply transformation from scene graph
                window.with_transformation(transform, |window| {
                    // Paint the fill if it exists
                    if let Some(fill_color) = node.fill() {
                        window.paint_quad(gpui::fill(bounds, fill_color));
                    }

                    // Paint the border if it exists
                    if let Some(border_color) = node.border_color() {
                        window.paint_quad(gpui::outline(bounds, border_color));
                    }

                    // Draw selection indicator if the node is selected
                    if selected_nodes.contains(&node.id()) {
                        let selection_bounds = Bounds {
                            origin: Point::new(
                                bounds.origin.x - gpui::Pixels(2.0),
                                bounds.origin.y - gpui::Pixels(2.0),
                            ),
                            size: Size::new(
                                bounds.size.width + gpui::Pixels(4.0),
                                bounds.size.height + gpui::Pixels(4.0),
                            ),
                        };
                        window.paint_quad(gpui::outline(selection_bounds, theme.selected));
                    }
                });
            }
        }

        window.request_animation_frame();
    });
}
```

## What to Remove

- [ ] Remove manual transformation calculations in `paint_nodes()`
  - [ ] Remove `adjusted_x`, `adjusted_y`, `adjusted_width`, `adjusted_height` calculations
- [ ] Remove viewport-based culling in `Canvas::visible_nodes()`
- [ ] Remove manual point conversion in `Canvas::nodes_at_point()`
- [ ] Remove `window_to_canvas_point()` and `canvas_to_window_point()` usage for rendering (scene graph handles this)

## Implementation Order

1. **First Phase: Core Data Structures**
   - [ ] Implement `SceneGraph` and `SceneNode` with basic operations
   - [ ] Write tests for transform propagation
   - [ ] Test core functionality in isolation

2. **Second Phase: Canvas Integration**
   - [ ] Add scene graph to Canvas
   - [ ] Update node management to use scene graph alongside flat list
   - [ ] Implement transform handling through scene graph

3. **Third Phase: Visibility and Hit Testing**
   - [ ] Implement hierarchical hit testing
   - [ ] Implement visibility culling
   - [ ] Update Canvas methods to use scene graph for these operations

4. **Fourth Phase: Rendering Integration**
   - [ ] Update CanvasElement rendering to use scene graph transforms
   - [ ] Remove manual transform calculations
   - [ ] Test with various zoom levels and scroll positions

5. **Final Phase: Cleanup and Optimization**
   - [ ] Remove redundant code
   - [ ] Optimize performance bottlenecks
   - [ ] Add more advanced features (like grouping)

## Notes on Implementation

- Keep the flat data structure for nodes to maintain simplicity in data manipulation
- Use the scene graph for spatial organization, transformations, and hit testing
- Add node IDs to mapping tables to connect data nodes with scene nodes
- Let GPUI's painter's algorithm handle rendering with transformations from scene graph
