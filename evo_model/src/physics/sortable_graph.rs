use physics::bond::*;

#[derive(Debug)]
pub struct SortableGraph<N> {
    pub nodes: Vec<N>,
    pub node_indexes: Vec<usize>,
    bonds: Vec<Bond>,
}

impl<N> SortableGraph<N> {
    pub fn new() -> Self {
        SortableGraph {
            nodes: vec![],
            node_indexes: vec![],
            bonds: vec![],
        }
    }

    pub fn add_node(&mut self, node: N) {
        self.node_indexes.push(self.nodes.len());
        self.nodes.push(node);
    }

    pub fn add_bond(&mut self, bond: Bond) {
        self.bonds.push(bond);
    }

    pub fn nodes(&self) -> &[N] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [N] {
        &mut self.nodes
    }
}
