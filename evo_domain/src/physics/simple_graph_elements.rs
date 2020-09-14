use crate::physics::node_graph::*;
use crate::physics::quantities::*;
use crate::physics::shapes::*;
use evo_domain_derive::*;

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

impl NodeWithHandle<SimpleGraphNode> for SimpleGraphNode {
    fn handle(&self) -> NodeHandle {
        self.graph_node_data.handle()
    }

    fn handle_mut(&mut self) -> &mut NodeHandle {
        self.graph_node_data.handle_mut()
    }
}

impl GraphNode<SimpleGraphNode> for SimpleGraphNode {
    fn node_handle(&self) -> NodeHandle {
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

impl NodeWithHandle<SimpleCircleNode> for SimpleCircleNode {
    fn handle(&self) -> NodeHandle {
        self.graph_node_data.handle()
    }

    fn handle_mut(&mut self) -> &mut NodeHandle {
        self.graph_node_data.handle_mut()
    }
}

impl GraphNode<SimpleCircleNode> for SimpleCircleNode {
    fn node_handle(&self) -> NodeHandle {
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

#[derive(Debug, GraphEdge, PartialEq)]
pub struct SimpleGraphEdge {
    edge_data: GraphEdgeData,
}

impl SimpleGraphEdge {
    pub fn new<N: NodeWithHandle<N>>(node1: &dyn GraphNode<N>, node2: &dyn GraphNode<N>) -> Self {
        SimpleGraphEdge {
            edge_data: GraphEdgeData::new(node1.node_handle(), node2.node_handle()),
        }
    }
}

#[derive(Debug, GraphMetaEdge, PartialEq)]
pub struct SimpleGraphMetaEdge {
    meta_edge_data: GraphMetaEdgeData,
}

impl SimpleGraphMetaEdge {
    pub fn new(edge1: &dyn GraphEdge, edge2: &dyn GraphEdge) -> Self {
        SimpleGraphMetaEdge {
            meta_edge_data: GraphMetaEdgeData::new(edge1.edge_handle(), edge2.edge_handle()),
        }
    }
}
