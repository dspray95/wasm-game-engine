pub trait Scenario {
    fn init() -> Self;
    fn tick(&self);
}