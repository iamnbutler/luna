use crate::prelude::*;

/// Creates a new 2D vector.
pub fn vec2(x: f32, y: f32) -> Vector2D {
    Vector2D { x, y }
}

/// Represents a vector in 2D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl From<(f32, f32)> for Vector2D {
    fn from(tuple: (f32, f32)) -> Self {
        Vector2D {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<Vector2D> for (f32, f32) {
    fn from(vec: Vector2D) -> Self {
        (vec.x, vec.y)
    }
}

impl From<[f32; 2]> for Vector2D {
    fn from(array: [f32; 2]) -> Self {
        Vector2D {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Vector2D> for [f32; 2] {
    fn from(vec: Vector2D) -> Self {
        [vec.x, vec.y]
    }
}

impl Default for Vector2D {
    fn default() -> Self {
        Vector2D { x: 0.0, y: 0.0 }
    }
}

/// Represents a position in world space coordinates.
///
/// World position is absolute within the entire canvas, independent of hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for WorldPosition {
    fn default() -> Self {
        WorldPosition { x: 0.0, y: 0.0 }
    }
}

impl From<(f32, f32)> for WorldPosition {
    fn from(tuple: (f32, f32)) -> Self {
        WorldPosition {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<[f32; 2]> for WorldPosition {
    fn from(array: [f32; 2]) -> Self {
        WorldPosition {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Vector2D> for WorldPosition {
    fn from(vec: Vector2D) -> Self {
        WorldPosition { x: vec.x, y: vec.y }
    }
}

/// Represents a position in local space coordinates.
///
/// Local position is relative to the parent element in the hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocalPosition {
    pub x: f32,
    pub y: f32,
}

impl Default for LocalPosition {
    fn default() -> Self {
        LocalPosition { x: 0.0, y: 0.0 }
    }
}

impl From<(f32, f32)> for LocalPosition {
    fn from(tuple: (f32, f32)) -> Self {
        LocalPosition {
            x: tuple.0,
            y: tuple.1,
        }
    }
}

impl From<[f32; 2]> for LocalPosition {
    fn from(array: [f32; 2]) -> Self {
        LocalPosition {
            x: array[0],
            y: array[1],
        }
    }
}

impl From<Vector2D> for LocalPosition {
    fn from(vec: Vector2D) -> Self {
        LocalPosition { x: vec.x, y: vec.y }
    }
}

/// Represents a local transform, containing position, scale, and rotation relative to the parent.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LocalTransform {
    pub position: LocalPosition,
    pub scale: Vector2D,
    pub rotation: f32,
}

/// Represents a world transform, containing absolute position, scale, and rotation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WorldTransform {
    pub position: WorldPosition,
    pub scale: Vector2D,
    pub rotation: f32,
}

/// An unrotated, rectangular bounding box (AABB) whose edges are parallel to the coordinate axes.
///
/// Used for efficient collision detection and spatial partitioning.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    min: Vector2D,
    max: Vector2D,
}

impl BoundingBox {
    pub fn new(min: Vector2D, max: Vector2D) -> Self {
        BoundingBox { min, max }
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn half_width(&self) -> f32 {
        self.width() / 2.0
    }

    pub fn half_height(&self) -> f32 {
        self.height() / 2.0
    }

    pub fn center(&self) -> Vector2D {
        vec2(
            self.min.x + self.width() / 2.0,
            self.min.y + self.height() / 2.0,
        )
    }
}

/// Component that handles transformation data and operations for entities
pub struct TransformComponent {
    /// Map of entity IDs to their local transforms
    transforms: HashMap<LunaEntityId, LocalTransform>,
    /// Cache of computed world transforms
    world_transform_cache: HashMap<LunaEntityId, WorldTransform>,
}

impl TransformComponent {
    pub fn new() -> Self {
        TransformComponent {
            transforms: HashMap::new(),
            world_transform_cache: HashMap::new(),
        }
    }

    /// Sets the local transform for an entity
    pub fn set_transform(&mut self, entity: LunaEntityId, transform: LocalTransform) {
        self.transforms.insert(entity, transform);
        // Invalidate cached world transform since local changed
        self.world_transform_cache.remove(&entity);
    }

    /// Gets the local transform for an entity
    pub fn get_transform(&self, entity: LunaEntityId) -> Option<&LocalTransform> {
        self.transforms.get(&entity)
    }

    /// Computes the world transform for an entity given its parent chain
    pub fn compute_world_transform(
        &self,
        entity: LunaEntityId,
        parent_chain: &[LunaEntityId],
    ) -> Option<WorldTransform> {
        // Check cache first
        if let Some(cached) = self.world_transform_cache.get(&entity) {
            return Some(cached.clone());
        }

        // Get local transform
        let local = self.transforms.get(&entity)?;

        // Start with entity's local transform
        let mut world = WorldTransform {
            position: WorldPosition {
                x: local.position.x,
                y: local.position.y,
            },
            scale: local.scale,
            rotation: local.rotation,
        };

        // Compose with parent transforms
        for parent_id in parent_chain.iter().rev() {
            if let Some(parent_transform) = self.transforms.get(parent_id) {
                // Apply parent transform
                // Position
                world.position.x =
                    parent_transform.position.x + (world.position.x * parent_transform.scale.x);
                world.position.y =
                    parent_transform.position.y + (world.position.y * parent_transform.scale.y);

                // Scale
                world.scale.x *= parent_transform.scale.x;
                world.scale.y *= parent_transform.scale.y;

                // Rotation
                world.rotation += parent_transform.rotation;
            }
        }

        // Cache the computed world transform
        self.world_transform_cache.insert(entity, world.clone());

        Some(world)
    }

    /// Converts a point from local space to world space
    pub fn local_to_world(
        &self,
        local_point: LocalPosition,
        entity: LunaEntityId,
        parent_chain: &[LunaEntityId],
    ) -> Option<WorldPosition> {
        let world_transform = self.compute_world_transform(entity, parent_chain)?;

        Some(WorldPosition {
            x: world_transform.position.x + (local_point.x * world_transform.scale.x),
            y: world_transform.position.y + (local_point.y * world_transform.scale.y),
        })
    }

    /// Converts a point from world space to local space
    pub fn world_to_local(
        &self,
        world_point: WorldPosition,
        entity: LunaEntityId,
        parent_chain: &[LunaEntityId],
    ) -> Option<LocalPosition> {
        let world_transform = self.compute_world_transform(entity, parent_chain)?;

        Some(LocalPosition {
            x: (world_point.x - world_transform.position.x) / world_transform.scale.x,
            y: (world_point.y - world_transform.position.y) / world_transform.scale.y,
        })
    }

    /// Invalidates the cached world transform for an entity
    pub fn invalidate_cache(&mut self, entity: LunaEntityId) {
        self.world_transform_cache.remove(&entity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_transform() {
        let mut transform_component = TransformComponent::new();
        let entity = LunaEntityId::from(1);

        let local = LocalTransform {
            position: LocalPosition { x: 10.0, y: 20.0 },
            scale: Vector2D { x: 2.0, y: 2.0 },
            rotation: 0.0,
        };

        transform_component.set_transform(entity, local);

        let retrieved = transform_component.get_transform(entity).unwrap();
        assert_eq!(retrieved.position.x, 10.0);
        assert_eq!(retrieved.position.y, 20.0);
        assert_eq!(retrieved.scale.x, 2.0);
        assert_eq!(retrieved.scale.y, 2.0);
    }

    #[test]
    fn test_world_transform_computation() {
        let mut transform_component = TransformComponent::new();

        // Create a simple parent-child hierarchy
        let parent = LunaEntityId::from(1);
        let child = LunaEntityId::from(2);

        // Parent at (10,10) with scale 2
        transform_component.set_transform(
            parent,
            LocalTransform {
                position: LocalPosition { x: 10.0, y: 10.0 },
                scale: Vector2D { x: 2.0, y: 2.0 },
                rotation: 0.0,
            },
        );

        // Child at (5,5) with scale 1.5
        transform_component.set_transform(
            child,
            LocalTransform {
                position: LocalPosition { x: 5.0, y: 5.0 },
                scale: Vector2D { x: 1.5, y: 1.5 },
                rotation: 0.0,
            },
        );

        // Compute world transform for child
        let world = transform_component
            .compute_world_transform(child, &[parent])
            .unwrap();

        // Child position should be: parent_pos + (child_pos * parent_scale)
        assert_eq!(world.position.x, 20.0); // 10 + (5 * 2)
        assert_eq!(world.position.y, 20.0); // 10 + (5 * 2)

        // Scales should multiply
        assert_eq!(world.scale.x, 3.0); // 2 * 1.5
        assert_eq!(world.scale.y, 3.0); // 2 * 1.5
    }
}
