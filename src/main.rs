use aoee_rust::scenairos::{basic_graph::BasicGraphScenario, grid_graph::GridGraphScenario, scenario::Scenario};
use macroquad::{color::Color, window::{clear_background, next_frame}};

const NOT_WHITE: Color = Color::new(251.0,250.0,250.0, 1.0);

#[macroquad::main("A* Pathfinding")]
async fn main() {
    let scenario = GridGraphScenario::init();
    loop {
        clear_background(NOT_WHITE);
        scenario.tick();
        next_frame().await
    }

}
