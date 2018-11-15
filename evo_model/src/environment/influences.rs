use environment::environment::*;
use physics::bond::*;
use physics::newtonian::*;
use physics::overlap::*;
use physics::quantities::*;
use physics::shapes::Circle;
use physics::sortable_graph::*;
use physics::util::*;

pub trait Influence<C>
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody
{
    fn apply(&self, cell_graph: &mut SortableGraph<C, Bond, AngleGusset>);
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

impl<C> Influence<C> for WallCollisions
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody
{
    fn apply(&self, cell_graph: &mut SortableGraph<C, Bond, AngleGusset>) {
        let overlaps = self.walls.find_overlaps(cell_graph);
        for (handle, overlap) in overlaps {
            let ball = cell_graph.node_mut(handle);
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

impl<C> Influence<C> for PairCollisions
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody
{
    fn apply(&self, cell_graph: &mut SortableGraph<C, Bond, AngleGusset>) {
        let overlaps = find_pair_overlaps(cell_graph);
        for (handle, overlap) in overlaps {
            let ball = cell_graph.node_mut(handle);
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

impl<C> Influence<C> for BondForces
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody
{
    fn apply(&self, cell_graph: &mut SortableGraph<C, Bond, AngleGusset>) {
        let strains = calc_bond_strains(cell_graph);
        for (handle, strain) in strains {
            let ball = cell_graph.node_mut(handle);
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

impl<C> Influence<C> for BondAngleForces
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody
{
    fn apply(&self, cell_graph: &mut SortableGraph<C, Bond, AngleGusset>) {
        let forces = calc_bond_angle_forces(cell_graph);
        for (handle, force) in forces {
            let ball = cell_graph.node_mut(handle);
            ball.forces_mut().add_force(force);
        }
    }
}

pub struct SimpleForceInfluence<C>
    where C: Circle + HasLocalEnvironment + NewtonianBody
{
    influence_force: Box<SimpleInfluenceForce<C>>
}

impl<C> SimpleForceInfluence<C>
    where C: Circle + HasLocalEnvironment + NewtonianBody
{
    pub fn new(influence_force: Box<SimpleInfluenceForce<C>>) -> Self {
        SimpleForceInfluence {
            influence_force
        }
    }
}

impl<C> Influence<C> for SimpleForceInfluence<C>
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody
{
    fn apply(&self, cell_graph: &mut SortableGraph<C, Bond, AngleGusset>) {
        for ball in cell_graph.unsorted_nodes_mut() {
            let force = self.influence_force.calc_force(ball);
            ball.forces_mut().add_force(force);
        }
    }
}

pub trait SimpleInfluenceForce<C>
    where C: Circle + HasLocalEnvironment + NewtonianBody
{
    fn calc_force(&self, ball: &C) -> Force;
}

#[derive(Debug)]
pub struct ConstantForce {
    force: Force
}

impl ConstantForce {
    pub fn new(force: Force) -> Self {
        ConstantForce {
            force
        }
    }
}

impl<C> SimpleInfluenceForce<C> for ConstantForce
    where C: Circle + HasLocalEnvironment + NewtonianBody
{
    fn calc_force(&self, _ball: &C) -> Force {
        self.force
    }
}

#[derive(Debug)]
pub struct WeightForce {
    gravity: Acceleration,
}

impl WeightForce {
    pub fn new(gravity: f64) -> Self {
        WeightForce {
            gravity: Acceleration::new(0.0, gravity)
        }
    }
}

impl<C> SimpleInfluenceForce<C> for WeightForce
    where C: Circle + HasLocalEnvironment + NewtonianBody
{
    fn calc_force(&self, ball: &C) -> Force {
        ball.mass() * self.gravity
    }
}

#[derive(Debug)]
pub struct BuoyancyForce {
    gravity: Acceleration,
    fluid_density: Density,
}

impl BuoyancyForce {
    pub fn new(gravity: f64, fluid_density: f64) -> Self {
        BuoyancyForce {
            gravity: Acceleration::new(0.0, gravity),
            fluid_density: Density::new(fluid_density),
        }
    }
}

impl<C> SimpleInfluenceForce<C> for BuoyancyForce
    where C: Circle + HasLocalEnvironment + NewtonianBody
{
    fn calc_force(&self, ball: &C) -> Force
    {
        let displaced_fluid_mass = ball.area() * self.fluid_density;
        -(displaced_fluid_mass * self.gravity)
    }
}

#[derive(Debug)]
pub struct DragForce {
    viscosity: f64
}

impl DragForce {
    pub fn new(viscosity: f64) -> Self {
        DragForce {
            viscosity
        }
    }

    fn calc_drag(&self, radius: f64, velocity: f64) -> f64
    {
        -velocity.signum() * self.viscosity * radius * sqr(velocity)
    }
}

impl<C> SimpleInfluenceForce<C> for DragForce
    where C: Circle + HasLocalEnvironment + NewtonianBody
{
    fn calc_force(&self, ball: &C) -> Force
        where C: Circle + NewtonianBody
    {
        Force::new(self.calc_drag(ball.radius().value(), ball.velocity().x()),
                   self.calc_drag(ball.radius().value(), ball.velocity().y()))
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

impl<C> Influence<C> for UniversalOverlap
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody
{
    fn apply(&self, cell_graph: &mut SortableGraph<C, Bond, AngleGusset>) {
        for ball in cell_graph.unsorted_nodes_mut() {
            ball.environment_mut().add_overlap(self.overlap);
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
        let mut cell_graph = SortableGraph::new();
        let wall_collisions = WallCollisions::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        let ball_handle = cell_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                        Position::new(9.5, 9.5), Velocity::new(1.0, 1.0)));

        wall_collisions.apply(&mut cell_graph);

        let ball = cell_graph.node(ball_handle);
        assert_eq!(1, ball.environment().overlaps().len());
        assert_ne!(0.0, ball.forces().net_force().x());
        assert_ne!(0.0, ball.forces().net_force().y());
    }

    #[test]
    fn pair_collisions_add_overlaps_and_forces() {
        let mut cell_graph = SortableGraph::new();
        let pair_collisions = PairCollisions::new();
        let ball1_handle = cell_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(0.0, 0.0), Velocity::new(1.0, 1.0)));
        let ball2_handle = cell_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(1.4, 1.4), Velocity::new(-1.0, -1.0)));

        pair_collisions.apply(&mut cell_graph);

        let ball1 = cell_graph.node(ball1_handle);
        assert_eq!(1, ball1.environment().overlaps().len());
        assert_ne!(0.0, ball1.forces().net_force().x());
        assert_ne!(0.0, ball1.forces().net_force().y());

        let ball2 = cell_graph.node(ball2_handle);
        assert_eq!(1, ball2.environment().overlaps().len());
        assert_ne!(0.0, ball2.forces().net_force().x());
        assert_ne!(0.0, ball2.forces().net_force().y());
    }

    #[test]
    fn bond_forces_add_forces() {
        let mut cell_graph = SortableGraph::new();
        let bond_forces = BondForces::new();
        let ball1_handle = cell_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(0.0, 0.0), Velocity::new(-1.0, -1.0)));
        let ball2_handle = cell_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(1.5, 1.5), Velocity::new(1.0, 1.0)));
        let bond = Bond::new(cell_graph.node(ball1_handle), cell_graph.node(ball2_handle));
        cell_graph.add_edge(bond);

        bond_forces.apply(&mut cell_graph);

        let ball1 = cell_graph.node(ball1_handle);
        assert_ne!(0.0, ball1.forces().net_force().x());
        assert_ne!(0.0, ball1.forces().net_force().y());

        let ball2 = cell_graph.node(ball2_handle);
        assert_ne!(0.0, ball2.forces().net_force().x());
        assert_ne!(0.0, ball2.forces().net_force().y());
    }

    #[test]
    fn bond_angle_forces_add_forces() {
        let mut cell_graph = SortableGraph::new();

        let ball1_handle = cell_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(0.1, 2.0), Velocity::new(0.0, 0.0)));
        let ball2_handle = cell_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        let ball3_handle = cell_graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                                         Position::new(0.0, -2.0), Velocity::new(0.0, 0.0)));

        let bond = Bond::new(cell_graph.node(ball1_handle), cell_graph.node(ball2_handle));
        let bond1_handle = cell_graph.add_edge(bond);
        let bond = Bond::new(cell_graph.node(ball2_handle), cell_graph.node(ball3_handle));
        let bond2_handle = cell_graph.add_edge(bond);

        let gusset = AngleGusset::new(cell_graph.edge(bond1_handle), cell_graph.edge(bond2_handle), Angle::from_radians(PI));
        cell_graph.add_meta_edge(gusset);

        BondAngleForces::new().apply(&mut cell_graph);

        let ball3 = cell_graph.node(ball3_handle);
        assert!(ball3.forces().net_force().x() < 0.0);
    }

    #[test]
    fn simple_force_influence_adds_force() {
        let mut cell_graph = SortableGraph::new();
        let force = Force::new(2.0, -3.0);
        let influence = SimpleForceInfluence::new(Box::new(ConstantForce::new(force)));
        let ball_handle = cell_graph.add_node(Ball::new(Length::new(1.0), Mass::new(3.0),
                                                        Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));

        influence.apply(&mut cell_graph);

        let ball = cell_graph.node(ball_handle);
        assert_eq!(force, ball.forces().net_force());
    }

    #[test]
    fn weight_adds_force_proportional_to_mass() {
        let weight = WeightForce::new(-2.0);
        let ball = Ball::new(Length::new(1.0), Mass::new(3.0), Position::new(0.0, 0.0), Velocity::new(0.0, 0.0));
        assert_eq!(Force::new(0.0, -6.0), weight.calc_force(&ball));
    }

    #[test]
    fn buoyancy_adds_force_proportional_to_area() {
        let buoyancy = BuoyancyForce::new(-2.0, 2.0);
        let ball = Ball::new(Length::new(2.0 / PI.sqrt()), Mass::new(1.0), Position::new(0.0, 0.0), Velocity::new(0.0, 0.0));
        let force = buoyancy.calc_force(&ball);
        assert_eq!(0.0, force.x());
        assert_eq!(16.0, force.y().round());
    }

    #[test]
    fn drag_adds_force_proportional_to_radius_and_velocity_squared() {
        let drag = DragForce::new(0.5);
        let ball = Ball::new(Length::new(2.0), Mass::new(1.0), Position::new(0.0, 0.0), Velocity::new(2.0, -3.0));
        assert_eq!(Force::new(-4.0, 9.0), drag.calc_force(&ball));
    }
}
