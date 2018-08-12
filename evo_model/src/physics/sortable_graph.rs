use std::cmp::Ordering;
use std::usize;

#[derive(Debug)]
pub struct SortableGraph<N: GraphNode, E: GraphEdge> {
    unsorted_nodes: Vec<N>,
    sortable_node_handles: Vec<NodeHandle>,
    edges: Vec<E>,
}

impl<N: GraphNode, E: GraphEdge> SortableGraph<N, E> {
    pub fn new() -> Self {
        SortableGraph {
            unsorted_nodes: vec![],
            sortable_node_handles: vec![],
            edges: vec![],
        }
    }

    pub fn add_node(&mut self, mut node: N) -> NodeHandle {
        node.graph_node_data_mut().node_handle.index = self.unsorted_nodes.len();
        let handle = node.node_handle();
        self.sortable_node_handles.push(handle);
        self.unsorted_nodes.push(node);
        handle
    }

    pub fn add_edge(&mut self, edge: E) {
        let edge_handle = EdgeHandle { index: self.edges.len() };
        self.add_edge_to_node(edge.node1_handle(), edge_handle);
        self.add_edge_to_node(edge.node2_handle(), edge_handle);
        self.edges.push(edge);
    }

    fn add_edge_to_node(&mut self, node_handle: NodeHandle, edge_handle: EdgeHandle) {
        self.node_mut(node_handle).graph_node_data_mut().edge_handles.push(edge_handle);
    }

    pub fn sort_node_handles(&mut self, cmp: fn(&N, &N) -> Ordering) {
        let nodes = &self.unsorted_nodes;
        // TODO convert this to insertion sort (and rename fn to insertion_sort)
        self.sortable_node_handles.sort_unstable_by(|h1, h2| cmp(&nodes[h1.index], &nodes[h2.index]));
    }

    pub fn node_handles(&self) -> &[NodeHandle] {
        &self.sortable_node_handles
    }

    pub fn unsorted_nodes(&self) -> &[N] {
        &self.unsorted_nodes
    }

    pub fn unsorted_nodes_mut(&mut self) -> &mut [N] {
        &mut self.unsorted_nodes
    }

    pub fn node(&self, handle: NodeHandle) -> &N {
        &self.unsorted_nodes[handle.index]
    }

    pub fn node_mut(&mut self, handle: NodeHandle) -> &mut N {
        &mut self.unsorted_nodes[handle.index]
    }

    pub fn edges(&self) -> &[E] {
        &self.edges
    }

    pub fn have_edge(&self, node1: &N, node2: &N) -> bool {
        self.has_edge_to(node1, node2) || self.has_edge_to(node2, node1)
    }

    fn has_edge_to(&self, node1: &N, node2: &N) -> bool {
        node1.graph_node_data().edge_handles.iter()
            .map(|edge_handle| { self.edges[edge_handle.index].node2_handle() })
            .any(|node2_handle| { node2_handle == node2.node_handle() })
    }

    pub fn meta_edges(&self) -> &[E] {
        &self.edges
    }
}

pub trait GraphNode {
    fn node_handle(&self) -> NodeHandle;

    fn graph_node_data(&self) -> &GraphNodeData;

    fn graph_node_data_mut(&mut self) -> &mut GraphNodeData;
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

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeData {
    node_handle: NodeHandle,
    edge_handles: Vec<EdgeHandle>,
}

impl GraphNodeData {
    pub fn new() -> Self {
        GraphNodeData {
            node_handle: NodeHandle::unset(),
            edge_handles: vec![],
        }
    }

    pub fn handle(&self) -> NodeHandle {
        self.node_handle
    }
}

pub trait GraphEdge {
    fn node1_handle(&self) -> NodeHandle;

    fn node2_handle(&self) -> NodeHandle;

    fn graph_edge_data(&self) -> &GraphEdgeData;

    fn graph_edge_data_mut(&mut self) -> &mut GraphEdgeData;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct EdgeHandle {
    index: usize
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphEdgeData {
    node1_handle: NodeHandle,
    node2_handle: NodeHandle,
}

impl GraphEdgeData {
    pub fn new(node1_handle: NodeHandle, node2_handle: NodeHandle) -> Self {
        GraphEdgeData {
            node1_handle,
            node2_handle,
        }
    }

    pub fn node1_handle(&self) -> NodeHandle {
        self.node1_handle
    }

    pub fn node1_handle_mut(&mut self) -> &mut NodeHandle {
        &mut self.node1_handle
    }

    pub fn node2_handle(&self) -> NodeHandle {
        self.node2_handle
    }

    pub fn node2_handle_mut(&mut self) -> &mut NodeHandle {
        &mut self.node2_handle
    }
}

#[derive(Debug, PartialEq)]
pub struct SimpleGraphEdge {
    edge_data: GraphEdgeData,
    pub node1_handle: NodeHandle,
    pub node2_handle: NodeHandle,
}

impl SimpleGraphEdge {
    pub fn new(node1: &GraphNode, node2: &GraphNode) -> Self {
        SimpleGraphEdge {
            edge_data: GraphEdgeData::new(node1.node_handle(), node2.node_handle()),
            node1_handle: node1.node_handle(),
            node2_handle: node2.node_handle(),
        }
    }
}

impl GraphEdge for SimpleGraphEdge {
    fn node1_handle(&self) -> NodeHandle {
        self.node1_handle
    }

    fn node2_handle(&self) -> NodeHandle {
        self.node2_handle
    }

    fn graph_edge_data(&self) -> &GraphEdgeData {
        &self.edge_data
    }

    fn graph_edge_data_mut(&mut self) -> &mut GraphEdgeData {
        &mut self.edge_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_node_has_correct_handle() {
        let mut graph: SortableGraph<SpyNode, SimpleGraphEdge> = SortableGraph::new();

        let handle = graph.add_node(SpyNode::new());

        assert_eq!(handle, graph.unsorted_nodes()[0].node_handle());
    }

    #[test]
    fn can_fetch_node_by_handle() {
        let mut graph: SortableGraph<SpyNode, SimpleGraphEdge> = SortableGraph::new();

        let handle = graph.add_node(SpyNode::new());

        assert_eq!(graph.unsorted_nodes()[0], *graph.node(handle));
    }

    #[test]
    fn added_edge_has_correct_handles() {
        let mut graph: SortableGraph<SpyNode, SimpleGraphEdge> = SortableGraph::new();

        graph.add_node(SpyNode::new());
        graph.add_node(SpyNode::new());

        let edge = SimpleGraphEdge::new(&graph.unsorted_nodes()[0], &graph.unsorted_nodes()[1]);
        graph.add_edge(edge);

        let node1 = &graph.unsorted_nodes()[0];
        let node2 = &graph.unsorted_nodes()[1];
        let edge = &graph.edges()[0];
        assert_eq!(node1, graph.node(edge.node1_handle()));
        assert_eq!(node2, graph.node(edge.node2_handle()));
    }

    #[test]
    fn have_edge() {
        let mut graph: SortableGraph<SpyNode, SimpleGraphEdge> = SortableGraph::new();

        graph.add_node(SpyNode::new());
        graph.add_node(SpyNode::new());
        graph.add_node(SpyNode::new());

        let edge = SimpleGraphEdge::new(&graph.unsorted_nodes()[0], &graph.unsorted_nodes()[1]);
        graph.add_edge(edge);

        assert!(graph.have_edge(&graph.unsorted_nodes()[0], &graph.unsorted_nodes()[1]));
        assert!(graph.have_edge(&graph.unsorted_nodes()[1], &graph.unsorted_nodes()[0]));
        assert!(!graph.have_edge(&graph.unsorted_nodes()[0], &graph.unsorted_nodes()[2]));
    }

    #[derive(Debug, PartialEq)]
    struct SpyNode {
        pub graph_node_data: GraphNodeData,
    }

    impl SpyNode {
        fn new() -> Self {
            SpyNode {
                graph_node_data: GraphNodeData::new(),
            }
        }
    }

    impl GraphNode for SpyNode {
        fn node_handle(&self) -> NodeHandle {
            self.graph_node_data.handle()
        }

        fn graph_node_data(&self) -> &GraphNodeData {
            &self.graph_node_data
        }

        fn graph_node_data_mut(&mut self) -> &mut GraphNodeData {
            &mut self.graph_node_data
        }
    }
}
