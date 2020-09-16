use crate::physics::node_graph::*;
use crate::physics::nodes_with_handles::*;
use crate::physics::quantities::*;
use crate::physics::shapes::*;
use evo_domain_derive::*;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleGraphNode {
    graph_node_data: GraphNodeData<SimpleGraphNode>,
    pub id: i32,
}

impl SimpleGraphNode {
    pub fn new(id: i32) -> Self {
        SimpleGraphNode {
            graph_node_data: GraphNodeData::new(),
            id,
        }
    }
}

impl WithHandle<SimpleGraphNode> for SimpleGraphNode {
    fn handle(&self) -> Handle<SimpleGraphNode> {
        self.graph_node_data.handle()
    }

    fn handle_mut(&mut self) -> &mut Handle<SimpleGraphNode> {
        self.graph_node_data.handle_mut()
    }
}

impl GraphNode<SimpleGraphNode> for SimpleGraphNode {
    fn node_handle(&self) -> Handle<SimpleGraphNode> {
        self.graph_node_data.handle()
    }

    fn graph_node_data(&self) -> &GraphNodeData<SimpleGraphNode> {
        &self.graph_node_data
    }

    fn graph_node_data_mut(&mut self) -> &mut GraphNodeData<SimpleGraphNode> {
        &mut self.graph_node_data
    }

    fn has_edge(&self, node_edge_index: usize) -> bool {
        self.graph_node_data.has_edge_handle(node_edge_index)
    }

    fn edge_handle(&self, node_edge_index: usize) -> EdgeHandle {
        self.graph_node_data.edge_handle(node_edge_index)
    }

    fn edge_handles(&self) -> &[Option<EdgeHandle>] {
        self.graph_node_data.edge_handles()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimpleCircleNode {
    graph_node_data: GraphNodeData<SimpleCircleNode>,
    center: Position,
    radius: Length,
}

impl SimpleCircleNode {
    pub fn new(center: Position, radius: Length) -> Self {
        SimpleCircleNode {
            graph_node_data: GraphNodeData::new(),
            center,
            radius,
        }
    }

    pub fn set_center(&mut self, pos: Position) {
        self.center = pos;
    }
}

impl Circle for SimpleCircleNode {
    fn radius(&self) -> Length {
        self.radius
    }

    fn center(&self) -> Position {
        self.center
    }
}

impl WithHandle<SimpleCircleNode> for SimpleCircleNode {
    fn handle(&self) -> Handle<SimpleCircleNode> {
        self.graph_node_data.handle()
    }

    fn handle_mut(&mut self) -> &mut Handle<SimpleCircleNode> {
        self.graph_node_data.handle_mut()
    }
}

impl GraphNode<SimpleCircleNode> for SimpleCircleNode {
    fn node_handle(&self) -> Handle<SimpleCircleNode> {
        self.graph_node_data.handle()
    }

    fn graph_node_data(&self) -> &GraphNodeData<SimpleCircleNode> {
        &self.graph_node_data
    }

    fn graph_node_data_mut(&mut self) -> &mut GraphNodeData<SimpleCircleNode> {
        &mut self.graph_node_data
    }

    fn has_edge(&self, node_edge_index: usize) -> bool {
        self.graph_node_data.has_edge_handle(node_edge_index)
    }

    fn edge_handle(&self, node_edge_index: usize) -> EdgeHandle {
        self.graph_node_data.edge_handle(node_edge_index)
    }

    fn edge_handles(&self) -> &[Option<EdgeHandle>] {
        self.graph_node_data.edge_handles()
    }
}

#[derive(Debug, PartialEq)]
pub struct SimpleGraphEdge<N: WithHandle<N>> {
    edge_data: GraphEdgeData<N>,
    _phantom: PhantomData<N>, // TODO lose this
}

impl<N: WithHandle<N>> SimpleGraphEdge<N> {
    pub fn new(node1: &dyn GraphNode<N>, node2: &dyn GraphNode<N>) -> Self {
        SimpleGraphEdge {
            edge_data: GraphEdgeData::new(node1.node_handle(), node2.node_handle()),
            _phantom: PhantomData,
        }
    }
}

impl<N: WithHandle<N>> GraphEdge<N> for SimpleGraphEdge<N> {
    fn edge_handle(&self) -> EdgeHandle {
        self.edge_data.handle()
    }

    fn node1_handle(&self) -> Handle<N> {
        self.edge_data.node1_handle()
    }

    fn node2_handle(&self) -> Handle<N> {
        self.edge_data.node2_handle()
    }

    fn other_node_handle(&self, node_handle: Handle<N>) -> Handle<N> {
        self.edge_data.other_node_handle(node_handle)
    }

    fn graph_edge_data(&self) -> &GraphEdgeData<N> {
        &self.edge_data
    }

    fn graph_edge_data_mut(&mut self) -> &mut GraphEdgeData<N> {
        &mut self.edge_data
    }
}

#[derive(Debug, GraphMetaEdge, PartialEq)]
pub struct SimpleGraphMetaEdge {
    meta_edge_data: GraphMetaEdgeData,
}

impl SimpleGraphMetaEdge {
    pub fn new<N: WithHandle<N>>(edge1: &dyn GraphEdge<N>, edge2: &dyn GraphEdge<N>) -> Self {
        SimpleGraphMetaEdge {
            meta_edge_data: GraphMetaEdgeData::new(edge1.edge_handle(), edge2.edge_handle()),
        }
    }
}
