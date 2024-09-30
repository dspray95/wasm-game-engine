use crate::vector::vector_2d::Vector2D;

#[derive(Clone, Copy, PartialEq)]
pub struct Node {
    pub index_of: i32,
    pub x_pos: f32,
    pub y_pos: f32,
}

impl Node {
    pub fn new(x: f32, y: f32, index: i32) -> Node {
        Node {
            index_of: index,
            x_pos: x,
            y_pos: y,
        }
    }

    pub fn new_inactive() -> Node {
        Node {
            index_of: -1,
            x_pos: -1.0,
            y_pos: -1.0,
        }
    }

    pub fn update_index(&mut self, index: i32) {
        self.index_of = index;
    }

    pub fn is_active(self) -> bool {
        return self.index_of >= 0;
    }
}

pub fn get_distance_between_nodes(node_a: Node, node_b: Node) -> f32{
    let x_diff = node_a.x_pos - node_b.x_pos;
    let y_diff = node_a.y_pos - node_b.y_pos;
    return (x_diff.powi(2) + y_diff.powi(2)).sqrt();
}

pub fn get_manhattan_distance_between_nodes(node_a: Node, node_b: Node) -> f32{
    let x_diff = (node_a.x_pos - node_b.x_pos).abs();
    let y_diff = (node_a.y_pos - node_b.y_pos).abs();
    return x_diff + y_diff;
}

pub fn get_midpoint_between_nodes(node_a: Node, node_b: Node) -> Vector2D {
    let x_pos = (node_a.x_pos + node_b.x_pos) / 2.0;
    let y_pos = (node_a.y_pos + node_b.y_pos) / 2.0;
    return Vector2D::new(x_pos, y_pos);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_midpoint_between_nodes() {
        let node_a = Node::new(0.0, 0.0, 0);
        let node_b = Node::new(4.0, 4.0, 1);
        let midpoint = get_midpoint_between_nodes(node_a, node_b);
        assert_eq!(midpoint.x, 2.0);
        assert_eq!(midpoint.y, 2.0);
    }

    #[test]
    fn test_get_midpoint_between_same_nodes() {
        let node_a = Node::new(1.0, 1.0, 0);
        let node_b = Node::new(1.0, 1.0, 1);
        let midpoint = get_midpoint_between_nodes(node_a, node_b);
        assert_eq!(midpoint.x, 1.0);
        assert_eq!(midpoint.y, 1.0);
    }

    #[test]
    fn test_get_midpoint_between_negative_coordinates() {
        let node_a = Node::new(-1.0, -1.0, 0);
        let node_b = Node::new(-3.0, -3.0, 1);
        let midpoint = get_midpoint_between_nodes(node_a, node_b);
        assert_eq!(midpoint.x, -2.0);
        assert_eq!(midpoint.y, -2.0);
    }

    #[test]
    fn test_get_midpoint_between_mixed_coordinates() {
        let node_a = Node::new(-1.0, 1.0, 0);
        let node_b = Node::new(3.0, -3.0, 1);
        let midpoint = get_midpoint_between_nodes(node_a, node_b);
        assert_eq!(midpoint.x, 1.0);
        assert_eq!(midpoint.y, -1.0);
    }
    #[test]
    fn test_get_distance_between_nodes() {
        let node_a = Node::new(0.0, 0.0, 0);
        let node_b = Node::new(3.0, 4.0, 1);
        let distance = get_distance_between_nodes(node_a, node_b);
        assert_eq!(distance, 5.0);
    }

    #[test]
    fn test_get_distance_between_same_nodes() {
        let node_a = Node::new(1.0, 1.0, 0);
        let node_b = Node::new(1.0, 1.0, 1);
        let distance = get_distance_between_nodes(node_a, node_b);
        assert_eq!(distance, 0.0);
    }

    #[test]
    fn test_get_distance_between_negative_coordinates() {
        let node_a = Node::new(-1.0, -1.0, 0);
        let node_b = Node::new(-4.0, -5.0, 1);
        let distance = get_distance_between_nodes(node_a, node_b);
        assert_eq!(distance, 5.0);
    }

    #[test]
    fn test_get_manhattan_distance_between_nodes() {
        let node_a = Node::new(0.0, 0.0, 0);
        let node_b = Node::new(3.0, 4.0, 1);
        let distance = get_manhattan_distance_between_nodes(node_a, node_b);
        assert_eq!(distance, 7.0);
    }

    #[test]
    fn test_get_manhattan_distance_between_same_nodes() {
        let node_a = Node::new(1.0, 1.0, 0);
        let node_b = Node::new(1.0, 1.0, 1);
        let distance = get_manhattan_distance_between_nodes(node_a, node_b);
        assert_eq!(distance, 0.0);
    }

    #[test]
    fn test_get_manhattan_distance_between_negative_coordinates() {
        let node_a = Node::new(-1.0, -1.0, 0);
        let node_b = Node::new(-4.0, -5.0, 1);
        let distance = get_manhattan_distance_between_nodes(node_a, node_b);
        assert_eq!(distance, 7.0);
    }

}
