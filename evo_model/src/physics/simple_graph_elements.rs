use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;

#[derive(Clone, Debug, GraphNode, PartialEq)]
pub struct SimpleGraphNode {
    graph_node_data: GraphNodeData,
}

impl SimpleGraphNode {
    pub fn new() -> Self {
        SimpleGraphNode {
            graph_node_data: GraphNodeData::new(),
        }
    }
}

#[derive(Clone, Debug, GraphNode, PartialEq)]
pub struct SimpleCircleNode {
    graph_node_data: GraphNodeData,
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
        return self.radius;
    }

    fn center(&self) -> Position {
        return self.center;
    }
}

#[derive(Debug, GraphEdge, PartialEq)]
pub struct SimpleGraphEdge {
    edge_data: GraphEdgeData,
}

impl SimpleGraphEdge {
    pub fn new(node1: &GraphNode, node2: &GraphNode) -> Self {
        SimpleGraphEdge {
            edge_data: GraphEdgeData::new(node1.node_handle(), node2.node_handle())
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SimpleGraphMetaEdge {
    meta_edge_data: GraphMetaEdgeData,
}

impl SimpleGraphMetaEdge {
    pub fn new(edge1: &GraphEdge, edge2: &GraphEdge) -> Self {
        SimpleGraphMetaEdge {
            meta_edge_data: GraphMetaEdgeData::new(edge1.edge_handle(), edge2.edge_handle())
        }
    }
}

impl GraphMetaEdge for SimpleGraphMetaEdge {
    fn edge1_handle(&self) -> EdgeHandle {
        self.meta_edge_data.edge1_handle()
    }

    fn edge2_handle(&self) -> EdgeHandle {
        self.meta_edge_data.edge2_handle()
    }

    fn graph_meta_edge_data(&self) -> &GraphMetaEdgeData {
        &self.meta_edge_data
    }

    fn graph_meta_edge_data_mut(&mut self) -> &mut GraphMetaEdgeData {
        &mut self.meta_edge_data
    }
}
