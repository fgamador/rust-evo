use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;
use physics::util::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Bond {
    edge_data: GraphEdgeData
}

impl Bond {
    pub fn new(circle1: &GraphNode, circle2: &GraphNode) -> Self {
        assert_ne!(circle1.node_handle(), circle2.node_handle());
        Bond {
            edge_data: GraphEdgeData::new(circle1.node_handle(), circle2.node_handle())
        }
    }

    pub fn calc_strain(&self) -> Displacement {
        Displacement::new(0.0, 0.0)
    }
}

impl GraphEdge for Bond {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BondStrain
{
    strain: Displacement,
}

impl BondStrain
{
    pub fn new(strain: Displacement) -> Self {
        BondStrain { strain }
    }

    // TODO move this to a Spring class
    pub fn to_force(&self) -> Force {
        const SPRING_CONSTANT: f64 = 1.0;
        Force::new(self.strain.x() * SPRING_CONSTANT, self.strain.y() * SPRING_CONSTANT)
    }
}

pub fn calc_bond_strains<C>(graph: &mut SortableGraph<C, Bond, AngleGusset>) -> Vec<(NodeHandle, BondStrain)>
    where C: Circle + GraphNode
{
    let mut strains: Vec<(NodeHandle, BondStrain)> = Vec::with_capacity(graph.edges().len() * 2);
    for bond in graph.edges() {
        let circle1 = graph.node(bond.node1_handle());
        let circle2 = graph.node(bond.node2_handle());

        let strain = calc_bond_strain(circle1, circle2);
        strains.push((circle1.node_handle(), BondStrain::new(strain)));
        strains.push((circle2.node_handle(), BondStrain::new(-strain)));
    }
    strains
}

fn calc_bond_strain<C>(circle1: &C, circle2: &C) -> Displacement
    where C: Circle
{
    let x_offset = circle1.center().x() - circle2.center().x();
    let y_offset = circle1.center().y() - circle2.center().y();
    let just_touching_center_sep = circle1.radius().value() + circle2.radius().value();
    let center_sep = (sqr(x_offset) + sqr(y_offset)).sqrt();
    if center_sep == 0.0 {
        return Displacement::new(0.0, 0.0);
    }

    let overlap_mag = just_touching_center_sep - center_sep;
    let x_strain = (x_offset / center_sep) * overlap_mag;
    let y_strain = (y_offset / center_sep) * overlap_mag;
    Displacement::new(x_strain, y_strain)
}

#[derive(Clone, Debug, PartialEq)]
pub struct AngleGusset {
    meta_edge_data: GraphMetaEdgeData,
    angle: Angle, // counterclockwise angle from bond1 to bond2
}

impl AngleGusset {
    pub fn new(bond1: &Bond, bond2: &Bond, angle: Angle) -> Self {
        assert_ne!(bond1.edge_handle(), bond2.edge_handle());
        assert_eq!(bond1.node2_handle(), bond2.node1_handle());
        AngleGusset {
            meta_edge_data: GraphMetaEdgeData::new(bond1.edge_handle(), bond2.edge_handle()),
            angle,
        }
    }
}

impl GraphMetaEdge for AngleGusset {
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

pub fn calc_bond_angle_forces_outer<'a, C>(graph: &'a mut SortableGraph<C, Bond, AngleGusset>, on_force: fn(&mut C, Force))
    where C: Circle + GraphNode
{
    let mut forces: Vec<(NodeHandle, Force)> = Vec::with_capacity(graph.meta_edges().len() * 2);

    for gusset in graph.meta_edges() {
        let bond1 = graph.edge(gusset.edge1_handle());
        let bond2 = graph.edge(gusset.edge2_handle());

        let node1 = graph.node(bond1.node1_handle());
        let _node2 = graph.node(bond1.node2_handle());
        let node3 = graph.node(bond2.node2_handle());

        // TODO stub
        forces.push((node1.node_handle(), Force::new(-1.0, 0.0)));
        forces.push((node3.node_handle(), Force::new(-1.0, 0.0)));
    }

    for (node_handle, force) in forces {
        on_force(graph.node_mut(node_handle), force);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    #[should_panic]
    fn cannot_bond_same_ball() {
        let mut graph: SortableGraph<SpyCircle, Bond, AngleGusset> = SortableGraph::new();
        graph.add_node(SpyCircle::new(Position::new(0.0, 0.0), Length::new(1.0)));
        Bond::new(&graph.unsorted_nodes()[0], &graph.unsorted_nodes()[1]);
    }

    #[test]
    fn bond_calculates_strain() {
        // {3, 4, 5} triangle (as {6, 8, 10})
        let circle1 = SpyCircle::new(Position::new(0.0, 0.0), Length::new(2.0));
        let circle2 = SpyCircle::new(Position::new(6.0, 8.0), Length::new(3.0));

        let strain = calc_bond_strain(&circle1, &circle2);

        // strain/hypotenuse 5 has legs 3 and 4
        assert_eq!(Displacement::new(3.0, 4.0), strain);
    }

    #[test]
    fn bonded_pair_with_matching_centers() {
        let circle1 = SpyCircle::new(Position::new(0.0, 0.0), Length::new(1.0));
        let circle2 = SpyCircle::new(Position::new(0.0, 0.0), Length::new(1.0));

        let strain = calc_bond_strain(&circle1, &circle2);

        // what else could we do?
        assert_eq!(Displacement::new(0.0, 0.0), strain);
    }

    #[test]
    #[should_panic]
    fn cannot_gusset_same_bond() {
        let mut graph: SortableGraph<SpyCircle, Bond, AngleGusset> = SortableGraph::new();
        graph.add_node(SpyCircle::new(Position::new(0.0, 0.0), Length::new(1.0)));
        graph.add_node(SpyCircle::new(Position::new(2.0, 0.0), Length::new(1.0)));
        let bond = Bond::new(&graph.unsorted_nodes()[0], &graph.unsorted_nodes()[1]);
        graph.add_edge(bond);
        AngleGusset::new(&graph.edges()[0], &graph.edges()[0], Angle::from_radians(PI));
    }

    #[test]
    #[should_panic]
    fn cannot_gusset_unconnected_bonds() {
        let mut graph: SortableGraph<SpyCircle, Bond, AngleGusset> = SortableGraph::new();
        graph.add_node(SpyCircle::new(Position::new(0.0, 0.0), Length::new(1.0)));
        graph.add_node(SpyCircle::new(Position::new(2.0, 0.0), Length::new(1.0)));
        graph.add_node(SpyCircle::new(Position::new(10.0, 0.0), Length::new(1.0)));
        graph.add_node(SpyCircle::new(Position::new(12.0, 0.0), Length::new(1.0)));
        let bond1 = Bond::new(&graph.unsorted_nodes()[0], &graph.unsorted_nodes()[1]);
        graph.add_edge(bond1);
        let bond2 = Bond::new(&graph.unsorted_nodes()[2], &graph.unsorted_nodes()[3]);
        graph.add_edge(bond2);
        AngleGusset::new(&graph.edges()[0], &graph.edges()[1], Angle::from_radians(PI));
    }

    //#[test]
    fn _gusset_forces() {
        let mut graph: SortableGraph<SpyCircle, Bond, AngleGusset> = SortableGraph::new();
        graph.add_node(SpyCircle::new(Position::new(0.0, 0.0), Length::new(1.0)));
        graph.add_node(SpyCircle::new(Position::new(0.0, -2.0), Length::new(1.0)));
        graph.add_node(SpyCircle::new(Position::new(2.0, -2.0), Length::new(1.0)));
        let bond1 = Bond::new(&graph.unsorted_nodes()[0], &graph.unsorted_nodes()[1]);
        graph.add_edge(bond1);
        let bond2 = Bond::new(&graph.unsorted_nodes()[1], &graph.unsorted_nodes()[2]);
        graph.add_edge(bond2);
        AngleGusset::new(&graph.edges()[0], &graph.edges()[1], Angle::from_radians(PI));
        // TODO
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct SpyCircle {
        graph_node_data: GraphNodeData,
        center: Position,
        radius: Length,
        pub strain: Displacement,
    }

    impl SpyCircle {
        pub fn new(center: Position, radius: Length) -> SpyCircle {
            SpyCircle {
                graph_node_data: GraphNodeData::new(),
                center,
                radius,
                strain: Displacement::new(0.0, 0.0),
            }
        }
    }

    impl Circle for SpyCircle {
        fn radius(&self) -> Length {
            return self.radius;
        }

        fn center(&self) -> Position {
            return self.center;
        }
    }

    impl GraphNode for SpyCircle {
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
