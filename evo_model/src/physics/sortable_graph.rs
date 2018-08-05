use std::cmp::Ordering;
use std::usize;

#[derive(Debug)]
pub struct SortableGraph<N: GraphNode, E> {
    nodes: Vec<N>,
    node_handles: Vec<NodeHandle>,
    edges: Vec<E>,
}

impl<N: GraphNode, E> SortableGraph<N, E> {
    pub fn new() -> Self {
        SortableGraph {
            nodes: vec![],
            node_handles: vec![],
            edges: vec![],
        }
    }

    pub fn add_node(&mut self, mut node: N) {
        node.handle_mut().index = self.nodes.len();
        self.node_handles.push(node.handle());
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

    pub fn edges(&self) -> &[E] {
        &self.edges
    }
}

pub trait GraphNode {
    fn handle(&self) -> NodeHandle;

    fn handle_mut(&mut self) -> &mut NodeHandle;
}

pub trait GraphEdge {
    fn handle1(&self) -> NodeHandle;

    fn handle1_mut(&mut self) -> &mut NodeHandle;

    fn handle2(&self) -> NodeHandle;

    fn handle2_mut(&mut self) -> &mut NodeHandle;
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

    #[test]
    fn added_node_has_correct_handle() {
        let mut graph: SortableGraph<SpyNode, SpyEdge> = SortableGraph::new();

        graph.add_node(SpyNode::new());

        let node = &graph.nodes()[0];
        assert_eq!(node, graph.node(node.handle()));
    }

    #[test]
    fn added_edge_has_correct_handles() {
        let mut graph: SortableGraph<SpyNode, SpyEdge> = SortableGraph::new();

        graph.add_node(SpyNode::new());
        graph.add_node(SpyNode::new());

        let edge = SpyEdge::new(&graph.nodes()[0], &graph.nodes()[1]);
        graph.add_edge(edge);

        let node1 = &graph.nodes()[0];
        let node2 = &graph.nodes()[1];
        let edge = &graph.edges()[0];
        assert_eq!(node1, graph.node(edge.handle1()));
        assert_eq!(node2, graph.node(edge.handle2()));
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
    struct SpyEdge {
        pub handle1: NodeHandle,
        pub handle2: NodeHandle,
    }

    impl SpyEdge {
        pub fn new(node1: &SpyNode, node2: &SpyNode) -> Self {
            SpyEdge {
                handle1: node1.handle(),
                handle2: node2.handle(),
            }
        }
    }

    impl GraphEdge for SpyEdge {
        fn handle1(&self) -> NodeHandle {
            self.handle1
        }

        fn handle1_mut(&mut self) -> &mut NodeHandle {
            &mut self.handle1
        }

        fn handle2(&self) -> NodeHandle {
            self.handle2
        }

        fn handle2_mut(&mut self) -> &mut NodeHandle {
            &mut self.handle2
        }
    }
}
