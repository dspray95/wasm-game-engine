use cgmath::Vector3;

pub struct EnemyKilledEvent {
    pub origin: Vector3<f32>,
    /// Whether this kill should spawn a diegetic "+score" popup. Bomb kills
    /// set this `false` — the bomb shows its own aggregated HUD total instead
    /// of a swarm of world-space pops.
    pub show_popup: bool,
}
