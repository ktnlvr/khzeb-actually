use core::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity(u64);

impl Entity {
    pub fn new(generation: u32, index: u32) -> Self {
        Self(((generation as u64) << 32) | index as u64)
    }

    pub fn generation(&self) -> u32 {
        ((self.0 >> 32) & 0xFFFFFFFF) as u32
    }

    pub fn index(&self) -> u32 {
        (self.0 & 0xFFFFFFFF) as u32
    }

    pub fn decouple(&self) -> (u32, u32) {
        (self.generation(), self.index())
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<Entity gen={} idx={}>",
            self.generation(),
            self.index()
        ))
    }
}

pub struct World {
    // The list of the slots for entities and their generations
    entity_list: Vec<(u32,)>,
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
                self.entity_list[vacant_idx as usize] = (generation,);
                Entity::new(generation, vacant_idx)
            }
            None => {
                let last_idx = self.entity_list.len() as u32;
                self.entity_list.push((0,));
                Entity::new(0, last_idx)
            }
        }
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
}
