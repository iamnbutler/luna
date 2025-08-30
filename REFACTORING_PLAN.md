# Luna Math & Coordinate System Refactoring

## Core Problem
We're using 3D graphics engine complexity for a simple 2D canvas that only does pan and zoom. We store position data 3 times, use 4x4 matrices for operations that need 2 adds and 2 multiplies, and have duplicate implementations of the same coordinate transforms.

## The Solution: Radical Simplification

### What We Actually Need
- 2D positions relative to parents (x, y offsets)
- Uniform zoom (single scale factor)
- Canvas pan (scroll offset)
- Axis-aligned bounding boxes (no rotation)

### What We're Deleting
- All 4x4 matrix operations
- Scene graph transforms (local_transform, world_transform)
- Duplicate position storage (local_bounds, world_bounds)
- Complex transform composition
- GPUI TransformationMatrix dependency
- ~500 lines of unnecessary code

## Implementation Plan

### Phase 1: Create Simple Transform System

Create a dead-simple transform for our actual needs:

```rust
// crates/luna_core/src/transform.rs
use glam::Vec2;

#[derive(Copy, Clone, Debug)]
pub struct CanvasTransform {
    pub offset: Vec2,  // scroll position
    pub scale: f32,    // zoom level
}

impl CanvasTransform {
    pub fn apply(&self, point: Vec2) -> Vec2 {
        point * self.scale + self.offset
    }
    
    pub fn apply_inverse(&self, point: Vec2) -> Vec2 {
        (point - self.offset) / self.scale
    }
}
```

### Phase 2: Simplify Bounds to Use glam Directly

```rust
// crates/luna_core/src/bounds.rs
use glam::Vec2;

#[derive(Copy, Clone, Debug)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl Bounds {
    pub fn from_origin_size(origin: Vec2, size: Vec2) -> Self {
        Self {
            min: origin,
            max: origin + size,
        }
    }
    
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x < other.max.x && 
        self.max.x > other.min.x &&
        self.min.y < other.max.y && 
        self.max.y > other.min.y
    }
    
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);
        
        (min.x <= max.x && min.y <= max.y).then_some(Self { min, max })
    }
    
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
    
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y
    }
}
```

### Phase 3: Eliminate SceneNode Position Storage

SceneNode should ONLY manage hierarchy. Position data lives in FrameNode:

```rust
// crates/scene_graph/src/scene_node.rs
pub struct SceneNode {
    parent: Option<SceneNodeId>,
    children: Vec<SceneNodeId>,
    node_id: NodeId,  // Links to the FrameNode which has position
    visible: bool,
    // NO position data, NO transforms, NO bounds
}
```

### Phase 4: Direct World Position Calculation

Calculate world positions on-demand from FrameNode data:

```rust
impl SceneGraph {
    pub fn get_world_position(&self, node_id: NodeId) -> Vec2 {
        let frame_node = self.get_frame_node(node_id);
        let local_pos = Vec2::new(frame_node.layout.x, frame_node.layout.y);
        
        if let Some(parent_id) = self.get_parent(node_id) {
            self.get_world_position(parent_id) + local_pos
        } else {
            local_pos
        }
    }
    
    pub fn get_world_bounds(&self, node_id: NodeId) -> Bounds {
        let pos = self.get_world_position(node_id);
        let frame_node = self.get_frame_node(node_id);
        Bounds::from_origin_size(
            pos, 
            Vec2::new(frame_node.layout.width, frame_node.layout.height)
        )
    }
}
```

### Phase 5: Unify Canvas Coordinate Transformation

One implementation, no matrices:

```rust
impl LunaCanvas {
    pub fn window_to_canvas(&self, window_pos: Vec2) -> Vec2 {
        let viewport_center = Vec2::new(
            self.viewport_size.width / 2.0,
            self.viewport_size.height / 2.0
        );
        let centered = window_pos - viewport_center;
        (centered / self.zoom) + self.scroll_position
    }
    
    pub fn canvas_to_window(&self, canvas_pos: Vec2) -> Vec2 {
        let viewport_center = Vec2::new(
            self.viewport_size.width / 2.0,
            self.viewport_size.height / 2.0
        );
        ((canvas_pos - self.scroll_position) * self.zoom) + viewport_center
    }
}
```

### Phase 6: Remove All Legacy Code

Delete:
- All TransformationMatrix usage
- SceneNode local_bounds and world_bounds
- SceneNode local_transform and world_transform
- update_world_transform and related methods
- Duplicate coordinate transformation implementations
- Complex bounds transformation methods
- GPUI matrix dependencies

## Execution Steps

### 1. Create New Core Types
- Implement `CanvasTransform` struct
- Implement simplified `Bounds` using glam
- Create coordinate wrapper types that use Vec2 internally

### 2. Gut the Scene Graph
- Remove all transform fields from SceneNode
- Remove all bounds fields from SceneNode
- Delete update_world_transform and related methods
- Implement simple get_world_position method

### 3. Simplify Canvas
- Replace TransformationMatrix with simple zoom/scroll fields
- Use the new window_to_canvas and canvas_to_window methods
- Delete all matrix-based transformation code

### 4. Update Rendering
- Use calculated world positions instead of cached
- Pass simple translate/scale to GPUI instead of matrices
- Update hit testing to use new bounds methods

### 5. Clean House
- Delete all unused imports
- Remove all TransformationMatrix code
- Remove all complex transform composition
- Delete coordinate conversion duplicates

## Expected Outcome

### Before
- 64 bytes per node for transforms
- 4x4 matrix operations for simple 2D math
- Position stored in 3 places
- ~600 lines of transform code
- Complex borrowing and update cycles

### After
- 0 bytes per node (position in FrameNode)
- 2 multiplies + 2 adds for transforms
- Position stored in 1 place
- ~80 lines of transform code
- Simple, direct calculations

## Key Principles

1. **No caching world positions** - Calculate on demand
2. **No matrices** - Just offset and scale
3. **No transform composition** - Just add parent positions
4. **Single source of truth** - Position only in FrameNode.layout
5. **Delete aggressively** - If it's not needed, it goes

## Testing Strategy

- Create tests that verify current behavior
- Implement new system alongside old
- Verify identical results
- Delete old system
- No migration path needed - this is pre-release software

## Success Metrics

- 500+ lines of code deleted
- 75% reduction in transform-related memory usage
- 5x faster coordinate transformations
- Zero matrix operations in the codebase
- All tests passing with simpler implementation