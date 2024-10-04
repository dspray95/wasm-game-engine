use crate::{networks::{edge::Edge, graph::Graph, node::Node, pathfinding::a_star::a_star_search}, rendering::flat_ui::rendering::draw_network};

use super::scenario::Scenario;

pub struct GridGraphScenario {
    pub graph: Graph,
    pub found_path: Vec<i32>,
}

impl Scenario for GridGraphScenario {
    fn init() -> GridGraphScenario {
        let mut scenario = GridGraphScenario {
            graph: Graph::new(),
            found_path: Vec::new(),
        };
        println!("Initalized graph with size of {} bytes", size_of_val(&*scenario.graph.nodes));
        println!("  and edges array size of {} bytes", size_of_val(&*scenario.graph.edges));

        const SCALE: f32 = 50.0;
        const PADDING: f32 = 50.0;
        const GRID_WIDTH: i32 = 10;

        //create nodes
        for i in 0..100 {
            let x_pos = (i % GRID_WIDTH) as f32 * SCALE + PADDING;
            let y_pos = (i / GRID_WIDTH) as f32 * SCALE + PADDING;
            scenario.graph.add_node(Node::new(x_pos, y_pos, i as i32));
        }
        //create edges
        for i in 0..100 {
            // If the node is not on the right edge of the grid, add a connection to the right
            if i % GRID_WIDTH != GRID_WIDTH - 1 {
                scenario.graph.add_edge(Edge::new(i, i + 1));
            }
            // If the node is not on the bottom edge of the grid, add a connection to below
            if i < 90 {
                scenario.graph.add_edge(Edge::new(i, i + GRID_WIDTH));
            }
        }

        scenario.found_path = a_star_search(90, 29, &scenario.graph);

        println!("Path:");
        for node_index in scenario.found_path.iter() {
            println!("  ↳ {}", node_index);
        } 

        return scenario;
    }

    fn tick(&self) {
        draw_network(&self.graph, &self.found_path);
    }
}