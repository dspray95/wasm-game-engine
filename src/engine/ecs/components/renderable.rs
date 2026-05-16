pub struct Renderable {
    pub model_id: usize,
    pub visible: bool,
}

impl Renderable {
    pub fn new(model_id: usize) -> Self {
        Self {
            model_id,
            visible: true,
        }
    }
}
