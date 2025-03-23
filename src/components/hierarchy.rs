use crate::LunaEntityId;
use std::collections::HashMap;

/// Component that manages parent-child relationships between entities
pub struct HierarchyComponent {
    /// Map of entity IDs to their parent
    parents: HashMap<LunaEntityId, LunaEntityId>,
    /// Map of entity IDs to their children
    children: HashMap<LunaEntityId, Vec<LunaEntityId>>,
}

impl HierarchyComponent {
    pub fn new() -> Self {
        HierarchyComponent {
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }

    /// Adds a child to a parent entity
    pub fn add_child(&mut self, parent: LunaEntityId, child: LunaEntityId) {
        // Remove child from its current parent if it has one
        if let Some(old_parent) = self.parents.get(&child) {
            if let Some(old_siblings) = self.children.get_mut(old_parent) {
                old_siblings.retain(|&sibling| sibling != child);
            }
        }

        // Set new parent-child relationship
        self.parents.insert(child, parent);
        self.children
            .entry(parent)
            .or_insert_with(Vec::new)
            .push(child);
    }

    /// Removes a child from its parent
    pub fn remove_child(&mut self, child: LunaEntityId) {
        if let Some(parent) = self.parents.remove(&child) {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|&sibling| sibling != child);
            }
        }
    }

    /// Gets the parent of an entity
    pub fn get_parent(&self, entity: LunaEntityId) -> Option<LunaEntityId> {
        self.parents.get(&entity).copied()
    }

    /// Gets the children of an entity
    pub fn get_children(&self, entity: LunaEntityId) -> Option<&Vec<LunaEntityId>> {
        self.children.get(&entity)
    }

    /// Gets the full chain of parents for an entity, from immediate parent to root
    pub fn get_parent_chain(&self, entity: LunaEntityId) -> Vec<LunaEntityId> {
        let mut chain = Vec::new();
        let mut current = entity;

        while let Some(parent) = self.get_parent(current) {
            chain.push(parent);
            current = parent;
        }

        chain
    }

    /// Checks if an entity is a descendant of another entity
    pub fn is_descendant_of(&self, entity: LunaEntityId, ancestor: LunaEntityId) -> bool {
        let mut current = entity;
        while let Some(parent) = self.get_parent(current) {
            if parent == ancestor {
                return true;
            }
            current = parent;
        }
        false
    }

    /// Removes an entity completely (both as a parent and as a child)
    pub fn remove(&mut self, entity: LunaEntityId) {
        // Remove as a child from its parent
        self.remove_child(entity);
        
        // Remove its children (they become parentless)
        if let Some(children) = self.children.remove(&entity) {
            for child in children {
                self.parents.remove(&child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_child_relationship() {
        let mut hierarchy = HierarchyComponent::new();
        let parent = LunaEntityId::from(1);
        let child = LunaEntityId::from(2);

        hierarchy.add_child(parent, child);

        assert_eq!(hierarchy.get_parent(child), Some(parent));
        assert_eq!(hierarchy.get_children(parent).unwrap().len(), 1);
        assert_eq!(hierarchy.get_children(parent).unwrap()[0], child);
    }

    #[test]
    fn test_reparenting() {
        let mut hierarchy = HierarchyComponent::new();
        let parent1 = LunaEntityId::from(1);
        let parent2 = LunaEntityId::from(2);
        let child = LunaEntityId::from(3);

        hierarchy.add_child(parent1, child);
        hierarchy.add_child(parent2, child);

        assert_eq!(hierarchy.get_parent(child), Some(parent2));
        assert!(hierarchy.get_children(parent1).unwrap().is_empty());
        assert_eq!(hierarchy.get_children(parent2).unwrap().len(), 1);
    }

    #[test]
    fn test_parent_chain() {
        let mut hierarchy = HierarchyComponent::new();
        let root = LunaEntityId::from(1);
        let middle = LunaEntityId::from(2);
        let leaf = LunaEntityId::from(3);

        hierarchy.add_child(root, middle);
        hierarchy.add_child(middle, leaf);

        let chain = hierarchy.get_parent_chain(leaf);
        assert_eq!(chain, vec![middle, root]);
    }
}
