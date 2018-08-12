use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Bond {
    edge_data: GraphEdgeData,
    handle1: NodeHandle,
    handle2: NodeHandle,
}

impl Bond {
    pub fn new(circle1: &GraphNode, circle2: &GraphNode) -> Self {
        Bond {
            edge_data: GraphEdgeData::new(circle1.node_handle(), circle2.node_handle()),
            handle1: circle1.node_handle(),
            handle2: circle2.node_handle(),
        }
    }

    pub fn calc_strain(&self) -> Displacement {
        Displacement::new(0.0, 0.0)
    }
}

impl GraphEdge for Bond {
    fn node1_handle(&self) -> NodeHandle {
        self.handle1
    }

    fn node2_handle(&self) -> NodeHandle {
        self.handle2
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

pub fn calc_bond_forces<'a, C>(graph: &'a mut SortableGraph<C, Bond>, on_bond_force: fn(&mut C, Force))
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

    for (handle, strain) in strains {
        on_bond_force(graph.node_mut(handle), strain.to_force());
    }
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AngleGusset {
    // TODO
}

impl AngleGusset {
    pub fn new(bond1: &Bond, bond2: &Bond, angle: Angle) -> Self {
        AngleGusset {}
    }
}

pub fn calc_bond_angle_forces<'a, C>(graph: &'a mut SortableGraph<C, Bond>, on_bond_force: fn(&mut C, Force))
    where C: Circle + GraphNode
{
//    let mut strains: Vec<(NodeHandle, BondAngleStrain)> = Vec::with_capacity(graph.edges().len() * 2);

    for gusset in graph.meta_edges() {
//        let circle1 = graph.node(bond.node1_handle());
//        let circle2 = graph.node(bond.node2_handle());

//        let strain = calc_bond_strain(circle1, circle2);
//        strains.push((circle1.node_handle(), BondStrain::new(strain)));
//        strains.push((circle2.node_handle(), BondStrain::new(-strain)));
    }

//    for (handle, strain) in strains {
//        on_bond_force(graph.node_mut(handle), strain.to_force());
//    }
}

// TODO find a better home
fn sqr(x: f64) -> f64 {
    x * x
}

#[cfg(test)]
mod tests {
    use super::*;

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
