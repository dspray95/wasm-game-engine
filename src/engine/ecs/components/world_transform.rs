use cgmath::{Deg, Matrix3, Matrix4, Quaternion, Rotation3, Vector3};

use crate::engine::{
    ecs::components::transform::Transform, instance::InstanceRaw,
};

/// World-space transform, written each frame by `hierarchy_system` by composing
/// each entity's local `Transform` with its parent chain. Read by anything that
/// needs world-space data (renderer, collision, screen projection, camera).
///
/// Never authored directly — to move an entity, write its local `Transform`
/// and let the hierarchy system propagate.
#[derive(PartialEq, Clone)]
pub struct WorldTransform {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl WorldTransform {
    pub fn identity() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::from_angle_y(Deg(0.0)),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn from_local(local: &Transform) -> Self {
        Self {
            position: local.position,
            rotation: local.rotation,
            scale: local.scale,
        }
    }

    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::from_translation(self.position)
                * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
                * Matrix4::from(self.rotation))
            .into(),
            normal: Matrix3::from(self.rotation).into(),
        }
    }
}
