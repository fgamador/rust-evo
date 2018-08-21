use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;

#[derive(Clone, Debug, PartialEq)]
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

impl GraphNode for SimpleGraphNode {
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

#[derive(Clone, Debug, PartialEq)]
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
}

impl Circle for SimpleCircleNode {
    fn radius(&self) -> Length {
        return self.radius;
    }

    fn center(&self) -> Position {
        return self.center;
    }
}

impl GraphNode for SimpleCircleNode {
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

#[derive(Debug, PartialEq)]
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

impl GraphEdge for SimpleGraphEdge {
    fn edge_handle(&self) -> EdgeHandle {
        self.edge_data.edge_handle()
    }

    fn node1_handle(&self) -> NodeHandle {
        self.edge_data.node1_handle()
    }

    fn node2_handle(&self) -> NodeHandle {
        self.edge_data.node2_handle()
    }

    fn graph_edge_data(&self) -> &GraphEdgeData {
        &self.edge_data
    }

    fn graph_edge_data_mut(&mut self) -> &mut GraphEdgeData {
        &mut self.edge_data
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
