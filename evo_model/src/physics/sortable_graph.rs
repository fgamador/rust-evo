use std::cmp::Ordering;

#[derive(Debug)]
pub struct SortableGraph<N, E> {
    nodes: Vec<N>,
    node_indexes: Vec<usize>,
    node_handles: Vec<NodeHandle>,
    edges: Vec<E>,
}

impl<N, E> SortableGraph<N, E> {
    pub fn new() -> Self {
        SortableGraph {
            nodes: vec![],
            node_indexes: vec![],
            node_handles: vec![],
            edges: vec![],
        }
    }

    pub fn add_node(&mut self, node: N) {
        self.node_indexes.push(self.nodes.len());
        self.node_handles.push(NodeHandle { index: self.nodes.len() });
        self.nodes.push(node);
    }

    pub fn add_edge(&mut self, edge: E) {
        self.edges.push(edge);
    }

    pub fn sort(&mut self, cmp: fn(&N, &N) -> Ordering) {
        let nodes = &self.nodes;
        // TODO convert this to insertion sort (and rename fn to insertion_sort)
        self.node_handles.sort_unstable_by(|h1, h2| cmp(&nodes[h1.index], &nodes[h2.index]));
    }

    // TODO temporary, I hope
    pub fn node_handles(&self) -> &[NodeHandle] {
        &self.node_handles
    }

    pub fn nodes(&self) -> &[N] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [N] {
        &mut self.nodes
    }

    pub fn node(&self, handle: NodeHandle) -> &N {
        &self.nodes[handle.index]
    }

    pub fn node_mut(&mut self, handle: NodeHandle) -> &mut N {
        &mut self.nodes[handle.index]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeHandle {
    index: usize
}
