
#[derive(Clone, Copy)]
pub struct Edge {
    pub source_index: i32,
    pub destination_index: i32
}

impl Edge {
    pub fn new(source_index: i32, destination_index: i32) -> Edge {
        //TODO always store the minumum value as source_index and the maximum value as destination_index
        Edge{
            source_index: source_index,
            destination_index: destination_index
        }
    }

    pub fn new_inactive() -> Edge {
        Edge{
            source_index: -1,
            destination_index: -1
        }
    }
    
    pub fn is_active(self) -> bool {
        self.source_index != -1 || self.destination_index != -1
    }
}