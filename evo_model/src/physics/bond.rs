use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;
use physics::util::*;

#[derive(Clone, Debug, GraphEdge, PartialEq)]
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

pub fn calc_bond_strains<C>(graph: &SortableGraph<C, Bond, AngleGusset>) -> Vec<(NodeHandle, BondStrain)>
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

#[derive(Clone, Debug, GraphMetaEdge, PartialEq)]
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

pub fn calc_bond_angle_forces<C>(graph: &SortableGraph<C, Bond, AngleGusset>) -> Vec<(NodeHandle, Force)>
    where C: Circle + GraphNode
{
    let mut forces: Vec<(NodeHandle, Force)> = Vec::with_capacity(graph.meta_edges().len() * 2);
    for gusset in graph.meta_edges() {
        let force_pair = calc_bond_angle_force_pair(gusset, graph);
        forces.push(force_pair.0);
        forces.push(force_pair.1);
    }
    forces
}

fn calc_bond_angle_force_pair<C>(gusset: &AngleGusset, graph: &SortableGraph<C, Bond, AngleGusset>)
                                 -> ((NodeHandle, Force), (NodeHandle, Force))
    where C: Circle + GraphNode
{
    let bond1 = graph.edge(gusset.edge1_handle());
    let bond2 = graph.edge(gusset.edge2_handle());

    let node1 = graph.node(bond1.node1_handle());
    let _node2 = graph.node(bond1.node2_handle());
    let node3 = graph.node(bond2.node2_handle());

    // TODO stub
    ((node1.node_handle(), calc_force_from_tangential_force(1.0)),
     (node3.node_handle(), calc_force_from_tangential_force(-1.0)))
}

fn calc_force_from_tangential_force(tangential_force: f64) -> Force {
    // TODO stub
    Force::new(-1.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics::simple_graph_elements::*;
    use std::f64::consts::PI;

    #[test]
    #[should_panic]
    fn cannot_bond_same_ball() {
        let mut graph: SortableGraph<SimpleCircleNode, Bond, AngleGusset> = SortableGraph::new();
        let node = add_simple_circle_node(&mut graph, (0.0, 0.0), 1.0);
        add_bond(&mut graph, node, node);
    }

    #[test]
    fn bond_calculates_strain() {
        // {3, 4, 5} triangle (as {6, 8, 10})
        let circle1 = SimpleCircle::new(Position::new(0.0, 0.0), Length::new(2.0));
        let circle2 = SimpleCircle::new(Position::new(6.0, 8.0), Length::new(3.0));

        let strain = calc_bond_strain(&circle1, &circle2);

        // strain/hypotenuse 5 has legs 3 and 4
        assert_eq!(Displacement::new(3.0, 4.0), strain);
    }

    #[test]
    fn bonded_pair_with_matching_centers_has_no_strain() {
        let circle1 = SimpleCircle::new(Position::new(0.0, 0.0), Length::new(1.0));
        let circle2 = SimpleCircle::new(Position::new(0.0, 0.0), Length::new(1.0));

        let strain = calc_bond_strain(&circle1, &circle2);

        // what else could we do?
        assert_eq!(Displacement::new(0.0, 0.0), strain);
    }

    #[test]
    #[should_panic]
    fn cannot_gusset_same_bond() {
        let mut graph: SortableGraph<SimpleCircleNode, Bond, AngleGusset> = SortableGraph::new();
        let node1 = add_simple_circle_node(&mut graph, (0.0, 0.0), 1.0);
        let node2 = add_simple_circle_node(&mut graph, (2.0, 0.0), 1.0);
        let bond = add_bond(&mut graph, node1, node2);
        add_angle_gusset(&mut graph, bond, bond, PI);
    }

    #[test]
    #[should_panic]
    fn cannot_gusset_unconnected_bonds() {
        let mut graph: SortableGraph<SimpleCircleNode, Bond, AngleGusset> = SortableGraph::new();
        let node1 = add_simple_circle_node(&mut graph, (0.0, 0.0), 1.0);
        let node2 = add_simple_circle_node(&mut graph, (2.0, 0.0), 1.0);
        let node3 = add_simple_circle_node(&mut graph, (10.0, 0.0), 1.0);
        let node4 = add_simple_circle_node(&mut graph, (12.0, 0.0), 1.0);
        let bond1 = add_bond(&mut graph, node1, node2);
        let bond2 = add_bond(&mut graph, node3, node4);
        add_angle_gusset(&mut graph, bond1, bond2, PI);
    }

    #[test]
    fn qualitative_gusset_forces() {
        let mut graph: SortableGraph<SimpleCircleNode, Bond, AngleGusset> = SortableGraph::new();
        let node1 = add_simple_circle_node(&mut graph, (0.01, 2.0), 1.0);
        let node2 = add_simple_circle_node(&mut graph, (0.0, 0.0), 1.0);
        let node3 = add_simple_circle_node(&mut graph, (0.01, -2.0), 1.0);
        let bond1 = add_bond(&mut graph, node1, node2);
        let bond2 = add_bond(&mut graph, node2, node3);
        let gusset = add_angle_gusset(&mut graph, bond1, bond2, PI);

        let force_pair = calc_bond_angle_force_pair(&gusset, &graph);

        assert_eq!(node1, (force_pair.0).0);
        assert!((force_pair.0).1.x() < 0.0);
        assert_eq!(node3, (force_pair.1).0);
        assert!((force_pair.1).1.x() < 0.0);
    }

    fn add_simple_circle_node(graph: &mut SortableGraph<SimpleCircleNode, Bond, AngleGusset>,
                              center: (f64, f64), radius: f64) -> NodeHandle {
        graph.add_node(SimpleCircleNode::new(Position::new(center.0, center.1), Length::new(radius)))
    }

    fn add_bond(graph: &mut SortableGraph<SimpleCircleNode, Bond, AngleGusset>,
                node1: NodeHandle, node2: NodeHandle) -> EdgeHandle {
        let bond = Bond::new(graph.node(node1), graph.node(node2));
        graph.add_edge(bond)
    }

    fn add_angle_gusset(graph: &mut SortableGraph<SimpleCircleNode, Bond, AngleGusset>,
                        bond1: EdgeHandle, bond2: EdgeHandle, radians: f64) -> AngleGusset {
        AngleGusset::new(graph.edge(bond1), graph.edge(bond2), Angle::from_radians(radians))
    }
}
