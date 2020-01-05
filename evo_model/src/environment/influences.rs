use crate::biology::cell::Cell;
use crate::environment::local_environment::*;
use crate::physics::bond::*;
use crate::physics::newtonian::*;
use crate::physics::overlap::*;
use crate::physics::quantities::*;
use crate::physics::shapes::Circle;
use crate::physics::sortable_graph::*;
use crate::physics::spring::*;
use crate::physics::util::*;

pub trait Influence {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>);
}

pub struct WallCollisions {
    walls: Walls,
    spring: Box<dyn Spring>,
}

impl WallCollisions {
    pub fn new(min_corner: Position, max_corner: Position, spring: Box<dyn Spring>) -> Self {
        WallCollisions {
            walls: Walls::new(min_corner, max_corner),
            spring,
        }
    }
}

impl Influence for WallCollisions {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>) {
        let overlaps = self.walls.find_overlaps(cell_graph);
        for (handle, overlap) in overlaps {
            let cell = cell_graph.node_mut(handle);
            cell.environment_mut().add_overlap(overlap);
            cell.forces_mut()
                .add_force(overlap.to_force(&(*self.spring)));
        }
    }
}

#[derive(Debug)]
pub struct PairCollisions {
    spring: LinearSpring,
}

impl PairCollisions {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        PairCollisions {
            spring: LinearSpring::new(1.0),
        }
    }
}

impl Influence for PairCollisions {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>) {
        let overlaps = find_pair_overlaps(cell_graph);
        for (handle, overlap) in overlaps {
            let cell = cell_graph.node_mut(handle);
            cell.environment_mut().add_overlap(overlap);
            cell.forces_mut().add_force(overlap.to_force(&self.spring));
        }
    }
}

#[derive(Debug)]
pub struct BondForces {}

impl BondForces {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        BondForces {}
    }
}

impl Influence for BondForces {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>) {
        let strains = calc_bond_strains(cell_graph);
        for (handle, strain) in strains {
            let cell = cell_graph.node_mut(handle);
            cell.forces_mut().add_force(strain.to_force());
        }
    }
}

#[derive(Debug)]
pub struct BondAngleForces {}

impl BondAngleForces {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        BondAngleForces {}
    }
}

impl Influence for BondAngleForces {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>) {
        let forces = calc_bond_angle_forces(cell_graph);
        for (handle, force) in forces {
            let cell = cell_graph.node_mut(handle);
            cell.forces_mut().add_force(force);
        }
    }
}

pub struct SimpleForceInfluence {
    influence_force: Box<dyn SimpleInfluenceForce>,
}

impl SimpleForceInfluence {
    pub fn new(influence_force: Box<dyn SimpleInfluenceForce>) -> Self {
        SimpleForceInfluence { influence_force }
    }
}

impl Influence for SimpleForceInfluence {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>) {
        for cell in cell_graph.unsorted_nodes_mut() {
            let force = self.influence_force.calc_force(cell);
            cell.forces_mut().add_force(force);
        }
    }
}

pub trait SimpleInfluenceForce {
    fn calc_force(&self, cell: &Cell) -> Force;
}

#[derive(Debug)]
pub struct ConstantForce {
    force: Force,
}

impl ConstantForce {
    pub fn new(force: Force) -> Self {
        ConstantForce { force }
    }
}

impl SimpleInfluenceForce for ConstantForce {
    fn calc_force(&self, _ball: &Cell) -> Force {
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
            gravity: Acceleration::new(0.0, gravity),
        }
    }
}

impl SimpleInfluenceForce for WeightForce {
    fn calc_force(&self, cell: &Cell) -> Force {
        cell.mass() * self.gravity
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

impl SimpleInfluenceForce for BuoyancyForce {
    fn calc_force(&self, cell: &Cell) -> Force {
        let displaced_fluid_mass = cell.area() * self.fluid_density;
        -(displaced_fluid_mass * self.gravity)
    }
}

#[derive(Debug)]
pub struct DragForce {
    viscosity: f64,
}

impl DragForce {
    pub fn new(viscosity: f64) -> Self {
        DragForce { viscosity }
    }

    fn calc_drag(&self, radius: f64, velocity: f64) -> f64 {
        -velocity.signum() * self.viscosity * radius * sqr(velocity)
    }
}

impl SimpleInfluenceForce for DragForce {
    fn calc_force(&self, cell: &Cell) -> Force {
        Force::new(
            self.calc_drag(cell.radius().value(), cell.velocity().x()),
            self.calc_drag(cell.radius().value(), cell.velocity().y()),
        )
    }
}

#[derive(Debug)]
pub struct UniversalOverlap {
    overlap: Overlap,
}

impl UniversalOverlap {
    pub fn new(overlap: Overlap) -> Self {
        UniversalOverlap { overlap }
    }
}

impl Influence for UniversalOverlap {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>) {
        for cell in cell_graph.unsorted_nodes_mut() {
            cell.environment_mut().add_overlap(self.overlap);
        }
    }
}

#[derive(Debug)]
pub struct Sunlight {
    slope: f64,
    intercept: f64,
}

impl Sunlight {
    pub fn new(min_y: f64, max_y: f64, min_intensity: f64, max_intensity: f64) -> Self {
        let slope = (max_intensity - min_intensity) / (max_y - min_y);
        Sunlight {
            slope,
            intercept: max_intensity - slope * max_y,
        }
    }

    fn calc_light_intensity(&self, y: f64) -> f64 {
        (self.slope * y + self.intercept).max(0.0)
    }
}

impl Influence for Sunlight {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>) {
        for cell in cell_graph.unsorted_nodes_mut() {
            let y = cell.center().y();
            cell.environment_mut()
                .add_light_intensity(self.calc_light_intensity(y));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::layers::*;
    use std::f64::consts::PI;

    #[test]
    fn wall_collisions_add_overlap_and_force() {
        let mut cell_graph = SortableGraph::new();
        let wall_collisions = WallCollisions::new(
            Position::new(-10.0, -10.0),
            Position::new(10.0, 10.0),
            Box::new(LinearSpring::new(1.0)),
        );
        let ball_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(9.5, 9.5),
            Velocity::new(1.0, 1.0),
        ));

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
        let ball1_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(0.0, 0.0),
            Velocity::new(1.0, 1.0),
        ));
        let ball2_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(1.4, 1.4),
            Velocity::new(-1.0, -1.0),
        ));

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
        let ball1_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(0.0, 0.0),
            Velocity::new(-1.0, -1.0),
        ));
        let ball2_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(1.5, 1.5),
            Velocity::new(1.0, 1.0),
        ));
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

        let ball1_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(0.1, 2.0),
            Velocity::ZERO,
        ));
        let ball2_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(0.0, 0.0),
            Velocity::ZERO,
        ));
        let ball3_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(0.0, -2.0),
            Velocity::ZERO,
        ));

        let bond = Bond::new(cell_graph.node(ball1_handle), cell_graph.node(ball2_handle));
        let bond1_handle = cell_graph.add_edge(bond);
        let bond = Bond::new(cell_graph.node(ball2_handle), cell_graph.node(ball3_handle));
        let bond2_handle = cell_graph.add_edge(bond);

        let gusset = AngleGusset::new(
            cell_graph.edge(bond1_handle),
            cell_graph.edge(bond2_handle),
            Angle::from_radians(PI),
        );
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
        let ball_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(3.0),
            Position::new(0.0, 0.0),
            Velocity::ZERO,
        ));

        influence.apply(&mut cell_graph);

        let ball = cell_graph.node(ball_handle);
        assert_eq!(force, ball.forces().net_force());
    }

    #[test]
    fn weight_adds_force_proportional_to_mass() {
        let weight = WeightForce::new(-2.0);
        let ball = Cell::ball(
            Length::new(1.0),
            Mass::new(3.0),
            Position::new(0.0, 0.0),
            Velocity::ZERO,
        );
        assert_eq!(Force::new(0.0, -6.0), weight.calc_force(&ball));
    }

    #[test]
    fn buoyancy_adds_force_proportional_to_area() {
        let buoyancy = BuoyancyForce::new(-2.0, 2.0);
        let ball = Cell::ball(
            Length::new(2.0 / PI.sqrt()),
            Mass::new(1.0),
            Position::new(0.0, 0.0),
            Velocity::ZERO,
        );
        let force = buoyancy.calc_force(&ball);
        assert_eq!(0.0, force.x());
        assert_eq!(16.0, force.y().round());
    }

    #[test]
    fn drag_adds_force_proportional_to_radius_and_velocity_squared() {
        let drag = DragForce::new(0.5);
        let ball = Cell::ball(
            Length::new(2.0),
            Mass::new(1.0),
            Position::new(0.0, 0.0),
            Velocity::new(2.0, -3.0),
        );
        assert_eq!(Force::new(-4.0, 9.0), drag.calc_force(&ball));
    }

    #[test]
    fn sunlight_adds_light() {
        let sunlight = Sunlight::new(-10.0, 10.0, 10.0, 20.0);
        let mut cell_graph = SortableGraph::new();
        let cell_handle = cell_graph.add_node(Cell::new(
            Position::new(0.0, 0.0),
            Velocity::ZERO,
            vec![Box::new(simple_cell_layer(
                Area::new(PI),
                Density::new(1.0),
            ))],
        ));

        sunlight.apply(&mut cell_graph);

        let cell = cell_graph.node(cell_handle);
        assert_eq!(15.0, cell.environment().light_intensity());
    }

    #[test]
    fn sunlight_never_negative() {
        let sunlight = Sunlight::new(-10.0, 0.0, 0.0, 10.0);
        let mut cell_graph = SortableGraph::new();
        let cell_handle = cell_graph.add_node(Cell::new(
            Position::new(0.0, -11.0),
            Velocity::ZERO,
            vec![Box::new(simple_cell_layer(
                Area::new(1.0),
                Density::new(1.0),
            ))],
        ));

        sunlight.apply(&mut cell_graph);

        let cell = cell_graph.node(cell_handle);
        assert_eq!(0.0, cell.environment().light_intensity());
    }

    fn simple_cell_layer(area: Area, density: Density) -> CellLayer {
        CellLayer::new(
            area,
            density,
            Color::Green,
            Box::new(NullCellLayerSpecialty::new()),
        )
    }
}
