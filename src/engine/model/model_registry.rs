use crate::engine::model::model::Model;

pub struct ModelRegistry {
    models: Vec<Model>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
        }
    }

    // Does not suport remove/un-register for now
    pub fn register(&mut self, model: Model) -> usize {
        // Register the model, return the index as ID for ue in Renderable component
        self.models.push(model);
        self.models.len() - 1
    }

    pub fn get(&self, id: usize) -> Option<&Model> {
        self.models.get(id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Model> {
        self.models.get_mut(id)
    }

    pub fn len(&self) -> usize {
        self.models.len()
    }
}

#[cfg(test)]
mod tests {
    // Model requires GPU resources so we test registry logic with a
    // lightweight stand-in rather than constructing real Model values.
    struct FakeRegistry {
        items: Vec<u32>,
    }

    impl FakeRegistry {
        fn new() -> Self { Self { items: Vec::new() } }
        fn register(&mut self, item: u32) -> usize {
            self.items.push(item);
            self.items.len() - 1
        }
        fn get(&self, id: usize) -> Option<&u32> { self.items.get(id) }
        fn get_mut(&mut self, id: usize) -> Option<&mut u32> { self.items.get_mut(id) }
        fn len(&self) -> usize { self.items.len() }
    }

    #[test]
    fn first_registered_model_gets_id_zero() {
        let mut reg = FakeRegistry::new();
        let id = reg.register(42);
        assert_eq!(id, 0);
    }

    #[test]
    fn each_registration_gets_sequential_id() {
        let mut reg = FakeRegistry::new();
        let a = reg.register(1);
        let b = reg.register(2);
        let c = reg.register(3);
        assert_eq!((a, b, c), (0, 1, 2));
    }

    #[test]
    fn get_returns_registered_value() {
        let mut reg = FakeRegistry::new();
        let id = reg.register(99);
        assert_eq!(reg.get(id), Some(&99));
    }

    #[test]
    fn get_out_of_bounds_returns_none() {
        let reg = FakeRegistry::new();
        assert!(reg.get(0).is_none());
        assert!(reg.get(999).is_none());
    }

    #[test]
    fn get_mut_allows_mutation() {
        let mut reg = FakeRegistry::new();
        let id = reg.register(1);
        *reg.get_mut(id).unwrap() = 100;
        assert_eq!(reg.get(id), Some(&100));
    }

    #[test]
    fn len_reflects_registration_count() {
        let mut reg = FakeRegistry::new();
        assert_eq!(reg.len(), 0);
        reg.register(1);
        reg.register(2);
        assert_eq!(reg.len(), 2);
    }
}
