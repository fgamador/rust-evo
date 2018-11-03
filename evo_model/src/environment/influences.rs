use environment::environment::*;
use physics::bond::*;
use physics::newtonian::*;
use physics::overlap::*;
use physics::quantities::*;
use physics::shapes::Circle;
use physics::sortable_graph::*;

pub trait Influence<T>
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    fn apply(&self, ball_graph: &mut SortableGraph<T, Bond, AngleGusset>);
}

#[derive(Debug)]
pub struct WallCollisions {
    walls: Walls,
}

impl WallCollisions {
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        WallCollisions {
            walls: Walls::new(min_corner, max_corner),
        }
    }
}

impl<T> Influence<T> for WallCollisions
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    fn apply(&self, ball_graph: &mut SortableGraph<T, Bond, AngleGusset>) {
        let overlaps = self.walls.find_overlaps(ball_graph);
        for (handle, overlap) in overlaps {
            let ball = ball_graph.node_mut(handle);
            ball.environment_mut().add_overlap(overlap);
            ball.forces_mut().add_force(overlap.to_force());
        }
    }
}

#[derive(Debug)]
pub struct PairCollisions {}

impl PairCollisions {
    pub fn new() -> Self {
        PairCollisions {}
    }
}

impl<T> Influence<T> for PairCollisions
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    fn apply(&self, ball_graph: &mut SortableGraph<T, Bond, AngleGusset>) {
        let overlaps = find_pair_overlaps(ball_graph);
        for (handle, overlap) in overlaps {
            let ball = ball_graph.node_mut(handle);
            ball.environment_mut().add_overlap(overlap);
            ball.forces_mut().add_force(overlap.to_force());
        }
    }
}

#[derive(Debug)]
pub struct BondForces {}

impl BondForces {
    pub fn new() -> Self {
        BondForces {}
    }
}

impl<T> Influence<T> for BondForces
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    fn apply(&self, ball_graph: &mut SortableGraph<T, Bond, AngleGusset>) {
        let strains = calc_bond_strains(ball_graph);
        for (handle, strain) in strains {
            let ball = ball_graph.node_mut(handle);
            ball.forces_mut().add_force(strain.to_force());
        }
    }
}

#[derive(Debug)]
pub struct BondAngleForces {}

impl BondAngleForces {
    pub fn new() -> Self {
        BondAngleForces {}
    }
}

impl<T> Influence<T> for BondAngleForces
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    fn apply(&self, ball_graph: &mut SortableGraph<T, Bond, AngleGusset>) {
        let forces = calc_bond_angle_forces(ball_graph);
        for (handle, force) in forces {
            let ball = ball_graph.node_mut(handle);
            ball.forces_mut().add_force(force);
        }
    }
}

#[derive(Debug)]
pub struct Weight {
    gravity: Acceleration,
}

impl Weight {
    pub fn new(gravity: f64) -> Self {
        Weight {
            gravity: Acceleration::new(0.0, gravity)
        }
    }

    fn calc_force<T>(&self, ball: &T) -> Force
        where T: NewtonianBody
    {
        ball.mass() * self.gravity
    }

    fn add_force<T>(&self, ball: &mut T)
        where T: NewtonianBody
    {
        let force = self.calc_force(ball);
        ball.forces_mut().add_force(force);
    }
}

impl<T> Influence<T> for Weight
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    fn apply(&self, ball_graph: &mut SortableGraph<T, Bond, AngleGusset>) {
        for ball in ball_graph.unsorted_nodes_mut() {
            self.add_force(ball);
        }
    }
}

#[derive(Debug)]
pub struct Buoyancy {
    gravity: Acceleration,
    fluid_density: Density,
}

impl Buoyancy {
    pub fn new(gravity: f64, fluid_density: f64) -> Self {
        Buoyancy {
            gravity: Acceleration::new(0.0, gravity),
            fluid_density: Density::new(fluid_density),
        }
    }

    fn add_force<T>(&self, ball: &mut T)
        where T: Circle + NewtonianBody
    {
        let displaced_fluid_mass = ball.area() * self.fluid_density;
        ball.forces_mut().add_force(-(displaced_fluid_mass * self.gravity));
    }
}

impl<T> Influence<T> for Buoyancy
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    fn apply(&self, ball_graph: &mut SortableGraph<T, Bond, AngleGusset>) {
        for ball in ball_graph.unsorted_nodes_mut() {
            self.add_force(ball);
        }
    }
}

#[derive(Debug)]
pub struct UniversalOverlap {
    overlap: Overlap
}

impl UniversalOverlap {
    pub fn new(overlap: Overlap) -> Self {
        UniversalOverlap {
            overlap
        }
    }
}

impl<T> Influence<T> for UniversalOverlap
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    fn apply(&self, ball_graph: &mut SortableGraph<T, Bond, AngleGusset>) {
        for ball in ball_graph.unsorted_nodes_mut() {
            ball.environment_mut().add_overlap(self.overlap);
        }
    }
}

#[derive(Debug)]
pub struct UniversalForce {
    force: Force
}

impl UniversalForce {
    pub fn new(force: Force) -> Self {
        UniversalForce {
            force
        }
    }
}

impl<T> Influence<T> for UniversalForce
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    fn apply(&self, ball_graph: &mut SortableGraph<T, Bond, AngleGusset>) {
        for ball in ball_graph.unsorted_nodes_mut() {
            ball.forces_mut().add_force(self.force);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics::ball::Ball;
    use std::f64::consts::PI;

    #[test]
    fn wall_collisions_add_overlap_and_force() {
        let mut ball_graph = SortableGraph::new();
        let wall_collisions = WallCollisions::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        let ball_handle = ball_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                        Position::new(9.5, 9.5), Velocity::new(1.0, 1.0)));

        wall_collisions.apply(&mut ball_graph);

        let ball = ball_graph.node(ball_handle);
        assert_eq!(1, ball.environment().overlaps().len());
        assert_ne!(0.0, ball.forces().net_force().x());
        assert_ne!(0.0, ball.forces().net_force().y());
    }

    #[test]
    fn pair_collisions_add_overlaps_and_forces() {
        let mut ball_graph = SortableGraph::new();
        let pair_collisions = PairCollisions::new();
        let ball1_handle = ball_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(0.0, 0.0), Velocity::new(1.0, 1.0)));
        let ball2_handle = ball_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(1.4, 1.4), Velocity::new(-1.0, -1.0)));

        pair_collisions.apply(&mut ball_graph);

        let ball1 = ball_graph.node(ball1_handle);
        assert_eq!(1, ball1.environment().overlaps().len());
        assert_ne!(0.0, ball1.forces().net_force().x());
        assert_ne!(0.0, ball1.forces().net_force().y());

        let ball2 = ball_graph.node(ball2_handle);
        assert_eq!(1, ball2.environment().overlaps().len());
        assert_ne!(0.0, ball2.forces().net_force().x());
        assert_ne!(0.0, ball2.forces().net_force().y());
    }

    #[test]
    fn bond_forces_add_forces() {
        let mut ball_graph = SortableGraph::new();
        let bond_forces = BondForces::new();
        let ball1_handle = ball_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(0.0, 0.0), Velocity::new(-1.0, -1.0)));
        let ball2_handle = ball_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(1.5, 1.5), Velocity::new(1.0, 1.0)));
        let bond = Bond::new(ball_graph.node(ball1_handle), ball_graph.node(ball2_handle));
        ball_graph.add_edge(bond);

        bond_forces.apply(&mut ball_graph);

        let ball1 = ball_graph.node(ball1_handle);
        assert_ne!(0.0, ball1.forces().net_force().x());
        assert_ne!(0.0, ball1.forces().net_force().y());

        let ball2 = ball_graph.node(ball2_handle);
        assert_ne!(0.0, ball2.forces().net_force().x());
        assert_ne!(0.0, ball2.forces().net_force().y());
    }

    #[test]
    fn bond_angle_forces_add_forces() {
        let mut ball_graph = SortableGraph::new();

        let ball1_handle = ball_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(0.1, 2.0), Velocity::new(0.0, 0.0)));
        let ball2_handle = ball_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        let ball3_handle = ball_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(0.0, -2.0), Velocity::new(0.0, 0.0)));

        let bond = Bond::new(ball_graph.node(ball1_handle), ball_graph.node(ball2_handle));
        let bond1_handle = ball_graph.add_edge(bond);
        let bond = Bond::new(ball_graph.node(ball2_handle), ball_graph.node(ball3_handle));
        let bond2_handle = ball_graph.add_edge(bond);

        let gusset = AngleGusset::new(ball_graph.edge(bond1_handle), ball_graph.edge(bond2_handle), Angle::from_radians(PI));
        ball_graph.add_meta_edge(gusset);

        BondAngleForces::new().apply(&mut ball_graph);

        let ball3 = ball_graph.node(ball3_handle);
        assert!(ball3.forces().net_force().x() < 0.0);
    }

    #[test]
    fn weight_adds_force_proportional_to_mass() {
        let mut ball_graph = SortableGraph::new();
        let weight = Weight::new(-2.0);
        let ball_handle = ball_graph.add_node(Ball::new(Length::new(1.0), Mass::new(3.0),
                                                        Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));

        weight.apply(&mut ball_graph);

        let ball = ball_graph.node(ball_handle);
        assert_eq!(0.0, ball.forces().net_force().x());
        assert_eq!(-6.0, ball.forces().net_force().y());
    }

    #[test]
    fn buoyancy_adds_force_proportional_to_area() {
        let mut ball_graph = SortableGraph::new();
        let buoyancy = Buoyancy::new(-2.0, 2.0);
        let ball_handle = ball_graph.add_node(Ball::new(Length::new(2.0 / PI.sqrt()), Mass::new(1.0),
                                                        Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));

        buoyancy.apply(&mut ball_graph);

        let ball = ball_graph.node(ball_handle);
        assert_eq!(0.0, ball.forces().net_force().x());
        assert_eq!(16.0, ball.forces().net_force().y().round());
    }
}
