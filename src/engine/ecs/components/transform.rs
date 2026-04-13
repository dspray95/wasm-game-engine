use cgmath::{ Deg, Matrix3, Matrix4, Quaternion, Rotation3, Vector3 };

use crate::engine::instance::InstanceRaw;

pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from_angle_y(Deg(0.0)),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Vector3::new(x, y, z);
        self
    }

    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = Vector3::new(x, y, z);
        self
    }

    pub fn with_rotation(mut self, rotation: Quaternion<f32>) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (
                Matrix4::from_translation(self.position) *
                Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z) *
                Matrix4::from(self.rotation)
            ).into(),
            normal: Matrix3::from(self.rotation).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{ InnerSpace, Zero };

    #[test]
    fn default_transform_has_zero_position() {
        let t = Transform::new();
        assert_eq!(t.position, Vector3::zero());
    }

    #[test]
    fn default_transform_has_unit_scale() {
        let t = Transform::new();
        assert_eq!(t.scale, Vector3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn with_position_sets_position() {
        let t = Transform::new().with_position(1.0, 2.0, 3.0);
        assert_eq!(t.position, Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn with_scale_sets_scale() {
        let t = Transform::new().with_scale(2.0, 3.0, 4.0);
        assert_eq!(t.scale, Vector3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn builder_chain_applies_all_values() {
        let t = Transform::new()
            .with_position(5.0, 0.0, 0.0)
            .with_scale(2.0, 2.0, 2.0);
        assert_eq!(t.position.x, 5.0);
        assert_eq!(t.scale.x, 2.0);
    }

    #[test]
    fn to_raw_translation_appears_in_last_column() {
        let t = Transform::new().with_position(3.0, 5.0, 7.0);
        let raw = t.to_raw();
        // Column-major: last column is translation
        assert_eq!(raw.model[3][0], 3.0);
        assert_eq!(raw.model[3][1], 5.0);
        assert_eq!(raw.model[3][2], 7.0);
    }

    #[test]
    fn to_raw_scale_appears_on_diagonal() {
        let t = Transform::new().with_scale(2.0, 3.0, 4.0);
        let raw = t.to_raw();
        assert!((raw.model[0][0] - 2.0).abs() < 1e-5);
        assert!((raw.model[1][1] - 3.0).abs() < 1e-5);
        assert!((raw.model[2][2] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn to_raw_normal_matrix_rows_are_unit_length() {
        let t = Transform::new()
            .with_rotation(Quaternion::from_angle_y(Deg(45.0)));
        let raw = t.to_raw();
        for row in &raw.normal {
            let len = Vector3::from(*row).magnitude();
            assert!((len - 1.0).abs() < 1e-5);
        }
    }
}
