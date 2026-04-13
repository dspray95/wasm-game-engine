pub struct SparseSet<T> {
    sparse: Vec<Option<usize>>,
    dense: Vec<u32>,
    data: Vec<T>,
}

impl<T> SparseSet<T> {
    pub fn default() -> Self {
        Self::new()
    }

    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn insert(&mut self, entity_id: u32, value: T) {
        let id = entity_id as usize;

        // Make sure our sparse array is big enough
        if id >= self.sparse.len() {
            self.sparse.resize(id + 1, None);
        }

        // Check if the entity already exists in this set
        match self.sparse[id] {
            Some(dense_index) => {
                // entity already has the component, so just overwrite
                self.data[dense_index] = value;
            }
            None => {
                // Create new component
                // Record the data's index (end of the dense array)
                self.sparse[id] = Some(self.dense.len());

                // Push entity id and data to the end of the arrays
                self.dense.push(entity_id);
                self.data.push(value);
            }
        }
    }

    pub fn remove(&mut self, entity_id: u32) {
        // Swaps the removed element with the final element (O(1)) rather than shifting every element (O(n))
        let id = entity_id as usize;

        // Nothing to remove if the entity isn't in the set
        let Some(index) = self.sparse.get(id).copied().flatten() else {
            return;
        };

        // Get entity from the end of the dense array
        let last_entity_id = *self.dense.last().unwrap() as usize;

        // Move last element to the gap vacated by the removed entity
        self.dense.swap_remove(index);
        self.data.swap_remove(index);

        // Clear removed entities sparse entry
        self.sparse[id] = None;

        // Update the moved entity's sparse entry
        if last_entity_id != id {
            self.sparse[last_entity_id] = Some(index);
        }
    }

    pub fn get(&self, entity_id: u32) -> Option<&T> {
        let id = entity_id as usize;
        // sparse.get() returns Option<&Option<usize>>, copied() removes the reference
        // giving Option<Option<usize>>, flatten() collapses to Option<usize>
        // None meaning either out of bounds or removed.
        let dense_idx = self.sparse.get(id).copied().flatten()?;
        Some(&self.data[dense_idx])
    }

    pub fn get_mut(&mut self, entity_id: u32) -> Option<&mut T> {
        let id = entity_id as usize;
        let dense_idx = self.sparse.get(id).copied().flatten()?;
        Some(&mut self.data[dense_idx])
    }

    pub fn contains(&self, entity_id: u32) -> bool {
        self.get(entity_id).is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, &T)> {
        self.dense.iter().copied().zip(self.data.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u32, &mut T)> {
        self.dense.iter().copied().zip(self.data.iter_mut())
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserted_value_is_retrievable() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(0, 42);
        assert_eq!(set.get(0), Some(&42));
    }

    #[test]
    fn contains_returns_false_for_missing_entity() {
        let set: SparseSet<i32> = SparseSet::new();
        assert!(!set.contains(99));
    }

    #[test]
    fn insert_overwrites_existing_component() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(0, 1);
        set.insert(0, 99);
        assert_eq!(set.get(0), Some(&99));
    }

    #[test]
    fn removed_entity_is_no_longer_present() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(0, 10);
        set.remove(0);
        assert!(!set.contains(0));
        assert_eq!(set.get(0), None);
    }

    #[test]
    fn remove_nonexistent_entity_does_not_panic() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.remove(42); // should be a no-op
    }

    #[test]
    fn remove_last_entity_does_not_corrupt_set() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(0, 1);
        set.remove(0);
        assert!(!set.contains(0));
    }

    #[test]
    fn remove_middle_entity_leaves_others_intact() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(0, 10);
        set.insert(1, 20);
        set.insert(2, 30);
        set.remove(1);
        assert_eq!(set.get(0), Some(&10));
        assert_eq!(set.get(1), None);
        assert_eq!(set.get(2), Some(&30));
    }

    #[test]
    fn get_mut_allows_mutation() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(0, 1);
        *set.get_mut(0).unwrap() = 100;
        assert_eq!(set.get(0), Some(&100));
    }

    #[test]
    fn iter_yields_all_entity_value_pairs() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(3, 30);
        set.insert(7, 70);
        let mut pairs: Vec<(u32, i32)> = set
            .iter()
            .map(|(id, v)| (id, *v))
            .collect();
        pairs.sort();
        assert_eq!(pairs, vec![(3, 30), (7, 70)]);
    }

    #[test]
    fn reinsertion_after_removal_works() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(0, 1);
        set.remove(0);
        set.insert(0, 2);
        assert_eq!(set.get(0), Some(&2));
    }
}
