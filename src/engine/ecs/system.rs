use crate::engine::{
    assets::server::AssetServer,
    ecs::{
        commands::commands::Commands,
        entity::EntityAllocator,
        systems::{
            camera_update_system::camera_update_system, collision_system::collision_system,
            event_swap_system::event_swap_system, hierarchy_system::hierarchy_system,
            render_sync_system::render_sync_system, toast_system::toast_system,
            velocity_system::velocity_system,
        },
        world::World,
    },
};

pub struct SystemContext<'a> {
    pub delta_time: f32,
    // None for systems that don't need GPU access (most of them)
    pub device: Option<&'a wgpu::Device>,
    pub queue: Option<&'a wgpu::Queue>,
    pub asset_server: Option<&'a mut AssetServer>,
    pub commands: &'a mut Commands,
    pub entity_allocator: &'a mut EntityAllocator,
}

impl<'a> SystemContext<'a> {
    pub fn new(
        delta_time: f32,
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        asset_server: &'a mut AssetServer,
        commands: &'a mut Commands,
        entity_allocator: &'a mut EntityAllocator,
    ) -> Self {
        Self {
            delta_time,
            device: Some(device),
            queue: Some(queue),
            asset_server: Some(asset_server),
            commands,
            entity_allocator,
        }
    }

    pub fn commands(&mut self) -> &mut Commands {
        &mut *self.commands
    }
}

pub type System = fn(&mut World, &mut SystemContext);

// Systems execute in order:
// 1. On Load ONLY - startup_systems (loading models/scene etc),
// 2. game_systems - systems that handle game specific logic e.g. `[input, ai, pathfinding, movement]`
// 3. engine_systems - systems that deal directly with the engine e.g. `[velocity, camera_update, render_sync]`
pub struct SystemSchedule {
    startup_systems: Vec<System>,
    game_systems: Vec<System>,
    engine_systems: Vec<System>,
    started: bool,
}

impl SystemSchedule {
    pub fn new() -> Self {
        Self {
            startup_systems: Vec::new(),
            game_systems: Vec::new(),
            engine_systems: vec![
                velocity_system,
                hierarchy_system,
                collision_system,
                camera_update_system,
                toast_system,
                render_sync_system,
                event_swap_system,
            ],
            started: false,
        }
    }

    /// Schedule with no pre-installed engine systems. Used in tests so we
    /// don't drag in render_sync_system (which needs a real GPU queue).
    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            startup_systems: Vec::new(),
            game_systems: Vec::new(),
            engine_systems: Vec::new(),
            started: false,
        }
    }

    pub fn add_startup(&mut self, system: System) {
        self.startup_systems.push(system);
    }

    pub fn add_game_system(&mut self, system: System) {
        self.game_systems.push(system);
    }

    /// Run a single system, then drain any commands it queued into `world`.
    /// Buffers are cleared (not dropped) before each system, so allocations
    /// stay warm across frames.
    fn run_system(world: &mut World, system_context: &mut SystemContext, system: System) {
        system_context.commands.clear();
        system(world, system_context);
        system_context
            .commands
            .apply(world, system_context.entity_allocator);
    }

    pub fn run_all(&mut self, world: &mut World, system_context: &mut SystemContext) {
        if !self.started {
            for system in &self.startup_systems {
                Self::run_system(world, system_context, *system);
            }
            self.started = true;
        }

        for game_system in &self.game_systems {
            Self::run_system(world, system_context, *game_system);
        }

        for engine_system in &self.engine_systems {
            Self::run_system(world, system_context, *engine_system);
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

    fn capture_dt_system(world: &mut World, system_context: &mut SystemContext) {
        world.add_resource(system_context.delta_time);
    }

    struct TestHarness {
        commands: Commands,
        allocator: EntityAllocator,
    }

    impl TestHarness {
        fn new() -> Self {
            Self {
                commands: Commands::new(),
                allocator: EntityAllocator::default(),
            }
        }

        fn ctx(&mut self, delta_time: f32) -> SystemContext {
            SystemContext {
                delta_time,
                device: None,
                queue: None,
                asset_server: None,
                commands: &mut self.commands,
                entity_allocator: &mut self.allocator,
            }
        }
    }

    #[test]
    fn system_runs_and_mutates_world() {
        let mut world = World::new();
        world.add_resource(Counter(0));
        let mut schedule = SystemSchedule::empty();
        schedule.add_game_system(increment_system);
        let mut harness = TestHarness::new();
        schedule.run_all(&mut world, &mut harness.ctx(0.016));
        assert_eq!(world.get_resource::<Counter>().unwrap().0, 1);
    }

    #[test]
    fn systems_run_in_order() {
        let mut world = World::new();
        world.add_resource(Counter(0));
        let mut schedule = SystemSchedule::empty();
        schedule.add_game_system(increment_system);
        schedule.add_game_system(double_system);
        let mut harness = TestHarness::new();
        schedule.run_all(&mut world, &mut harness.ctx(0.016));
        assert_eq!(world.get_resource::<Counter>().unwrap().0, 2);
    }

    #[test]
    fn multiple_runs_accumulate() {
        let mut world = World::new();
        world.add_resource(Counter(0));
        let mut schedule = SystemSchedule::empty();
        schedule.add_game_system(increment_system);
        let mut harness = TestHarness::new();
        schedule.run_all(&mut world, &mut harness.ctx(0.016));
        schedule.run_all(&mut world, &mut harness.ctx(0.016));
        schedule.run_all(&mut world, &mut harness.ctx(0.016));
        assert_eq!(world.get_resource::<Counter>().unwrap().0, 3);
    }

    #[test]
    fn empty_schedule_does_not_panic() {
        let mut world = World::new();
        let mut schedule = SystemSchedule::empty();
        let mut harness = TestHarness::new();
        schedule.run_all(&mut world, &mut harness.ctx(0.016));
    }

    #[test]
    fn delta_time_is_accessible_in_system() {
        let mut world = World::new();
        let mut schedule = SystemSchedule::empty();
        schedule.add_game_system(capture_dt_system);
        let mut harness = TestHarness::new();
        schedule.run_all(&mut world, &mut harness.ctx(1.0 / 60.0));
        let stored = world.get_resource::<f32>().unwrap();
        assert!((stored - 1.0 / 60.0).abs() < f32::EPSILON);
    }
}
