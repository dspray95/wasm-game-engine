use std::vec;

use super::node::Node;
use super::edge::Edge;

//With 1 million max nodes we have arrays of sizes:
// nodes: 1_000_000 * 16 = 16_000_000 bytes = 16 MB
// edges: 1_000_000 * 8 = 8_000_000 bytes = 8 MB
const MAX_NODES: usize = 1_000_000;

#[derive(Clone)]
pub struct Graph {
    last_node_index: i32,
    last_edge_index: i32,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            last_node_index: -1,
            last_edge_index: -1,
            nodes: vec![Node::new_inactive(); MAX_NODES],
            edges: vec![Edge::new_inactive(); MAX_NODES],
        }
    }

    pub fn add_node(&mut self, node_to_add: Node) -> bool {
        if node_to_add.index_of > self.nodes.len() as i32 {panic!("No more space in nodes array!")};
        if !node_to_add.is_active() {return false;}

        let new_node_index: i32 = self.last_node_index + 1;
        if node_to_add.index_of > self.last_node_index {
            //In this case we're appending a node to our current set, so we don't need to do any extra work
            self.nodes[node_to_add.index_of as usize] = node_to_add;
        } else {
            //If we're inserting a node to an already existing index we need to shift the rest of the nodes
            //Up one
            self.shift_nodes_right(node_to_add.index_of, new_node_index, 1);
            //Insert new node
            self.nodes[node_to_add.index_of as usize] = node_to_add;
        }
        self.last_node_index = new_node_index;
        return true;
    }

    /// Adds a node to the graph between two existing nodes
    /// This will also replace any edges between the existing node with new edges going through
    /// the new node.
    pub fn add_node_between(&mut self, node_to_add: Node, node_before: Node, node_after: Node){
        self.add_node(node_to_add);

        let mut edges_to_remove: Vec<i32> = Vec::new();
        for i in 0..=self.last_edge_index {
            let edge = &mut self.edges[i as usize];
            if edge.source_index == node_before.index_of && edge.destination_index == node_after.index_of{
                let edge_a_to_b: Edge = Edge::new(node_before.index_of, node_to_add.index_of);
                let edge_b_to_c: Edge = Edge::new(node_to_add.index_of, node_after.index_of);    
                self.add_edge(edge_a_to_b);
                self.add_edge(edge_b_to_c);
                edges_to_remove.push(i);
            } else if edge.source_index == node_after.index_of && edge.destination_index == node_before.index_of{
                let edge_a_to_b: Edge = Edge::new(node_before.index_of, node_to_add.index_of);
                let edge_b_to_c: Edge = Edge::new(node_to_add.index_of, node_after.index_of);    
                self.add_edge(edge_a_to_b);
                self.add_edge(edge_b_to_c);
                edges_to_remove.push(i);
            }
        }

        for edge_index in edges_to_remove {
            self.remove_edge(edge_index);
        }

    }

    pub fn remove_node(&mut self, index_to_remove: i32) -> bool {
        if index_to_remove > self.last_node_index || index_to_remove < 0 {
            panic!("Can't remove a node that doesn't exist! Tried to remove index {}", index_to_remove);
        }

        //Delte edges that are connected to the node we're removing - this could be faster
        let mut edges_to_remove: Vec<i32> = Vec::new();
        for edge in &self.edges {
            if edge.source_index == index_to_remove || edge.destination_index == index_to_remove {
                edges_to_remove.push(edge.source_index);
            }
        }
        for edge_index in edges_to_remove.iter().rev() {
            self.remove_edge(*edge_index);
        }

        //Shifting the nodes left will overwrite the node we want to remove
        self.shift_nodes_left(index_to_remove + 1, self.last_node_index, 1);
        self.last_node_index -= 1;
        return true
    }

    pub fn add_edge(&mut self, edge_to_add: Edge){
        if self.has_edge_bi_directional(
            edge_to_add.source_index, 
            edge_to_add.destination_index
        ){
            return;
        
        }
        if edge_to_add.source_index > self.last_node_index || edge_to_add.destination_index > self.last_node_index {
            panic!("We can't add an edge for nodes that dont exist!");
        }

        let new_edge_index: i32 = self.last_edge_index + 1;
        self.edges[new_edge_index as usize] = edge_to_add;
        self.last_edge_index = new_edge_index;
    }
    
    pub fn remove_edge(&mut self, index_to_remove: i32) {
        if index_to_remove > self.last_edge_index || index_to_remove < 0 {
            panic!("Can't remove an edge that doesn't exist! Tried to remove index {}", index_to_remove);
        }
        self.edges[index_to_remove as usize] = Edge::new_inactive();

        //We need to shift the edges left to fill the empty space
        for i in index_to_remove..self.last_edge_index {
            self.edges[i as usize] = self.edges[(i + 1) as usize];
        }
        self.edges[self.last_edge_index as usize] = Edge::new_inactive();
        self.last_edge_index -= 1;
    }

    /// Returns a vector of nodes that are connected to the given node by an 
    /// edge in this graph.
    pub fn get_connected_nodes(&self, node: Node) -> Vec<Node>{
        let mut connected_nodes: Vec<Node> = Vec::new();
        for i in 0..=self.last_edge_index {
            let edge = &self.edges[i as usize];
            if edge.source_index == node.index_of {
                connected_nodes.push(self.nodes[edge.destination_index as usize]);
            } else if edge.destination_index == node.index_of {
                connected_nodes.push(self.nodes[edge.source_index as usize]);
            }
           
        }
        return connected_nodes;
    }

    pub fn get_node(&self, index: i32) -> Node {
        return self.nodes[index as usize];
    }

    pub fn has_edge_bi_directional(&self, edge_source: i32, edge_destination: i32) -> bool{
        for i in 0..=self.last_edge_index {
            let edge = &self.edges[i as usize];
            if edge_source == edge.source_index{
                if edge_destination == edge.destination_index {
                    return true;
                }
            } else if edge_source == edge.destination_index {
                if edge_destination == edge.source_index {
                    return true;
                }
            }
        }
        return false;
    }

    fn shift_nodes_right(&mut self, start_at: i32, end_at: i32, positions_to_shift: i32){
        if end_at > self.nodes.len() as i32 {
            panic!("Not enough space to shift nodes right {} positions!", positions_to_shift)
        };

        //Shift nodes right
        for i in (start_at..=end_at).rev(){
            self.nodes[i as usize] = self.nodes[(i - positions_to_shift) as usize];
            self.nodes[i as usize].index_of = i;   
        }

        //Fix edges
        for i in 0..=self.last_edge_index {
            let edge = &mut self.edges[i as usize];
            if edge.source_index >= start_at {
                edge.source_index += positions_to_shift;
            }
            if edge.destination_index >= start_at {
                edge.destination_index += positions_to_shift;
            }
        }
        //Set the node at the start index to be an inactive one, since we've moved
        //the node that was there to a new index
        self.nodes[start_at as usize] = Node::new_inactive();
    }

    fn shift_nodes_left(&mut self, start_at: i32, end_at: i32, positions_to_shift: i32){
        if end_at <= 0 {
           panic!("Can't shift nodes left past index 0!");
        }
        
        //Shift nodes left
        for i in start_at..=end_at {
            if !self.nodes[i as usize].is_active() {
                break;
            }
            self.nodes[(i - positions_to_shift) as usize] = self.nodes[i as usize];
            self.nodes[(i - positions_to_shift) as usize].index_of = i - positions_to_shift;
        }

        //Fix edges
        for i in 0..=self.last_edge_index {
            let edge = &mut self.edges[i as usize];
            if edge.source_index >= start_at {
                edge.source_index -= positions_to_shift;
            }
            if edge.destination_index >= start_at {
                edge.destination_index -= positions_to_shift;
            }
        }
        //Set the node in the new empty space to be inactive
        self.nodes[end_at as usize] = Node::new_inactive();
    }

}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_shift_left() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);

        let edge = Edge::new(1, 2);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);
        graph.add_edge(edge);

        graph.shift_nodes_left(1, graph.last_node_index, 1);

        assert_eq!(graph.nodes[0].index_of, 0);
        assert_eq!(graph.nodes[0].x_pos, 2.0);

        assert_eq!(graph.nodes[1].index_of, 1);
        assert_eq!(graph.nodes[1].x_pos, 3.0);

        assert_eq!(graph.edges[0].source_index, 0);
        assert_eq!(graph.edges[0].destination_index, 1);

        assert_eq!(graph.nodes[2].index_of, -1);
    }
    
    #[test]
    fn test_shift_right() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);

        let edge = Edge::new(0, 1);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);
        graph.add_edge(edge);

        graph.shift_nodes_right(1, graph.last_node_index + 1, 1);

        assert_eq!(graph.nodes[2].x_pos, 2.0);
        assert_eq!(graph.nodes[2].index_of, 2);
        assert_eq!(graph.nodes[1].index_of, -1);
        assert_eq!(graph.edges[0].source_index, 0);
        assert_eq!(graph.edges[0].destination_index, 2);
    }

    #[test]
    fn test_add_node() {
        let mut graph = Graph::new();
        let node = Node::new(1.0, 2.0, 0);
        graph.add_node(node);

        assert_eq!(graph.last_node_index, 0);
        assert_eq!(graph.nodes[0].x_pos, 1.0);
        assert_eq!(graph.nodes[0].y_pos, 2.0);
        assert_eq!(graph.nodes[0].index_of, 0);
    }

    #[test]
    fn test_add_multiple_nodes() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 2.0, 0);
        let node_1 = Node::new(3.0, 4.0, 1);
        graph.add_node(node_0);
        graph.add_node(node_1);

        assert_eq!(graph.last_node_index, 1);
        assert_eq!(graph.nodes[0].x_pos, 1.0);
        assert_eq!(graph.nodes[0].y_pos, 2.0);
        assert_eq!(graph.nodes[0].index_of, 0);
        assert_eq!(graph.nodes[1].x_pos, 3.0);
        assert_eq!(graph.nodes[1].y_pos, 4.0);
        assert_eq!(graph.nodes[1].index_of, 1);
    }

    #[test]
    fn test_insert_node_into_already_occupied_index() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);
        let node4: Node = Node::new(4.0, 4.0, 1);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);
        graph.add_node(node4);

        assert_eq!(graph.last_node_index, 3);
        assert_eq!(graph.nodes[0].x_pos, 1.0);
        assert_eq!(graph.nodes[1].x_pos, 4.0);
        assert_eq!(graph.nodes[2].x_pos, 2.0);
        assert_eq!(graph.nodes[3].x_pos, 3.0);
    }

    #[test]
    fn test_add_node_with_negative_index() {
        let mut graph = Graph::new();
        let node = Node::new(1.0, 2.0, -1);
        assert_eq!(graph.add_node(node), false);

       
    }

    #[test]
    fn test_remove_first_node() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);

        graph.remove_node(0);

        assert_eq!(graph.last_node_index, 1);
        assert_eq!(graph.nodes[0].x_pos, 2.0);
        assert_eq!(graph.nodes[0].index_of, 0);
        assert_eq!(graph.nodes[1].x_pos, 3.0);
        assert_eq!(graph.nodes[1].index_of, 1);
        assert_eq!(graph.nodes[2].is_active(), false);
    }

    #[test]
    fn test_remove_last_node() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);

        graph.remove_node(2);

        assert_eq!(graph.last_node_index, 1);
        assert_eq!(graph.nodes[0].x_pos, 1.0);
        assert_eq!(graph.nodes[0].index_of, 0);
        assert_eq!(graph.nodes[1].x_pos, 2.0);
        assert_eq!(graph.nodes[1].index_of, 1);
        assert_eq!(graph.nodes[2].is_active(), false);
    }

    #[test]
    fn test_remove_middle_node() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);

        graph.remove_node(1);

        assert_eq!(graph.last_node_index, 1);
        assert_eq!(graph.nodes[0].x_pos, 1.0);
        assert_eq!(graph.nodes[0].index_of, 0);
        assert_eq!(graph.nodes[1].x_pos, 3.0);
        assert_eq!(graph.nodes[1].index_of, 1);
        assert_eq!(graph.nodes[2].is_active(), false);
    }

    #[test]
    #[should_panic]
    fn test_remove_nonexistent_node() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);

        graph.add_node(node_0);
        graph.add_node(node_1);

        graph.remove_node(3);
    }

    #[test]
    fn test_remove_node_with_edges() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);

        let edge1 = Edge::new(0, 1);
        let edge2 = Edge::new(1, 2);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);
        graph.add_edge(edge1);
        graph.add_edge(edge2);

        graph.remove_node(1);

        assert_eq!(graph.last_node_index, 1);
        assert_eq!(graph.nodes[0].x_pos, 1.0);
        assert_eq!(graph.nodes[0].index_of, 0);
        assert_eq!(graph.nodes[1].x_pos, 3.0);
        assert_eq!(graph.nodes[1].index_of, 1);
        assert_eq!(graph.nodes[2].is_active(), false);

        assert_eq!(graph.edges[0].is_active(), false);
        assert_eq!(graph.edges[1].is_active(), false);
    }

    #[test]
    fn test_add_node_between() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);
        let node_between = Node::new(1.5, 1.5, 3);

        let edge1 = Edge::new(0, 1);
        let edge2 = Edge::new(1, 2);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);
        graph.add_edge(edge1);
        graph.add_edge(edge2);

        graph.add_node_between(node_between, node_0, node_1);

        assert_eq!(graph.last_node_index, 3);
        assert_eq!(graph.nodes[0].x_pos, 1.0);
        assert_eq!(graph.nodes[0].index_of, 0);
        assert_eq!(graph.nodes[1].x_pos, 2.0);
        assert_eq!(graph.nodes[1].index_of, 1);
        assert_eq!(graph.nodes[2].x_pos, 3.0);
        assert_eq!(graph.nodes[2].index_of, 2);
        assert_eq!(graph.nodes[3].x_pos, 1.5);
        assert_eq!(graph.nodes[3].index_of, 3);

        // The first edge will now be the second one we added, since we deleted 
        // the edge between node_0 and node_1 when we inserted the new node
        assert_eq!(graph.edges[0].source_index, 1);
        assert_eq!(graph.edges[0].destination_index, 2);
        // These edges should be the new ones added to replace the original
        // edge between node_0 and node_1
        assert_eq!(graph.edges[1].source_index, 0);
        assert_eq!(graph.edges[1].destination_index, 3);
        assert_eq!(graph.edges[2].source_index, 3);
        assert_eq!(graph.edges[2].destination_index, 1);
    }

    #[test]
    fn test_add_node_between_reverse_edge() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);
        let node_between = Node::new(1.5, 1.5, 3);

        let edge1 = Edge::new(1, 0);
        let edge2 = Edge::new(2, 1);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);
        graph.add_edge(edge1);
        graph.add_edge(edge2);

        graph.add_node_between(node_between, node_0, node_1);

        assert_eq!(graph.last_node_index, 3);
        assert_eq!(graph.nodes[0].x_pos, 1.0);
        assert_eq!(graph.nodes[0].index_of, 0);
        assert_eq!(graph.nodes[1].x_pos, 2.0);
        assert_eq!(graph.nodes[1].index_of, 1);
        assert_eq!(graph.nodes[2].x_pos, 3.0);
        assert_eq!(graph.nodes[2].index_of, 2);
        assert_eq!(graph.nodes[3].x_pos, 1.5);
        assert_eq!(graph.nodes[3].index_of, 3);

        assert_eq!(graph.edges[0].source_index, 2);
        assert_eq!(graph.edges[0].destination_index, 1);
        assert_eq!(graph.edges[1].source_index, 0);
        assert_eq!(graph.edges[1].destination_index, 3);
        assert_eq!(graph.edges[2].source_index, 3);
        assert_eq!(graph.edges[2].destination_index, 1);
    }

    #[test]
    fn test_add_node_between_no_existing_edge() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);
        let node_between = Node::new(1.5, 1.5, 1);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);

        graph.add_node_between(node_between, node_0, node_1);

        assert_eq!(graph.last_node_index, 3);
        assert_eq!(graph.nodes[0].x_pos, 1.0);
        assert_eq!(graph.nodes[0].index_of, 0);
        assert_eq!(graph.nodes[1].x_pos, 1.5);
        assert_eq!(graph.nodes[1].index_of, 1);
        assert_eq!(graph.nodes[2].x_pos, 2.0);
        assert_eq!(graph.nodes[2].index_of, 2);
        assert_eq!(graph.nodes[3].x_pos, 3.0);
        assert_eq!(graph.nodes[3].index_of, 3);
        //Make sure we haven't inadvertently added any edges
        assert!(!graph.edges[0].is_active());
        assert_eq!(graph.last_edge_index, -1);
    }

    #[test]
    fn test_get_connected_nodes() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);
        let node_2 = Node::new(3.0, 3.0, 2);

        let edge1 = Edge::new(0, 1);
        let edge2 = Edge::new(1, 2);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_node(node_2);
        graph.add_edge(edge1);
        graph.add_edge(edge2);

        let connected_nodes = graph.get_connected_nodes(node_1);
        assert_eq!(connected_nodes.len(), 2);
        assert!(connected_nodes.contains(&node_0));
        assert!(connected_nodes.contains(&node_2));
    }

    #[test]
    fn test_get_connected_nodes_no_edges() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);

        graph.add_node(node_0);
        graph.add_node(node_1);

        let connected_nodes = graph.get_connected_nodes(node_0);
        assert_eq!(connected_nodes.len(), 0);
    }

    #[test]
    fn test_get_connected_nodes_single_edge() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);

        let edge = Edge::new(0, 1);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_edge(edge);

        let connected_nodes = graph.get_connected_nodes(node_0);
        assert_eq!(connected_nodes.len(), 1);
        assert!(connected_nodes.contains(&node_1));
    }

    #[test]
    fn test_get_connected_nodes_bidirectional_edges() {
        let mut graph = Graph::new();
        let node_0 = Node::new(1.0, 1.0, 0);
        let node_1 = Node::new(2.0, 2.0, 1);

        let edge1 = Edge::new(0, 1);
        let edge2 = Edge::new(1, 0);

        graph.add_node(node_0);
        graph.add_node(node_1);
        graph.add_edge(edge1);
        graph.add_edge(edge2);

        let connected_nodes = graph.get_connected_nodes(node_0);
        assert_eq!(connected_nodes.len(), 1);
        assert!(connected_nodes.contains(&node_1));
    }
}
