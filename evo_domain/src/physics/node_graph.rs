use crate::physics::nodes_with_handles::*;
use smallvec::SmallVec;
use std::convert::TryInto;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::marker::PhantomData;
use std::u32;

pub const MAX_NODE_EDGES: usize = 8;

#[derive(Debug)]
pub struct NodeGraph<N: GraphNode<N>, E: GraphEdge<N>, ME: GraphMetaEdge> {
    nodes: NodesWithHandles<N>,
    edges: Vec<E>,
    meta_edges: Vec<ME>,
}

impl<N: GraphNode<N>, E: GraphEdge<N>, ME: GraphMetaEdge> NodeGraph<N, E, ME> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        NodeGraph {
            nodes: NodesWithHandles::new(),
            edges: vec![],
            meta_edges: vec![],
        }
    }

    pub fn add_node(&mut self, node: N) -> Handle<N> {
        self.nodes.add_node(node)
    }

    pub fn is_valid_handle(&self, handle: Handle<N>) -> bool {
        self.nodes.is_valid_handle(handle)
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
        node_handle: Handle<N>,
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
    pub fn remove_nodes(&mut self, handles: &[Handle<N>]) {
        for handle in handles {
            self.remove_node_edges(&self.node(*handle).graph_node_data().edge_handles.clone());
        }
        let edges = &mut self.edges;
        self.nodes.remove_nodes(handles, |node, prev_handle| {
            Self::fix_swapped_node_edges(node, prev_handle, node.node_handle(), edges);
        });
    }

    fn fix_swapped_node_edges(
        node: &N,
        old_handle: Handle<N>,
        new_handle: Handle<N>,
        edges: &mut [E],
    ) {
        for edge_handle in node.graph_node_data().edge_handles.clone().iter() {
            if let Some(edge_handle) = edge_handle {
                let edge = &mut edges[edge_handle.index()];
                edge.graph_edge_data_mut()
                    .replace_node_handle(old_handle, new_handle);
            }
        }
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

    fn remove_edge_from_node(&mut self, node_handle: Handle<N>, edge_handle: EdgeHandle) {
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
        let edge_data = self.edge(new_handle).graph_edge_data();
        let node1_handle = edge_data.node1_handle;
        let node2_handle = edge_data.node2_handle;
        self.replace_edge_handle(node1_handle, old_handle, new_handle);
        self.replace_edge_handle(node2_handle, old_handle, new_handle);
        // TODO update handles to swapped edges in remaining meta-edges
    }

    fn replace_edge_handle(
        &mut self,
        node_handle: Handle<N>,
        old_handle: EdgeHandle,
        new_handle: EdgeHandle,
    ) {
        self.node_mut(node_handle)
            .graph_node_data_mut()
            .replace_edge_handle(old_handle, new_handle);
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

    pub fn for_each_node<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, &mut N, &mut EdgeSource<N, E>),
    {
        let mut edge_source = EdgeSource::new(&mut self.edges);
        for (index, node) in self.nodes.nodes_mut().iter_mut().enumerate() {
            f(index, node, &mut edge_source);
        }
    }

    pub fn with_nodes<F>(&mut self, handle1: Handle<N>, handle2: Handle<N>, f: F)
    where
        F: FnMut(&mut N, &mut N),
    {
        self.nodes.with_nodes(handle1, handle2, f);
    }

    pub fn nodes(&self) -> &[N] {
        self.nodes.nodes()
    }

    pub fn nodes_mut(&mut self) -> &mut [N] {
        self.nodes.nodes_mut()
    }

    pub fn node(&self, handle: Handle<N>) -> &N {
        self.nodes.node(handle)
    }

    pub fn node_mut(&mut self, handle: Handle<N>) -> &mut N {
        self.nodes.node_mut(handle)
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

pub struct EdgeSource<'a, N: WithHandle<N>, E: GraphEdge<N>> {
    edges: &'a mut [E],
    _phantom: PhantomData<N>,
}

impl<'a, N: WithHandle<N>, E: GraphEdge<N>> EdgeSource<'a, N, E> {
    fn new(edges: &'a mut [E]) -> Self {
        EdgeSource {
            edges,
            _phantom: PhantomData,
        }
    }

    pub fn edge(&mut self, handle: EdgeHandle) -> &mut E {
        &mut self.edges[handle.index()]
    }
}

pub trait GraphNode<N: WithHandle<N>>: WithHandle<N> {
    fn node_handle(&self) -> Handle<N>;

    fn graph_node_data(&self) -> &GraphNodeData<N>;

    fn graph_node_data_mut(&mut self) -> &mut GraphNodeData<N>;

    fn has_edge(&self, node_edge_index: usize) -> bool;

    fn edge_handle(&self, node_edge_index: usize) -> EdgeHandle;

    fn edge_handles(&self) -> &[Option<EdgeHandle>];
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeData<N: WithHandle<N>> {
    handle: Handle<N>,
    edge_handles: [Option<EdgeHandle>; MAX_NODE_EDGES],
    _phantom: PhantomData<N>, // TODO lose this
}

impl<N: WithHandle<N>> GraphNodeData<N> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        GraphNodeData {
            handle: Handle::unset(),
            edge_handles: [None; MAX_NODE_EDGES],
            _phantom: PhantomData,
        }
    }

    pub fn handle(&self) -> Handle<N> {
        self.handle
    }

    pub fn handle_mut(&mut self) -> &mut Handle<N> {
        &mut self.handle
    }

    pub fn has_edge_handle(&self, node_edge_index: usize) -> bool {
        self.edge_handles[node_edge_index] != None
    }

    pub fn edge_handle(&self, edge_index: usize) -> EdgeHandle {
        self.edge_handles[edge_index].unwrap()
    }

    pub fn edge_handles(&self) -> &[Option<EdgeHandle>] {
        &self.edge_handles
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

pub trait GraphEdge<N: WithHandle<N>> {
    fn edge_handle(&self) -> EdgeHandle;

    fn node1_handle(&self) -> Handle<N>;

    fn node2_handle(&self) -> Handle<N>;

    fn other_node_handle(&self, node_handle: Handle<N>) -> Handle<N>;

    fn graph_edge_data(&self) -> &GraphEdgeData<N>;

    fn graph_edge_data_mut(&mut self) -> &mut GraphEdgeData<N>;
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

impl fmt::Display for EdgeHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.index)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphEdgeData<N: WithHandle<N>> {
    handle: EdgeHandle,
    node1_handle: Handle<N>,
    node2_handle: Handle<N>,
    _phantom: PhantomData<N>, // TODO lose this
}

impl<N: WithHandle<N>> GraphEdgeData<N> {
    pub fn new(node1_handle: Handle<N>, node2_handle: Handle<N>) -> Self {
        GraphEdgeData {
            handle: EdgeHandle::unset(),
            node1_handle,
            node2_handle,
            _phantom: PhantomData,
        }
    }

    pub fn handle(&self) -> EdgeHandle {
        self.handle
    }

    pub fn node1_handle(&self) -> Handle<N> {
        self.node1_handle
    }

    pub fn node1_handle_mut(&mut self) -> &mut Handle<N> {
        &mut self.node1_handle
    }

    pub fn node2_handle(&self) -> Handle<N> {
        self.node2_handle
    }

    pub fn node2_handle_mut(&mut self) -> &mut Handle<N> {
        &mut self.node2_handle
    }

    pub fn other_node_handle(&self, node_handle: Handle<N>) -> Handle<N> {
        if node_handle == self.node1_handle {
            self.node2_handle
        } else {
            self.node1_handle
        }
    }

    fn joins(&self, node_handle1: Handle<N>, node_handle2: Handle<N>) -> bool {
        (self.node1_handle == node_handle1 && self.node2_handle == node_handle2)
            || (self.node1_handle == node_handle2 && self.node2_handle == node_handle1)
    }

    fn replace_node_handle(&mut self, old_handle: Handle<N>, new_handle: Handle<N>) {
        if self.node1_handle == old_handle {
            self.node1_handle = new_handle;
        }
        if self.node2_handle == old_handle {
            self.node2_handle = new_handle;
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
    fn added_edge_has_correct_handles() {
        let mut graph: NodeGraph<
            SimpleGraphNode,
            SimpleGraphEdge<SimpleGraphNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();

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
        let mut graph: NodeGraph<
            SimpleGraphNode,
            SimpleGraphEdge<SimpleGraphNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();

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
                node1_handle: Handle::new(1),
                node2_handle: Handle::new(2),
                _phantom: PhantomData,
            }
        );
        assert_eq!(
            *graph.node(node0_handle).graph_node_data(),
            GraphNodeData {
                handle: Handle::new(0),
                edge_handles: [None; MAX_NODE_EDGES],
                _phantom: PhantomData,
            }
        );
        assert_eq!(
            *graph.node(node1_handle).graph_node_data(),
            GraphNodeData {
                handle: Handle::new(1),
                edge_handles: {
                    let mut handles = [None; MAX_NODE_EDGES];
                    handles[1] = Some(EdgeHandle { index: 0 });
                    handles
                },
                _phantom: PhantomData,
            }
        );
        assert_eq!(
            *graph.node(node2_handle).graph_node_data(),
            GraphNodeData {
                handle: Handle::new(2),
                edge_handles: {
                    let mut handles = [None; MAX_NODE_EDGES];
                    handles[0] = Some(EdgeHandle { index: 0 });
                    handles
                },
                _phantom: PhantomData,
            }
        );
    }

    #[test]
    fn removing_node_updates_edges() {
        let mut graph: NodeGraph<
            SimpleGraphNode,
            SimpleGraphEdge<SimpleGraphNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();

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
                node1_handle: Handle::new(1),
                node2_handle: Handle::new(0),
                _phantom: PhantomData,
            }
        );
    }

    #[test]
    fn have_edge() {
        let mut graph: NodeGraph<
            SimpleGraphNode,
            SimpleGraphEdge<SimpleGraphNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();

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
        let mut graph: NodeGraph<
            SimpleGraphNode,
            SimpleGraphEdge<SimpleGraphNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();

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
