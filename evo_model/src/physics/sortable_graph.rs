use physics::bond::*;

#[derive(Debug)]
pub struct SortableGraph<T> {
    pub nodes: Vec<T>,
    pub node_indexes: Vec<usize>,
    bonds: Vec<Bond>,
}

impl<T> SortableGraph<T> {
    pub fn new() -> Self {
        SortableGraph {
            nodes: vec![],
            node_indexes: vec![],
            bonds: vec![],
        }
    }

    pub fn add_node(&mut self, node: T) {
        self.node_indexes.push(self.nodes.len());
        self.nodes.push(node);
    }

    pub fn add_bond(&mut self, bond: Bond) {
        self.bonds.push(bond);
    }

    pub fn balls(&self) -> &[T] {
        &self.nodes
    }

    pub fn balls_mut(&mut self) -> &mut [T] {
        &mut self.nodes
    }
}
