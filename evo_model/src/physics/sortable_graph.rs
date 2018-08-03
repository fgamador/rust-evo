#[derive(Debug)]
pub struct SortableGraph<N, E> {
    pub nodes: Vec<N>,
    pub node_indexes: Vec<usize>,
    edges: Vec<E>,
}

impl<N, E> SortableGraph<N, E> {
    pub fn new() -> Self {
        SortableGraph {
            nodes: vec![],
            node_indexes: vec![],
            edges: vec![],
        }
    }

    pub fn add_node(&mut self, node: N) {
        self.node_indexes.push(self.nodes.len());
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, bond: E) {
        self.edges.push(bond);
    }

    pub fn nodes(&self) -> &[N] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [N] {
        &mut self.nodes
    }
}
