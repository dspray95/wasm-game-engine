use crate::game::cube::Cube;

use super::{ app::App, light::LightUniform, model::model::{ self, Model } };

pub struct Scene {
    models: Vec<Model>,
    global_illumination: LightUniform,
    objects: Vec<Cube>,
}

impl Scene {
    pub fn new(app: &mut App, objects: Vec<Cube>) -> Self {
        Self {
            models: vec!(),
            global_illumination: LightUniform {
                position: [-5.0, 10.0, 0.0],
                _padding: 0,
                color: [0.443, 0.941, 0.922],
                __padding: 0,
            },
            objects,
        }
    }
}
