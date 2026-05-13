#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ScoreType {
    EnemyKilled,
    Progress,
    Pickup,
}
pub struct ScoreEvent {
    pub score_type: ScoreType,
}
