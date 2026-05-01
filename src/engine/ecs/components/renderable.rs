pub struct Renderable {
    pub model_id: usize,
}

impl Renderable {
    pub fn new(model_id: usize) -> Self {
        Self { model_id }
    }
}