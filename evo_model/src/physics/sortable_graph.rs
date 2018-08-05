use std::cmp::Ordering;
use std::usize;

#[derive(Debug)]
pub struct SortableGraph<N, E> {
    nodes: Vec<N>,
    node_handles: Vec<NodeHandle>,
    edges: Vec<E>,
}

impl<N, E> SortableGraph<N, E> {
    pub fn new() -> Self {
        SortableGraph {
            nodes: vec![],
            node_handles: vec![],
            edges: vec![],
        }
    }

    pub fn add_node(&mut self, node: N) {
//        node.handle_mut().index = self.nodes.len();
//        self.node_handles.push(node.handle());

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

pub trait GraphNode {
    fn handle(&self) -> NodeHandle;

    fn handle_mut(&mut self) -> &mut NodeHandle;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeHandle {
    index: usize
}

impl NodeHandle {
    pub fn unset() -> Self {
        NodeHandle { index: usize::MAX }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //#[test]
    fn added_node_has_correct_handle() {
        let mut graph: SortableGraph<SpyNode, SpyEdge> = SortableGraph::new();

        graph.add_node(SpyNode::new());

        let node = &graph.nodes()[0];
        assert_eq!(node, graph.node(node.handle()));
    }

    #[derive(Debug, PartialEq)]
    struct SpyNode {
        pub handle: NodeHandle
    }

    impl SpyNode {
        fn new() -> Self {
            SpyNode {
                handle: NodeHandle::unset()
            }
        }
    }

    impl GraphNode for SpyNode {
        fn handle(&self) -> NodeHandle {
            self.handle
        }

        fn handle_mut(&mut self) -> &mut NodeHandle {
            &mut self.handle
        }
    }

    #[derive(Debug, PartialEq)]
    struct SpyEdge {}
}
