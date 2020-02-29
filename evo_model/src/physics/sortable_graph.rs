use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::usize;

#[derive(Debug)]
pub struct SortableGraph<N: GraphNode, E: GraphEdge, ME: GraphMetaEdge> {
    unsorted_nodes: Vec<N>,
    sortable_node_handles: Vec<NodeHandle>,
    edges: Vec<E>,
    meta_edges: Vec<ME>,
}

impl<N: GraphNode, E: GraphEdge, ME: GraphMetaEdge> SortableGraph<N, E, ME> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        SortableGraph {
            unsorted_nodes: vec![],
            sortable_node_handles: vec![],
            edges: vec![],
            meta_edges: vec![],
        }
    }

    pub fn add_node(&mut self, mut node: N) -> NodeHandle {
        node.graph_node_data_mut().node_handle.index = self.unsorted_nodes.len();
        let node_handle = node.node_handle();
        self.sortable_node_handles.push(node_handle);
        self.unsorted_nodes.push(node);
        node_handle
    }

    /// Removes the nodes referenced by `handles`.
    ///
    /// Warning: this function has two big gotchas:
    ///
    /// 1) `handles` should be in ascending order of `index`. If not, the function will
    /// panic on index out-of-bounds if we're removing nodes at the end of `unsorted_nodes`.
    ///
    /// 2) Worse, this function changes the nodes referenced by some of the remaining handles.
    /// Never retain handles across a call to this function.
    pub fn remove_nodes(&mut self, handles: &[NodeHandle]) {
        for handle in handles.iter().rev() {
            self.remove_edges(&self.node(*handle).graph_node_data().edge_handles.clone());
            self.unsorted_nodes.swap_remove(handle.index);
            if handle.index < self.unsorted_nodes.len() {
                self.fix_swapped_node_and_its_edges(*handle);
            }
        }
        self.remove_obsolete_node_handles();
    }

    fn fix_swapped_node_and_its_edges(&mut self, handle: NodeHandle) {
        self.node_mut(handle).graph_node_data_mut().node_handle = handle;
        // for edge_handle in self.node(handle).graph_node_data().edge_handles.clone() {
        //     let edge = self.edge_mut(edge_handle);
        //     // TODO find the right handle to update
        //     edge.graph_edge_data_mut().node1_handle = handle;
        // }
    }

    fn remove_obsolete_node_handles(&mut self) {
        let first_invalid_index = self.unsorted_nodes.len();
        self.sortable_node_handles
            .retain(|h| h.index < first_invalid_index);
    }

    pub fn add_edge(&mut self, mut edge: E) -> EdgeHandle {
        edge.graph_edge_data_mut().edge_handle.index = self.edges.len();
        let edge_handle = edge.edge_handle();
        self.add_edge_to_node(edge.node1_handle(), edge_handle);
        self.add_edge_to_node(edge.node2_handle(), edge_handle);
        self.edges.push(edge);
        edge_handle
    }

    fn add_edge_to_node(&mut self, node_handle: NodeHandle, edge_handle: EdgeHandle) {
        self.node_mut(node_handle)
            .graph_node_data_mut()
            .edge_handles
            .push(edge_handle);
    }

    /// Same gotchas as in remove_nodes.
    pub fn remove_edges(&mut self, edge_handles: &[EdgeHandle]) {
        for edge_handle in edge_handles.iter().rev() {
            self.remove_edge_from_node(self.edge(*edge_handle).node1_handle(), *edge_handle);
            self.remove_edge_from_node(self.edge(*edge_handle).node2_handle(), *edge_handle);
            // TODO obsolete meta-edges
            self.edges.swap_remove(edge_handle.index);
            if edge_handle.index < self.edges.len() {
                self.fix_swapped_edge(EdgeHandle::new(self.edges.len()), *edge_handle);
            }
        }
    }

    fn remove_edge_from_node(&mut self, node_handle: NodeHandle, edge_handle: EdgeHandle) {
        let edge_handles = &mut self
            .node_mut(node_handle)
            .graph_node_data_mut()
            .edge_handles;
        let index = edge_handles.iter().position(|&h| h == edge_handle).unwrap();
        // TODO can this swap-reordering mess up the ordering needed by remove_edges?
        edge_handles.swap_remove(index);
    }

    fn fix_swapped_edge(&mut self, old_handle: EdgeHandle, new_handle: EdgeHandle) {
        self.edge_mut(new_handle).graph_edge_data_mut().edge_handle = new_handle;
        let edge_data = self.edge(new_handle).graph_edge_data().clone();
        self.replace_edge_handle(edge_data.node1_handle, old_handle, new_handle);
        self.replace_edge_handle(edge_data.node2_handle, old_handle, new_handle);
        // TODO handles to swapped edges in remaining meta-edges
    }

    fn replace_edge_handle(
        &mut self,
        node_handle: NodeHandle,
        old_handle: EdgeHandle,
        new_handle: EdgeHandle,
    ) {
        let node_data = self.node_mut(node_handle).graph_node_data_mut();
        for edge_handle in &mut node_data.edge_handles {
            if *edge_handle == old_handle {
                edge_handle.index = new_handle.index;
            }
        }
    }

    pub fn add_meta_edge(&mut self, meta_edge: ME) {
        //        edge.graph_edge_data_mut().edge_handle.index = self.meta_edges.len();
        //        let edge_handle = edge.edge_handle();
        //        self.add_edge_to_node(edge.node1_handle(), edge_handle);
        //        self.add_edge_to_node(edge.node2_handle(), edge_handle);
        self.meta_edges.push(meta_edge);
    }

    pub fn sort_node_handles(&mut self, cmp: fn(&N, &N) -> Ordering) {
        let nodes = &self.unsorted_nodes;
        // TODO convert this to insertion sort (and rename fn to insertion_sort)
        self.sortable_node_handles
            .sort_unstable_by(|h1, h2| cmp(&nodes[h1.index], &nodes[h2.index]));
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

    pub fn edge(&self, handle: EdgeHandle) -> &E {
        &self.edges[handle.index]
    }

    pub fn edge_mut(&mut self, handle: EdgeHandle) -> &mut E {
        &mut self.edges[handle.index]
    }

    pub fn have_edge(&self, node1: &N, node2: &N) -> bool {
        self.has_edge_to(node1, node2) || self.has_edge_to(node2, node1)
    }

    fn has_edge_to(&self, node1: &N, node2: &N) -> bool {
        node1
            .graph_node_data()
            .edge_handles
            .iter()
            .map(|edge_handle| self.edges[edge_handle.index].node2_handle())
            .any(|node2_handle| node2_handle == node2.node_handle())
    }

    pub fn meta_edges(&self) -> &[ME] {
        &self.meta_edges
    }
}

pub trait GraphNode {
    // TODO handle? self_handle?
    fn node_handle(&self) -> NodeHandle;

    fn graph_node_data(&self) -> &GraphNodeData;

    fn graph_node_data_mut(&mut self) -> &mut GraphNodeData;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeHandle {
    // TODO u32?
    index: usize,
}

impl NodeHandle {
    pub fn unset() -> Self {
        NodeHandle { index: usize::MAX }
    }
}

impl fmt::Display for NodeHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.index)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeData {
    // TODO self_handle?
    node_handle: NodeHandle,
    // TODO SmallVec?
    edge_handles: Vec<EdgeHandle>,
}

impl GraphNodeData {
    #[allow(clippy::new_without_default)]
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
    // TODO handle? self_handle?
    fn edge_handle(&self) -> EdgeHandle;

    fn node1_handle(&self) -> NodeHandle;

    fn node2_handle(&self) -> NodeHandle;

    fn graph_edge_data(&self) -> &GraphEdgeData;

    fn graph_edge_data_mut(&mut self) -> &mut GraphEdgeData;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeHandle {
    // TODO u32?
    index: usize,
}

impl EdgeHandle {
    fn new(index: usize) -> Self {
        EdgeHandle { index }
    }

    pub fn unset() -> Self {
        EdgeHandle { index: usize::MAX }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphEdgeData {
    // TODO self_handle?
    edge_handle: EdgeHandle,
    node1_handle: NodeHandle,
    node2_handle: NodeHandle,
}

impl GraphEdgeData {
    pub fn new(node1_handle: NodeHandle, node2_handle: NodeHandle) -> Self {
        GraphEdgeData {
            edge_handle: EdgeHandle::unset(),
            node1_handle,
            node2_handle,
        }
    }

    // TODO handle? self_handle?
    pub fn edge_handle(&self) -> EdgeHandle {
        self.edge_handle
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

pub trait GraphMetaEdge {
    fn edge1_handle(&self) -> EdgeHandle;

    fn edge2_handle(&self) -> EdgeHandle;

    fn graph_meta_edge_data(&self) -> &GraphMetaEdgeData;

    fn graph_meta_edge_data_mut(&mut self) -> &mut GraphMetaEdgeData;
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphMetaEdgeData {
    edge1_handle: EdgeHandle,
    edge2_handle: EdgeHandle,
}

impl GraphMetaEdgeData {
    pub fn new(edge1_handle: EdgeHandle, edge2_handle: EdgeHandle) -> Self {
        GraphMetaEdgeData {
            edge1_handle,
            edge2_handle,
        }
    }

    pub fn edge1_handle(&self) -> EdgeHandle {
        self.edge1_handle
    }

    pub fn edge2_handle(&self) -> EdgeHandle {
        self.edge2_handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::simple_graph_elements::*;

    #[test]
    fn added_node_has_correct_handle() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();

        let node_handle = graph.add_node(SimpleGraphNode::new(0));

        let node = &graph.unsorted_nodes()[0];
        assert_eq!(node.node_handle(), node_handle);
    }

    #[test]
    fn can_fetch_node_by_handle() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();

        let node_handle = graph.add_node(SimpleGraphNode::new(0));

        let node = &graph.unsorted_nodes()[0];
        assert_eq!(*graph.node(node_handle), *node);
    }

    #[test]
    fn can_remove_last_and_non_last_nodes() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();
        let node0_handle = graph.add_node(SimpleGraphNode::new(0));
        graph.add_node(SimpleGraphNode::new(1));
        let node2_handle = graph.add_node(SimpleGraphNode::new(2));

        graph.remove_nodes(&vec![node0_handle, node2_handle]);

        assert_eq!(graph.unsorted_nodes.len(), 1);
        let node = &graph.unsorted_nodes()[0];
        assert_eq!(node.id, 1);
        assert_eq!(node.node_handle().index, 0);
        assert_eq!(graph.sortable_node_handles.len(), 1);
        assert_eq!(graph.sortable_node_handles[0].index, 0);
    }

    #[test]
    fn added_edge_has_correct_handles() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();

        let node0_handle = graph.add_node(SimpleGraphNode::new(0));
        let node1_handle = graph.add_node(SimpleGraphNode::new(1));

        let edge_handle = graph.add_edge(SimpleGraphEdge::new(
            graph.node(node0_handle),
            graph.node(node1_handle),
        ));

        let edge = &graph.edges()[0];
        assert_eq!(edge.edge_handle(), edge_handle);
        assert_eq!(edge.node1_handle(), node0_handle);
        assert_eq!(edge.node2_handle(), node1_handle);
    }

    #[test]
    fn removing_edge_updates_graph() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();

        let node0_handle = graph.add_node(SimpleGraphNode::new(0));
        let node1_handle = graph.add_node(SimpleGraphNode::new(1));
        let node2_handle = graph.add_node(SimpleGraphNode::new(2));
        let edge01_handle = graph.add_edge(SimpleGraphEdge::new(
            graph.node(node0_handle),
            graph.node(node1_handle),
        ));
        graph.add_edge(SimpleGraphEdge::new(
            graph.node(node1_handle),
            graph.node(node2_handle),
        ));

        graph.remove_edges(&vec![edge01_handle]);

        assert_eq!(graph.edges().len(), 1);
        assert_eq!(
            *graph.edge(EdgeHandle { index: 0 }).graph_edge_data(),
            GraphEdgeData {
                edge_handle: EdgeHandle { index: 0 },
                node1_handle: NodeHandle { index: 1 },
                node2_handle: NodeHandle { index: 2 }
            }
        );
        assert_eq!(
            *graph.node(node0_handle).graph_node_data(),
            GraphNodeData {
                node_handle: NodeHandle { index: 0 },
                edge_handles: vec![]
            }
        );
        assert_eq!(
            *graph.node(node1_handle).graph_node_data(),
            GraphNodeData {
                node_handle: NodeHandle { index: 1 },
                edge_handles: vec![EdgeHandle { index: 0 }]
            }
        );
        assert_eq!(
            *graph.node(node2_handle).graph_node_data(),
            GraphNodeData {
                node_handle: NodeHandle { index: 2 },
                edge_handles: vec![EdgeHandle { index: 0 }]
            }
        );
    }

    #[test]
    fn removing_node_updates_edges() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();

        let node0_handle = graph.add_node(SimpleGraphNode::new(0));
        let node1_handle = graph.add_node(SimpleGraphNode::new(1));
        let node2_handle = graph.add_node(SimpleGraphNode::new(2));
        graph.add_edge(SimpleGraphEdge::new(
            graph.node(node0_handle),
            graph.node(node1_handle),
        ));
        graph.add_edge(SimpleGraphEdge::new(
            graph.node(node1_handle),
            graph.node(node2_handle),
        ));
        graph.add_edge(SimpleGraphEdge::new(
            graph.node(node2_handle),
            graph.node(node0_handle),
        ));

        graph.remove_nodes(&vec![node0_handle]);

        assert_eq!(graph.edges().len(), 1);
        // assert_eq!(
        //     *graph.edge(EdgeHandle { index: 0 }).graph_edge_data(),
        //     GraphEdgeData {
        //         edge_handle: EdgeHandle { index: 0 },
        //         node1_handle: NodeHandle { index: 1 },
        //         node2_handle: NodeHandle { index: 0 }
        //     }
        // );
    }

    #[test]
    fn have_edge() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();

        let node0_handle = graph.add_node(SimpleGraphNode::new(0));
        let node1_handle = graph.add_node(SimpleGraphNode::new(1));
        let node2_handle = graph.add_node(SimpleGraphNode::new(2));
        graph.add_edge(SimpleGraphEdge::new(
            graph.node(node0_handle),
            graph.node(node1_handle),
        ));

        assert!(graph.have_edge(&graph.node(node0_handle), &graph.node(node1_handle)));
        assert!(graph.have_edge(&graph.node(node1_handle), &graph.node(node0_handle)));
        assert!(!graph.have_edge(&graph.node(node0_handle), &graph.node(node2_handle)));
    }

    #[test]
    fn added_meta_edge_has_correct_handles() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();

        let node0_handle = graph.add_node(SimpleGraphNode::new(0));
        let node1_handle = graph.add_node(SimpleGraphNode::new(1));
        let node2_handle = graph.add_node(SimpleGraphNode::new(2));

        let edge01_handle = graph.add_edge(SimpleGraphEdge::new(
            graph.node(node0_handle),
            graph.node(node1_handle),
        ));
        let edge12_handle = graph.add_edge(SimpleGraphEdge::new(
            graph.node(node1_handle),
            graph.node(node2_handle),
        ));

        graph.add_meta_edge(SimpleGraphMetaEdge::new(
            graph.edge(edge01_handle),
            graph.edge(edge12_handle),
        ));

        let meta_edge = &graph.meta_edges()[0];
        assert_eq!(meta_edge.edge1_handle(), edge01_handle);
        assert_eq!(meta_edge.edge2_handle(), edge12_handle);
    }
}
