use std::cmp::Ordering;

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

    pub fn add_edge(&mut self, edge: E) {
        self.edges.push(edge);
    }

    pub fn sort(&mut self, cmp: fn(&N, &N) -> Ordering) {
        let nodes = &mut self.nodes;
        // TODO convert this to insertion sort (and rename fn to insertion_sort)
        self.node_indexes.sort_unstable_by(|i1, i2| cmp(&nodes[*i1], &nodes[*i2]));
    }

    // TODO temporary, I hope
    pub fn node_indexes(&self) -> &[usize] {
        &self.node_indexes
    }

    pub fn nodes(&self) -> &[N] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [N] {
        &mut self.nodes
    }

    pub fn node(&self, index: usize) -> &N {
        &self.nodes[index]
    }

    pub fn node_mut(&mut self, index: usize) -> &mut N {
        &mut self.nodes[index]
    }
}
