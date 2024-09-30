use aoee_rust::networks::graph::Graph;
use aoee_rust::networks::node::Node;
use aoee_rust::networks::edge::Edge;
use aoee_rust::networks::pathfinding::a_star::a_star_search;
use aoee_rust::rendering::flat_ui::rendering::draw_network;
use macroquad::{color::Color, window::{clear_background, next_frame}};

const NOT_WHITE: Color = Color::new(251.0,250.0,250.0, 1.0);


#[macroquad::main("A* Pathfinding")]
async fn main() {
    let mut graph = Graph::new();
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
        (96.0, 173.0)
    ];
    for i in 0..10 {
        let node = Node::new(POSITIONS[i].0, POSITIONS[i].1, i as i32);
        graph.add_node(node);
        if i != 0 {
            graph.add_edge(Edge::new((i as i32)-1, i as i32));
        }
    }
    graph.add_edge(Edge::new(1, 7));
    graph.add_edge(Edge::new(0, 6));
    graph.add_edge(Edge::new(5, 3));

    let found_path = a_star_search(0, 4, &graph);
    println!("Path:");
    for node_index in found_path.iter() {
        println!("  ↳ {}", node_index);
    }
    
    loop {

        clear_background(NOT_WHITE);
        draw_network(&graph, &found_path).await;
        next_frame().await
    }

}
