use macroquad::prelude::*;

use crate::networks::{graph::Graph, node::{get_distance_between_nodes, get_midpoint_between_nodes, Node}};

pub async fn draw_network(graph: Graph, path: Vec<i32>) {
    const SCALE: f32 = 1.0;
    const PADDING: f32 = 0.0;
    const NOT_WHITE: Color = Color::new(251.0,250.0,250.0, 1.0);
    const NODE_IN_PATH_COLOR: Color = GREEN;

    loop {
        clear_background(NOT_WHITE);

        for edge in graph.edges {
            if !edge.is_active() {break;}
            let node_a: Node = graph.nodes[edge.source_index as usize];
            let node_b: Node = graph.nodes[edge.destination_index as usize];
            let x_1 = (node_a.x_pos * SCALE) + PADDING;
            let y_1 = (node_a.y_pos * SCALE) + PADDING;
            let x_2 = (node_b.x_pos * SCALE) + PADDING;
            let y_2 = (node_b.y_pos * SCALE) + PADDING;

            let midpoint = get_midpoint_between_nodes(node_a, node_b);
            let distance = get_distance_between_nodes(node_a, node_b);
            let label = format!("{:.0}px", distance);
            draw_line(x_1, y_1, x_2, y_2, 2.0, GRAY);
            draw_text(&label, midpoint.x, midpoint.y, 12.0, GREEN);

        }

        for node in graph.nodes {
            if !node.is_active() {break;}
            let x_pos = (node.x_pos * SCALE) + PADDING;
            let y_pos = (node.y_pos * SCALE) + PADDING;

            let node_color = if path.contains(&node.index_of) {NODE_IN_PATH_COLOR} else {BLUE};
            draw_circle(x_pos, y_pos, 5.0, node_color);
            let label = format!("N{}", node.index_of);
            draw_text(&label, x_pos - 8.0, y_pos + 20.0, 18.0, BLUE);
        }

       
        next_frame().await
    }
}

