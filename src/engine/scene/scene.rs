use crate::engine::ecs::system::SystemSchedule;

pub trait Scene {
    fn setup_ecs(&self, _schedule: &mut SystemSchedule) {}
}
