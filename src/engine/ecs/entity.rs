use crate::engine::ecs::world::World;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Entity {
    pub id: u32,
    generation: u32,
}

pub struct EntityAllocator {
    generations: Vec<u32>,
    free_ids: Vec<u32>,
    next_id: u32,
}

impl EntityAllocator {
    pub fn default() -> Self {
        Self::new()
    }

    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_ids: Vec::new(),
            next_id: 0,
        }
    }

    pub fn spawn(&mut self) -> Entity {
        if let Some(id) = self.free_ids.pop() {
            Entity {
                id,
                generation: self.generations[id as usize],
            }
        } else {
            let id = self.next_id;
            self.next_id += 1;
            self.generations.push(0);
            Entity {
                id,
                generation: 0,
            }
        }
    }

    pub fn despawn(&mut self, entity: &Entity) {
        self.free_ids.push(entity.id);
        self.generations[entity.id as usize] += 1;
    }

    pub fn is_alive(&self, entity: &Entity) -> bool {
        if (entity.id as usize) >= self.generations.len() {
            return false;
        }

        entity.generation == self.generations[entity.id as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_returns_unique_entities() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.spawn();
        let b = alloc.spawn();
        assert_ne!(a, b);
    }

    #[test]
    fn spawned_entity_is_alive() {
        let mut alloc = EntityAllocator::new();
        let e = alloc.spawn();
        assert!(alloc.is_alive(&e));
    }

    #[test]
    fn despawned_entity_is_not_alive() {
        let mut alloc = EntityAllocator::new();
        let e = alloc.spawn();
        alloc.despawn(&e);
        assert!(!alloc.is_alive(&e));
    }

    #[test]
    fn recycled_id_has_new_generation() {
        let mut alloc = EntityAllocator::new();
        let a = alloc.spawn();
        alloc.despawn(&a);
        let b = alloc.spawn();
        // same slot reused, but stale handle is dead
        assert!(!alloc.is_alive(&a));
        assert!(alloc.is_alive(&b));
    }

    #[test]
    fn is_alive_with_out_of_range_id_returns_false() {
        let alloc = EntityAllocator::new();
        let ghost = Entity { id: 999, generation: 0 };
        assert!(!alloc.is_alive(&ghost));
    }
}
