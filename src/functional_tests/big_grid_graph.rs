use std::time::SystemTime;

use crate::{networks::{edge::Edge, graph::Graph, node::Node, pathfinding::a_star::a_star_search}, rendering::flat_ui::rendering::draw_network};

use super::scenario::Scenario;

pub struct BigGridGraphScenario {
    pub graph: Graph,
    pub found_path: Vec<i32>,
}

impl Scenario for BigGridGraphScenario {
    ///Creates a graph with 10000 nodes and edges connecting them in a grid pattern (100x100)
    fn init() -> BigGridGraphScenario {
        let mut scenario = BigGridGraphScenario {
            graph: Graph::new(),
            found_path: Vec::new(),
        };
        println!("Initalized graph with size of {} bytes", size_of_val(&*scenario.graph.nodes));
        println!("  and edges array size of {} bytes", size_of_val(&*scenario.graph.edges));

        const POSITION_SCALE: f32 = 5.0;
        const PADDING: f32 = 40.0;
        const GRID_WIDTH: i32 = 100;

        let graph_creation_start_time = SystemTime::now();
        //create nodes
        for i in 0..10000 {
            let x_pos = (i % GRID_WIDTH) as f32 * POSITION_SCALE + PADDING;
            let y_pos = (i / GRID_WIDTH) as f32 * POSITION_SCALE + PADDING;
            scenario.graph.add_node(Node::new(x_pos, y_pos, i as i32));
        }
        //create edges
        for i in 0..10000 {
            // If the node is not on the right edge of the grid, add a connection to the right
            if i % GRID_WIDTH != GRID_WIDTH - 1 {
                scenario.graph.add_edge(Edge::new(i, i + 1));
            }
            // If the node is not on the bottom edge of the grid, add a connection to below
            if i < 9000 {
                scenario.graph.add_edge(Edge::new(i, i + GRID_WIDTH));
            }
        }
        println!("Graph creation took {}ms", graph_creation_start_time.elapsed().unwrap().as_millis());
        println!("  Graph has {} nodes and {} edges", scenario.graph.last_node_index, scenario.graph.last_edge_index);
        let search_start_time = SystemTime::now();
        scenario.found_path = a_star_search(9000, 29, &scenario.graph);
        println!("Pathfinding took {}ms", search_start_time.elapsed().unwrap().as_millis());

        return scenario;
    }

    fn tick(&self) {
        //This graph is too big to draw with the macroquad shapes rendering method
        ()
    }
}