use std::any::Any;

use micromap::Map;

use crate::utils::Name;

use super::{component::Component, entity::Entity};

pub struct World {
    // The list of the slots for entities and their generations
    // TODO(ktnlvr): Use an archetype-based component system
    entity_list: Vec<(u32, Map<Name, Box<dyn Any>, 8>)>,
    // The list of vacant IDs
    free_list: Vec<u32>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_list: vec![],
            free_list: vec![],
        }
    }

    pub fn spawn(&mut self) -> Entity {
        match self.free_list.pop() {
            Some(vacant_idx) => {
                let (generation, _) = self.entity_list[vacant_idx as usize].0.overflowing_add(1);
                self.entity_list[vacant_idx as usize] = (generation, Default::default());
                Entity::new(generation, vacant_idx)
            }
            None => {
                let last_idx = self.entity_list.len() as u32;
                self.entity_list.push((0, Default::default()));
                Entity::new(0, last_idx)
            }
        }
    }

    pub fn add_component<C: Component + 'static>(&mut self, to: Entity, component: C) {
        let index = to.index() as usize;

        let Some((_, components)) = self.entity_list.get_mut(index) else {
            return;
        };

        components.insert(component.name(), Box::new(component));
    }

    pub fn has_component(&self, entt: Entity, name: Name) -> bool {
        self.entity_list
            .get(entt.index() as usize)
            .filter(|(_, cs)| cs.keys().find(|k| **k == name).is_some())
            .is_some()
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.free_list.push(entity.index());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_reuse() {
        let mut world = World::new();

        let a = world.spawn();
        let (generation, index) = a.decouple();

        world.despawn(a);

        let b = world.spawn();
        let (next_generation, next_index) = b.decouple();

        assert_ne!(generation, next_generation);
        assert_eq!(index, next_index);
    }

    #[test]
    pub fn test_components() {
        let mut world = World::new();

        let label_a = Name::new("A");
        let label_b = Name::new("B");
        let label_shared = Name::new("Shared");

        let a = world.spawn();
        let b = world.spawn();

        world.add_component(a, label_a.clone());
        world.add_component(b, label_b.clone());

        assert!(world.has_component(a, label_a.clone()));
        assert!(world.has_component(b, label_b.clone()));

        assert!(!world.has_component(a, label_b));
        assert!(!world.has_component(b, label_a));

        world.add_component(a, label_shared.clone());
        world.add_component(b, label_shared.clone());

        assert!(world.has_component(a, label_shared.clone()));
        assert!(world.has_component(b, label_shared));
    }
}
