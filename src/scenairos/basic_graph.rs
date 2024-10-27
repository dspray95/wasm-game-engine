use crate::networks::{edge::Edge, graph::Graph, node::Node, pathfinding::a_star::a_star_search};

use super::scenario::Scenario;

pub struct BasicGraphScenario {
    graph: Graph,
    found_path: Vec<i32>,
}

impl Scenario for BasicGraphScenario {
    fn init() -> BasicGraphScenario {
        let mut scenario = BasicGraphScenario {
            graph: Graph::new(),
            found_path: Vec::new(),
        };
        println!(
            "Initalized graph with nodes array size of {} bytes",
            size_of_val(&*scenario.graph.nodes)
        );
        println!(
            "  and edges array size of {} bytes",
            size_of_val(&*scenario.graph.edges)
        );

        const POSITIONS: [(f32, f32); 10] = [
            (50.0, 299.0),
            (190.0, 255.0),
            (260.0, 154.0),
            (304.0, 190.0),
            (380.0, 205.0),
            (318.0, 442.0),
            (142.0, 363.0),
            (253.0, 232.0),
            (140.0, 200.0),
            (96.0, 173.0),
        ];
        for i in 0..10 {
            let node = Node::new(POSITIONS[i].0, POSITIONS[i].1, i as i32);
            scenario.graph.add_node(node);
            if i != 0 {
                scenario.graph.add_edge(Edge::new((i as i32) - 1, i as i32));
            }
        }
        scenario.graph.add_edge(Edge::new(1, 7));
        scenario.graph.add_edge(Edge::new(0, 6));
        scenario.graph.add_edge(Edge::new(5, 3));

        scenario.found_path = a_star_search(0, 4, &scenario.graph);

        println!("Path:");
        for node_index in scenario.found_path.iter() {
            println!("  ↳ {}", node_index);
        }
        return scenario;
    }

    fn tick(&self) {}
}
