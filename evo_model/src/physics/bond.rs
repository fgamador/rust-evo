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
    pub fn new(circle1: &GraphNode, circle2: &GraphNode) -> Self {
        Bond {
            handle1: circle1.handle(),
            handle2: circle2.handle(),
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

pub fn calc_bond_forces<'a, C>(graph: &'a mut SortableGraph<C, Bond>, on_bond_force: fn(&mut C, Force))
    where C: Circle + GraphNode
{
    let mut strains: Vec<(NodeHandle, BondStrain)> = Vec::with_capacity(graph.edges().len() * 2);

    for bond in graph.edges() {
        let circle1 = graph.node(bond.handle1());
        let circle2 = graph.node(bond.handle2());

        let strain = calc_bond_strain(circle1, circle2);
        strains.push((circle1.handle(), BondStrain::new(strain)));
        strains.push((circle2.handle(), BondStrain::new(-strain)));
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
