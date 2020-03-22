use smallvec::SmallVec;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::u32;

pub const MAX_NODE_EDGES: usize = 8;

#[derive(Debug)]
pub struct SortableGraph<N: GraphNode, E: GraphEdge, ME: GraphMetaEdge> {
    nodes: Vec<N>,
    node_handles: Vec<NodeHandle>,
    edges: Vec<E>,
    meta_edges: Vec<ME>,
}

impl<N: GraphNode, E: GraphEdge, ME: GraphMetaEdge> SortableGraph<N, E, ME> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        SortableGraph {
            nodes: vec![],
            node_handles: vec![],
            edges: vec![],
            meta_edges: vec![],
        }
    }

    pub fn add_node(&mut self, mut node: N) -> NodeHandle {
        let handle = self.next_node_handle();
        node.graph_node_data_mut().handle = handle;
        self.nodes.push(node);
        self.node_handles.push(handle);
        handle
    }

    fn next_node_handle(&self) -> NodeHandle {
        NodeHandle::new(self.nodes.len().try_into().unwrap())
    }

    pub fn add_edge(
        &mut self,
        mut edge: E,
        edge_index_on_node1: usize,
        edge_index_on_node2: usize,
    ) -> EdgeHandle {
        let handle = self.next_edge_handle();
        edge.graph_edge_data_mut().handle = handle;
        self.add_edge_to_node(edge.node1_handle(), handle, edge_index_on_node1);
        self.add_edge_to_node(edge.node2_handle(), handle, edge_index_on_node2);
        self.edges.push(edge);
        handle
    }

    fn next_edge_handle(&self) -> EdgeHandle {
        EdgeHandle::new(self.edges.len().try_into().unwrap())
    }

    fn add_edge_to_node(
        &mut self,
        node_handle: NodeHandle,
        edge_handle: EdgeHandle,
        edge_index: usize,
    ) {
        self.node_mut(node_handle)
            .graph_node_data_mut()
            .set_edge_handle(edge_index, edge_handle);
    }

    pub fn add_meta_edge(&mut self, meta_edge: ME) {
        // TODO unfinished
        //        edge.graph_edge_data_mut().edge_handle.index = self.meta_edges.len();
        //        let edge_handle = edge.edge_handle();
        //        self.add_edge_to_node(edge.node1_handle(), edge_handle);
        //        self.add_edge_to_node(edge.node2_handle(), edge_handle);
        self.meta_edges.push(meta_edge);
    }

    /// Removes the nodes referenced by `handles`.
    ///
    /// Warning: this function has two big gotchas:
    ///
    /// 1) `handles` should be in ascending order of `index`. If not, the function will
    /// panic on index out-of-bounds if we're removing nodes at the end of self.nodes.
    ///
    /// 2) Worse, this function changes the nodes referenced by some of the remaining handles.
    /// Never retain handles across a call to this function.
    pub fn remove_nodes(&mut self, handles: &[NodeHandle]) {
        for handle in handles.iter().rev() {
            self.remove_node(*handle);
        }
        self.remove_obsolete_node_handles();
    }

    /// Warning: invalidates handles to the last node in self.nodes.
    fn remove_node(&mut self, handle: NodeHandle) {
        self.remove_node_edges(&self.node(handle).graph_node_data().edge_handles.clone());
        self.nodes.swap_remove(handle.index());
        self.fix_swapped_node_if_needed(handle);
    }

    fn fix_swapped_node_if_needed(&mut self, handle: NodeHandle) {
        let old_last_handle = self.next_node_handle();
        if handle != old_last_handle {
            self.fix_swapped_node_and_its_edges(old_last_handle, handle);
        }
    }

    fn fix_swapped_node_and_its_edges(&mut self, old_handle: NodeHandle, new_handle: NodeHandle) {
        self.node_mut(new_handle).graph_node_data_mut().handle = new_handle;
        for edge_handle in self
            .node(new_handle)
            .graph_node_data()
            .edge_handles
            .clone()
            .iter()
        {
            if let Some(edge_handle) = edge_handle {
                self.edge_mut(*edge_handle)
                    .graph_edge_data_mut()
                    .replace_node_handle(old_handle, new_handle);
            }
        }
    }

    fn remove_obsolete_node_handles(&mut self) {
        let first_invalid_index = self.next_node_handle().index;
        self.node_handles.retain(|h| h.index < first_invalid_index);
    }

    fn remove_node_edges(&mut self, handles: &[Option<EdgeHandle>]) {
        let mut live_handles: SmallVec<[EdgeHandle; MAX_NODE_EDGES]> =
            handles.iter().filter_map(|h| *h).collect();
        live_handles.sort();
        self.remove_edges(&live_handles);
    }

    /// Same gotchas as in remove_nodes.
    pub fn remove_edges(&mut self, handles: &[EdgeHandle]) {
        for handle in handles.iter().rev() {
            self.remove_edge(*handle);
        }
    }

    /// Warning: invalidates handles to the last edge in self.edges.
    fn remove_edge(&mut self, handle: EdgeHandle) {
        self.remove_edge_from_node(self.edge(handle).node1_handle(), handle);
        self.remove_edge_from_node(self.edge(handle).node2_handle(), handle);
        // TODO remove obsolete meta-edges
        self.edges.swap_remove(handle.index());
        self.fix_swapped_edge_if_needed(handle);
    }

    fn remove_edge_from_node(&mut self, node_handle: NodeHandle, edge_handle: EdgeHandle) {
        self.node_mut(node_handle)
            .graph_node_data_mut()
            .remove_edge_handle(edge_handle);
    }

    fn fix_swapped_edge_if_needed(&mut self, handle: EdgeHandle) {
        let old_last_handle = self.next_edge_handle();
        if handle != old_last_handle {
            self.fix_swapped_edge(old_last_handle, handle);
        }
    }

    fn fix_swapped_edge(&mut self, old_handle: EdgeHandle, new_handle: EdgeHandle) {
        self.edge_mut(new_handle).graph_edge_data_mut().handle = new_handle;
        let edge_data = self.edge(new_handle).graph_edge_data().clone();
        self.replace_edge_handle(edge_data.node1_handle, old_handle, new_handle);
        self.replace_edge_handle(edge_data.node2_handle, old_handle, new_handle);
        // TODO update handles to swapped edges in remaining meta-edges
    }

    fn replace_edge_handle(
        &mut self,
        node_handle: NodeHandle,
        old_handle: EdgeHandle,
        new_handle: EdgeHandle,
    ) {
        self.node_mut(node_handle)
            .graph_node_data_mut()
            .replace_edge_handle(old_handle, new_handle);
    }

    pub fn sort_node_handles(&mut self, cmp: fn(&N, &N) -> Ordering) {
        let nodes = &self.nodes;
        self.node_handles
            .sort_unstable_by(|h1, h2| cmp(&nodes[h1.index()], &nodes[h2.index()]));
    }

    pub fn sort_already_mostly_sorted_node_handles(&mut self, cmp: fn(&N, &N) -> Ordering) {
        let nodes = &self.nodes;
        Self::insertion_sort_by(&mut self.node_handles, |h1, h2| {
            cmp(&nodes[h1.index()], &nodes[h2.index()]) == Ordering::Less
        });
    }

    fn insertion_sort_by<T, F>(seq: &mut [T], mut is_less: F)
    where
        T: Copy,
        F: FnMut(T, T) -> bool,
    {
        for i in 1..seq.len() {
            for j in (1..=i).rev() {
                if is_less(seq[j - 1], seq[j]) {
                    break;
                }
                seq.swap(j - 1, j)
            }
        }
    }

    pub fn have_edge(&self, node1: &N, node2: &N) -> bool {
        node1
            .graph_node_data()
            .edge_handles
            .iter()
            .filter_map(|h| *h)
            .any(|edge_handle| {
                self.edge(edge_handle)
                    .graph_edge_data()
                    .joins(node1.node_handle(), node2.node_handle())
            })
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
        &self.nodes[handle.index()]
    }

    pub fn node_mut(&mut self, handle: NodeHandle) -> &mut N {
        &mut self.nodes[handle.index()]
    }

    pub fn edges(&self) -> &[E] {
        &self.edges
    }

    pub fn edge(&self, handle: EdgeHandle) -> &E {
        &self.edges[handle.index()]
    }

    pub fn edge_mut(&mut self, handle: EdgeHandle) -> &mut E {
        &mut self.edges[handle.index()]
    }

    pub fn meta_edges(&self) -> &[ME] {
        &self.meta_edges
    }
}

pub trait GraphNode {
    fn node_handle(&self) -> NodeHandle;

    fn graph_node_data(&self) -> &GraphNodeData;

    fn graph_node_data_mut(&mut self) -> &mut GraphNodeData;

    fn has_edge(&self, node_edge_index: usize) -> bool;

    fn edge_handle(&self, node_edge_index: usize) -> EdgeHandle;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeHandle {
    index: u32,
}

impl NodeHandle {
    fn new(index: u32) -> Self {
        NodeHandle { index }
    }

    pub fn unset() -> Self {
        NodeHandle { index: u32::MAX }
    }

    fn index(self) -> usize {
        self.index.try_into().unwrap()
    }
}

impl fmt::Display for NodeHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.index)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeData {
    handle: NodeHandle,
    edge_handles: [Option<EdgeHandle>; MAX_NODE_EDGES],
}

impl GraphNodeData {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        GraphNodeData {
            handle: NodeHandle::unset(),
            edge_handles: [None; MAX_NODE_EDGES],
        }
    }

    pub fn handle(&self) -> NodeHandle {
        self.handle
    }

    pub fn has_edge_handle(&self, node_edge_index: usize) -> bool {
        self.edge_handles[node_edge_index] != None
    }

    pub fn edge_handle(&self, edge_index: usize) -> EdgeHandle {
        self.edge_handles[edge_index].unwrap()
    }

    fn set_edge_handle(&mut self, node_edge_index: usize, handle: EdgeHandle) {
        assert_eq!(self.edge_handles[node_edge_index], None);
        self.edge_handles[node_edge_index] = Some(handle);
    }

    fn remove_edge_handle(&mut self, handle: EdgeHandle) {
        for edge_handle in &mut self.edge_handles {
            if *edge_handle == Some(handle) {
                *edge_handle = None;
                break;
            }
        }
    }

    fn replace_edge_handle(&mut self, old_handle: EdgeHandle, new_handle: EdgeHandle) {
        for edge_handle in &mut self.edge_handles {
            if *edge_handle == Some(old_handle) {
                *edge_handle = Some(new_handle);
            }
        }
    }
}

pub trait GraphEdge {
    fn edge_handle(&self) -> EdgeHandle;

    fn node1_handle(&self) -> NodeHandle;

    fn node2_handle(&self) -> NodeHandle;

    fn graph_edge_data(&self) -> &GraphEdgeData;

    fn graph_edge_data_mut(&mut self) -> &mut GraphEdgeData;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EdgeHandle {
    index: u32,
}

impl EdgeHandle {
    fn new(index: u32) -> Self {
        EdgeHandle { index }
    }

    pub fn unset() -> Self {
        EdgeHandle { index: u32::MAX }
    }

    fn index(self) -> usize {
        self.index.try_into().unwrap()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphEdgeData {
    handle: EdgeHandle,
    node1_handle: NodeHandle,
    node2_handle: NodeHandle,
}

impl GraphEdgeData {
    pub fn new(node1_handle: NodeHandle, node2_handle: NodeHandle) -> Self {
        GraphEdgeData {
            handle: EdgeHandle::unset(),
            node1_handle,
            node2_handle,
        }
    }

    pub fn handle(&self) -> EdgeHandle {
        self.handle
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

    fn joins(&self, node_handle1: NodeHandle, node_handle2: NodeHandle) -> bool {
        (self.node1_handle == node_handle1 && self.node2_handle == node_handle2)
            || (self.node1_handle == node_handle2 && self.node2_handle == node_handle1)
    }

    fn replace_node_handle(&mut self, old_handle: NodeHandle, new_handle: NodeHandle) {
        if self.node1_handle == old_handle {
            self.node1_handle.index = new_handle.index;
        }
        if self.node2_handle == old_handle {
            self.node2_handle.index = new_handle.index;
        }
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

        let node = &graph.nodes()[0];
        assert_eq!(node.node_handle(), node_handle);
    }

    #[test]
    fn can_fetch_node_by_handle() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();

        let node_handle = graph.add_node(SimpleGraphNode::new(0));

        let node = &graph.nodes()[0];
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

        assert_eq!(graph.nodes.len(), 1);
        let node = &graph.nodes()[0];
        assert_eq!(node.id, 1);
        assert_eq!(node.node_handle().index, 0);
        assert_eq!(graph.node_handles.len(), 1);
        assert_eq!(graph.node_handles[0].index, 0);
    }

    #[test]
    fn added_edge_has_correct_handles() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();

        let node0_handle = graph.add_node(SimpleGraphNode::new(0));
        let node1_handle = graph.add_node(SimpleGraphNode::new(1));

        let edge_handle = graph.add_edge(
            SimpleGraphEdge::new(graph.node(node0_handle), graph.node(node1_handle)),
            1,
            0,
        );

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
        let edge01_handle = graph.add_edge(
            SimpleGraphEdge::new(graph.node(node0_handle), graph.node(node1_handle)),
            1,
            0,
        );
        graph.add_edge(
            SimpleGraphEdge::new(graph.node(node1_handle), graph.node(node2_handle)),
            1,
            0,
        );

        graph.remove_edges(&vec![edge01_handle]);

        assert_eq!(graph.edges().len(), 1);
        assert_eq!(
            *graph.edge(EdgeHandle { index: 0 }).graph_edge_data(),
            GraphEdgeData {
                handle: EdgeHandle { index: 0 },
                node1_handle: NodeHandle { index: 1 },
                node2_handle: NodeHandle { index: 2 }
            }
        );
        assert_eq!(
            *graph.node(node0_handle).graph_node_data(),
            GraphNodeData {
                handle: NodeHandle { index: 0 },
                edge_handles: [None; MAX_NODE_EDGES]
            }
        );
        assert_eq!(
            *graph.node(node1_handle).graph_node_data(),
            GraphNodeData {
                handle: NodeHandle { index: 1 },
                edge_handles: {
                    let mut handles = [None; MAX_NODE_EDGES];
                    handles[1] = Some(EdgeHandle { index: 0 });
                    handles
                }
            }
        );
        assert_eq!(
            *graph.node(node2_handle).graph_node_data(),
            GraphNodeData {
                handle: NodeHandle { index: 2 },
                edge_handles: {
                    let mut handles = [None; MAX_NODE_EDGES];
                    handles[0] = Some(EdgeHandle { index: 0 });
                    handles
                }
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
        graph.add_edge(
            SimpleGraphEdge::new(graph.node(node0_handle), graph.node(node1_handle)),
            1,
            0,
        );
        graph.add_edge(
            SimpleGraphEdge::new(graph.node(node1_handle), graph.node(node2_handle)),
            1,
            0,
        );
        graph.add_edge(
            SimpleGraphEdge::new(graph.node(node2_handle), graph.node(node0_handle)),
            1,
            0,
        );

        graph.remove_nodes(&vec![node0_handle]);

        assert_eq!(graph.edges().len(), 1);
        assert_eq!(
            *graph.edge(EdgeHandle { index: 0 }).graph_edge_data(),
            GraphEdgeData {
                handle: EdgeHandle { index: 0 },
                node1_handle: NodeHandle { index: 1 },
                node2_handle: NodeHandle { index: 0 }
            }
        );
    }

    #[test]
    fn have_edge() {
        let mut graph: SortableGraph<SimpleGraphNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            SortableGraph::new();

        let node0_handle = graph.add_node(SimpleGraphNode::new(0));
        let node1_handle = graph.add_node(SimpleGraphNode::new(1));
        let node2_handle = graph.add_node(SimpleGraphNode::new(2));
        graph.add_edge(
            SimpleGraphEdge::new(graph.node(node0_handle), graph.node(node1_handle)),
            1,
            0,
        );

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

        let edge01_handle = graph.add_edge(
            SimpleGraphEdge::new(graph.node(node0_handle), graph.node(node1_handle)),
            1,
            0,
        );
        let edge12_handle = graph.add_edge(
            SimpleGraphEdge::new(graph.node(node1_handle), graph.node(node2_handle)),
            1,
            0,
        );

        graph.add_meta_edge(SimpleGraphMetaEdge::new(
            graph.edge(edge01_handle),
            graph.edge(edge12_handle),
        ));

        let meta_edge = &graph.meta_edges()[0];
        assert_eq!(meta_edge.edge1_handle(), edge01_handle);
        assert_eq!(meta_edge.edge2_handle(), edge12_handle);
    }
}
