use crate::engine::{ ecs::world::World, model::model_registry::ModelRegistry };

pub struct SystemContext<'a> {
    pub delta_time: f32,
    // None for systems that don't need GPU access (most of them)
    // Only render_sync_system needs these
    pub queue: Option<&'a wgpu::Queue>,
    pub model_registry: Option<&'a mut ModelRegistry>,
}

impl<'a> SystemContext<'a> {
    pub fn new(
        delta_time: f32,
        queue: &'a wgpu::Queue,
        model_registry: &'a mut ModelRegistry,
    ) -> Self {
        Self {
            delta_time,
            queue: Some(queue),
            model_registry: Some(model_registry),
        }
    }
}

pub type System = fn(&mut World, &mut SystemContext);

pub struct SystemSchedule {
    systems: Vec<System>, // Ordered, e.g. `[input, ai, pathfinding, movement, resource, render_sync etc]`
}

impl SystemSchedule {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_system(&mut self, system: System) {
        self.systems.push(system);
    }

    pub fn run_all(&mut self, world: &mut World, system_context: &mut SystemContext) {
        // run_all takes &mut self and &mut World. When you call each system with world,
        // you're passing the same &mut World repeatedly through the loop. Rust will let you
        // do this because each call completes before the next one starts — the borrow is
        // released between iterations.
        // We'll need to reconsider this if we want to run systems async
        for system in &self.systems {
            system(world, system_context);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Counter(u32);

    fn increment_system(world: &mut World, _ctx: &mut SystemContext) {
        world.get_resource_mut::<Counter>().unwrap().0 += 1;
    }

    fn double_system(world: &mut World, _ctx: &mut SystemContext) {
        world.get_resource_mut::<Counter>().unwrap().0 *= 2;
    }

    fn capture_dt_system(world: &mut World, ctx: &mut SystemContext) {
        world.add_resource(ctx.delta_time);
    }

    fn make_ctx() -> SystemContext<'static> {
        SystemContext {
            delta_time: 0.016,
            queue: None,
            model_registry: None,
        }
    }

    #[test]
    fn system_runs_and_mutates_world() {
        let mut world = World::new();
        world.add_resource(Counter(0));
        let mut schedule = SystemSchedule::new();
        schedule.add_system(increment_system);
        schedule.run_all(&mut world, &mut make_ctx());
        assert_eq!(world.get_resource::<Counter>().unwrap().0, 1);
    }

    #[test]
    fn systems_run_in_order() {
        // increment then double → (0+1)*2 = 2
        // reversed would be: double then increment → (0*2)+1 = 1
        let mut world = World::new();
        world.add_resource(Counter(0));
        let mut schedule = SystemSchedule::new();
        schedule.add_system(increment_system);
        schedule.add_system(double_system);
        schedule.run_all(&mut world, &mut make_ctx());
        assert_eq!(world.get_resource::<Counter>().unwrap().0, 2);
    }

    #[test]
    fn multiple_runs_accumulate() {
        let mut world = World::new();
        world.add_resource(Counter(0));
        let mut schedule = SystemSchedule::new();
        schedule.add_system(increment_system);
        schedule.run_all(&mut world, &mut make_ctx());
        schedule.run_all(&mut world, &mut make_ctx());
        schedule.run_all(&mut world, &mut make_ctx());
        assert_eq!(world.get_resource::<Counter>().unwrap().0, 3);
    }

    #[test]
    fn empty_schedule_does_not_panic() {
        let mut world = World::new();
        let mut schedule = SystemSchedule::new();
        schedule.run_all(&mut world, &mut make_ctx());
    }

    #[test]
    fn delta_time_is_accessible_in_system() {
        let mut world = World::new();
        let mut schedule = SystemSchedule::new();
        schedule.add_system(capture_dt_system);
        let mut ctx = SystemContext { delta_time: 1.0 / 60.0, queue: None, model_registry: None };
        schedule.run_all(&mut world, &mut ctx);
        let stored = world.get_resource::<f32>().unwrap();
        assert!((stored - 1.0 / 60.0).abs() < f32::EPSILON);
    }
}
