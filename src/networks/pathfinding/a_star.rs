use core::f32;
use std::collections::HashMap;

use crate::networks::{graph::Graph, node::{get_distance_between_nodes, Node}};

fn reconstruct_path(came_from: HashMap<i32, i32>, current: i32) -> Vec<i32> {
    let mut total_path = vec!(current);
    let mut current_node = current;
    while came_from.contains_key(&current_node) {
        current_node = came_from[&current_node];
        total_path.insert(0, current_node);
    }
    return total_path;
}

pub fn a_star_search(start_index: i32, target_index: i32, graph: &Graph) -> Vec<i32>{
    let start_node: Node = graph.get_node(start_index);
    let target_node: Node = graph.get_node(target_index);

    let mut frontier: Vec<Node> = vec!(start_node);
    let mut came_from: HashMap<i32, i32> = HashMap::new();

    let mut g_score: Vec<f32> = vec![f32::INFINITY; graph.nodes.len()];
    g_score[start_index as usize] = 0.0;

    let mut f_score: Vec<f32> = vec![f32::INFINITY; graph.nodes.len()];
    f_score[start_index as usize] = get_heuristic(start_node, target_node);

    while frontier.len() > 0 {
        // current node = the node in frontier having the lowest f_score value
        let mut lowest_f_score = f32::INFINITY;
        let mut index_of_node_with_lowest_f_score: i32 = -1;
        for i in 0..frontier.len() {
            let frontier_node = frontier[i];
            let node_f_score = f_score[frontier_node.index_of as usize];
            if node_f_score < lowest_f_score {
                lowest_f_score = node_f_score;
                index_of_node_with_lowest_f_score = i as i32;
            }
        }
        let current_node = frontier[index_of_node_with_lowest_f_score as usize];

        frontier.remove(index_of_node_with_lowest_f_score as usize);

        if current_node.index_of == target_index {
            return reconstruct_path(came_from, current_node.index_of);
        }

        for neighbour_node in graph.get_connected_nodes(current_node) {
            let distance_to_neighbour = get_distance_between_nodes(
                current_node, 
                neighbour_node
            );
            let tentative_g_score: f32 = g_score[current_node.index_of as usize] + distance_to_neighbour;
            if tentative_g_score < g_score[neighbour_node.index_of as usize] {
                came_from.insert(neighbour_node.index_of, current_node.index_of);
                g_score[neighbour_node.index_of as usize] = tentative_g_score;
                f_score[neighbour_node.index_of as usize] = tentative_g_score + get_heuristic(neighbour_node, target_node);
                if !frontier.contains(&neighbour_node) {
                    frontier.push(neighbour_node);
                }
            }
        }
    }
    panic!("No path found");
}

fn get_heuristic(node: Node, target_node: Node) -> f32 {
    get_distance_between_nodes(node, target_node)
}

#[cfg(test)]
mod tests {
    use crate::networks::edge::Edge;

    use super::*;

    fn create_test_graph() -> Graph {
        let mut graph = Graph::new();
        let node_a = Node::new(0.0, 0.0, 0);
        let node_b = Node::new(1.0, 1.0,  1);
        let node_c = Node::new(2.0, 2.0, 2);
        let node_d = Node::new(3.0, 3.0, 3 );
        let node_e = Node::new(4.0, 4.0, 4);

        graph.add_node(node_a);
        graph.add_node(node_b);
        graph.add_node(node_c);
        graph.add_node(node_d);
        graph.add_node(node_e);

        graph.add_edge(Edge::new(0, 1));
        graph.add_edge(Edge::new(1, 2));
        graph.add_edge(Edge::new(2, 3));
        graph.add_edge(Edge::new(3, 4));
        graph.add_edge(Edge::new(0, 4));

        graph
    }

    #[test]
    fn test_a_star_search_direct_path() {
        let graph = create_test_graph();
        let path = a_star_search(0, 4, &graph);
        assert_eq!(path, vec![0, 4]);
    }

    #[test]
    #[should_panic]
    fn test_a_star_search_no_path() {
        let mut graph = create_test_graph();
        graph.remove_edge(4);
        graph.remove_edge(3);
        a_star_search(0, 4, &graph);
    }

    #[test]
    fn test_a_star_search_same_start_and_end() {
        let graph = create_test_graph();
        let path = a_star_search(0, 0, &graph);
        assert_eq!(path, vec![0]);
    }

    #[test]
    fn test_a_star_search_alternative_path() {
        let mut graph = create_test_graph();
        graph.remove_edge(4);
        graph.add_edge(Edge::new(1, 4));
        let path = a_star_search(0, 4, &graph);
        assert_eq!(path, vec![0, 1, 4]);
    }
}
