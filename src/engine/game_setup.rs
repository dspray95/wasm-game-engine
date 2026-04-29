use crate::engine::{
    assets::server::AssetServer,
    ecs::{
        component_registry::ComponentRegistry,
        system::{ SystemContext, SystemSchedule },
        world::World,
    },
    state::context::GpuContext,
    ui::ui_registry::UIRegistry,
};

pub trait GameSetup {
    type Action: serde::de::DeserializeOwned + std::hash::Hash + Eq + 'static;

    /// Sets up the game's systems schedule for the ecs
    /// Systems here run after engine systems and before the rendering systems
    fn setup_ecs(&self, _schedule: &mut SystemSchedule) {}

    /// Any initial UI setup
    fn setup_ui(&self, _ui_registry: &mut UIRegistry) {}

    /// Register game specific components for RON deserialisation
    fn register_components(&self, _component_registry: &mut ComponentRegistry) {}

    /// Compile-time RON content for this scenes entity layout
    fn world_ron(&self) -> Option<&'static str> {
        None
    }

    /// Compile time RON content for input bindings
    fn bindings_ron(&self) -> Option<&'static str> {
        None
    }

    fn load_assets(
        &self,
        _gpu_context: &GpuContext,
        _asset_server: &mut AssetServer,
        _world: &mut World
    ) {}

    /// One-time hook for game-specific setup that doesn't fit elsewhere
    /// (loading game-specific GPU assets, setting up game resources).
    fn setup(&self, _world: &mut World, _system_context: &mut SystemContext) {}
}
