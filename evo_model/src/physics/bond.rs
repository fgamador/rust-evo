//use physics::ball::*;
use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bond {
    handle1: NodeHandle,
    handle2: NodeHandle,
}

impl Bond {
    pub fn new(ball1: &GraphNode, ball2: &GraphNode) -> Self {
        Bond {
            handle1: ball1.handle(),
            handle2: ball2.handle(),
        }
    }

    pub fn calc_strain(&self) -> Displacement {
        Displacement::new(0.0, 0.0)
    }
}

impl GraphEdge for Bond {
    fn handle1(&self) -> NodeHandle {
        self.handle1
    }

    fn handle1_mut(&mut self) -> &mut NodeHandle {
        &mut self.handle1
    }

    fn handle2(&self) -> NodeHandle {
        self.handle2
    }

    fn handle2_mut(&mut self) -> &mut NodeHandle {
        &mut self.handle2
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

pub fn calc_bond_forces<'a, C>(graph: &'a mut SortableGraph<C, Bond>, on_force: fn(&mut C, Force))
    where C: Circle + GraphNode
{
    let mut strains: Vec<(NodeHandle, BondStrain)> = Vec::with_capacity(graph.edges().len() * 2);

    for bond in graph.edges() {
        let ball1 = graph.node(bond.handle1());
        let ball2 = graph.node(bond.handle2());

        // TODO hard-coded
        let strain = Displacement::new(0.1, 0.1);
        strains.push((ball1.handle(), BondStrain::new(strain)));
        strains.push((ball2.handle(), BondStrain::new(-strain)));
    }

    for (handle, strain) in strains {
        on_force(graph.node_mut(handle), strain.to_force());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics::quantities::*;

    //#[test]
    fn bond_calculates_strain() {
        // {3, 4, 5} triangle (as {6, 8, 10})
        let circle1 = SpyCircle::new(Position::new(0.0, 0.0), Length::new(2.0));
        let circle2 = SpyCircle::new(Position::new(6.0, 8.0), Length::new(3.0));
        let bond = Bond::new(&circle1, &circle2);

        let strain = bond.calc_strain();

        // overlap/hypotenuse 5 has legs 3 and 4
        assert_eq!(Displacement::new(3.0, 4.0), strain);
    }

//    // TODO redundant
//    #[test]
//    fn new_bond_has_correct_ball_handles() {
//        let mut graph: SortableGraph<Ball, Bond> = SortableGraph::new();
//
//        let h1 = graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
//                                          Position::new(1.0, 1.0), Velocity::new(1.0, 1.0)));
//        let h2 = graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
//                                          Position::new(1.0, 1.0), Velocity::new(1.0, 1.0)));
//
//        let bond = Bond::new(graph.node(h1), graph.node(h2));
//        graph.add_edge(bond);
//
//        let ball1 = &graph.nodes()[0];
//        let ball2 = &graph.nodes()[1];
//        assert_eq!(ball1, graph.node(bond.handle1()));
//        assert_eq!(ball2, graph.node(bond.handle2()));
//    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct SpyCircle {
        handle: NodeHandle,
        center: Position,
        radius: Length,
        pub strain: Displacement,
    }

    impl SpyCircle {
        pub fn new(center: Position, radius: Length) -> SpyCircle {
            SpyCircle {
                handle: NodeHandle::unset(),
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
        fn handle(&self) -> NodeHandle {
            self.handle
        }

        fn handle_mut(&mut self) -> &mut NodeHandle {
            &mut self.handle
        }
    }
}
