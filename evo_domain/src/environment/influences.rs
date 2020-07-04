use crate::biology::cell::Cell;
use crate::environment::local_environment::*;
use crate::physics::bond::*;
use crate::physics::newtonian::*;
use crate::physics::overlap::*;
use crate::physics::quantities::*;
use crate::physics::shapes::Circle;
use crate::physics::sortable_graph::*;
use crate::physics::util::*;
use log::trace;

pub trait Influence {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>);
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

    fn add_overlap_and_force(&self, cell: &mut Cell, overlap: Overlap) {
        cell.environment_mut().add_overlap(overlap);
        let force = Self::collision_force(cell.mass(), cell.velocity(), -overlap.incursion());
        trace!("Cell {} Wall {:?}", cell.node_handle(), force);
        cell.forces_mut().add_force(force);
    }

    fn collision_force(mass: Mass, velocity: Velocity, overlap: Displacement) -> Force {
        Force::new(
            Self::x_or_y_collision_force(mass, velocity.x(), overlap.x()),
            Self::x_or_y_collision_force(mass, velocity.y(), overlap.y()),
        )
    }

    fn x_or_y_collision_force(mass: Mass, velocity: f64, overlap: f64) -> f64 {
        let v = if overlap > 0.0 {
            velocity.max(overlap)
        } else if overlap < 0.0 {
            velocity.min(overlap)
        } else {
            -velocity
        };
        -mass.value() * (velocity + v)
    }
}

impl Influence for WallCollisions {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>) {
        let overlaps = self.walls.find_overlaps(cell_graph);
        for (handle, overlap) in overlaps {
            self.add_overlap_and_force(cell_graph.node_mut(handle), overlap);
        }
    }
}

#[derive(Debug)]
pub struct PairCollisions {}

impl PairCollisions {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        PairCollisions {}
    }

    fn add_overlap_and_force(cell: &mut Cell, overlap: Overlap, force: Force) {
        cell.environment_mut().add_overlap(overlap);
        trace!("Cell {} Pair {:?}", cell.node_handle(), force);
        cell.forces_mut().add_force(force);
    }

    fn cell1_collision_force(cell1: &Cell, overlap1: Overlap, cell2: &Cell) -> Force {
        if overlap1.incursion() == Displacement::ZERO {
            return Force::ZERO;
        }

        let collision_force = Self::body1_elastic_collision_force(
            cell1.mass(),
            cell2.mass(),
            cell1.velocity() - cell2.velocity(),
            cell1.position() - cell2.position(),
        );
        let overlap_force = Self::body1_overlap_force(cell1.mass(), cell2.mass(), overlap1);

        if overlap_force.value().magnitude() > collision_force.value().magnitude() {
            overlap_force
        } else {
            collision_force
        }
    }

    // Derived from Wikipedia's "Elastic collision" page, the "angle-free representation"
    // at the end of the two-dimensional collision section. This is the force needed to
    // produce Wikipedia's post-elastic-collision velocity.
    fn body1_elastic_collision_force(
        mass1: Mass,
        mass2: Mass,
        relative_velocity1: DeltaV,
        relative_position1: Displacement,
    ) -> Force {
        Force::from(
            -2.0 * (mass1.value() * mass2.value() / (mass1 + mass2).value())
                * relative_velocity1
                    .value()
                    .project_onto(relative_position1.value()),
        )
    }

    fn body1_overlap_force(mass1: Mass, mass2: Mass, overlap1: Overlap) -> Force {
        Force::from(
            (mass1.value() * mass2.value() / (mass1 + mass2).value())
                * overlap1.incursion().value(),
        )
    }
}

impl Influence for PairCollisions {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>) {
        let overlaps = find_pair_overlaps(cell_graph);
        for ((handle1, overlap1), (handle2, overlap2)) in overlaps {
            let force1 = Self::cell1_collision_force(
                cell_graph.node(handle1),
                overlap1,
                cell_graph.node(handle2),
            );
            Self::add_overlap_and_force(cell_graph.node_mut(handle1), overlap1, force1);
            Self::add_overlap_and_force(cell_graph.node_mut(handle2), overlap2, -force1);
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

    fn add_force(cell: &mut Cell, force: Force) {
        trace!("Cell {} Bond {:?}", cell.node_handle(), force);
        cell.forces_mut().set_net_force_if_stronger(force);
    }

    fn cell1_bond_force(cell1: &Cell, strain1: BondStrain, cell2: &Cell) -> Force {
        let velocity_force = Self::body1_clear_velocity_force(
            cell1.mass(),
            cell2.mass(),
            cell1.velocity(),
            cell2.velocity(),
            cell1.position() - cell2.position(),
        );
        let strain_force = Self::body1_clear_strain_force(cell1.mass(), cell2.mass(), strain1);
        if cell1.is_selected() {
            println!(
                "Bond {}-{} velocity force: {}, strain force: {}",
                cell1.node_handle(),
                cell2.node_handle(),
                velocity_force,
                strain_force
            );
        }
        velocity_force + strain_force
    }

    fn body1_clear_velocity_force(
        mass1: Mass,
        mass2: Mass,
        velocity1: Velocity,
        velocity2: Velocity,
        relative_position1: Displacement,
    ) -> Force {
        let velocity_cm = (mass1 * velocity1 + mass2 * velocity2) / (mass1 + mass2);
        -Force::from(
            mass1.value()
                * (velocity1 - velocity_cm)
                    .value()
                    .project_onto(relative_position1.value()),
        )
    }

    fn body1_clear_strain_force(mass1: Mass, mass2: Mass, strain1: BondStrain) -> Force {
        Force::from(
            (mass1.value() * mass2.value() / (mass1 + mass2).value()) * strain1.strain().value(),
        )
    }

    // TODO lose this after updating tests
    pub fn bond_force(
        mass1: Mass,
        velocity1: Velocity,
        strain1: Displacement,
        mass2: Mass,
        velocity2: Velocity,
    ) -> Force {
        let mass_factor = (mass1.value() * mass2.value()) / (mass1.value() + mass2.value());
        let relative_velocity1 = velocity1 - velocity2;
        Force::new(
            Self::x_or_y_bond_force(mass_factor, relative_velocity1.x(), strain1.x()),
            Self::x_or_y_bond_force(mass_factor, relative_velocity1.y(), strain1.y()),
        )
    }

    fn x_or_y_bond_force(mass_factor: f64, relative_velocity1: f64, strain1: f64) -> f64 {
        let v = if strain1 != 0.0 {
            strain1
        } else {
            -relative_velocity1
        };
        -mass_factor * (relative_velocity1 + v)
    }
}

impl Influence for BondForces {
    fn apply(&self, cell_graph: &mut SortableGraph<Cell, Bond, AngleGusset>) {
        let strains = calc_bond_strains(cell_graph);
        for ((handle1, strain1), (handle2, _strain2)) in strains {
            let force1 =
                Self::cell1_bond_force(cell_graph.node(handle1), strain1, cell_graph.node(handle2));
            Self::add_force(cell_graph.node_mut(handle1), force1);
            Self::add_force(cell_graph.node_mut(handle2), -force1);
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
            trace!("Cell {} BondAngle {:?}", cell.node_handle(), force);
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
        for cell in cell_graph.nodes_mut() {
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
        let force = cell.mass() * self.gravity;
        trace!("Cell {} Weight {:?}", cell.node_handle(), force);
        force
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
        let force = -(displaced_fluid_mass * self.gravity);
        trace!("Cell {} Buoyancy {:?}", cell.node_handle(), force);
        force
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

    fn calc_drag(&self, mass: Mass, radius: Length, velocity: f64) -> f64 {
        -velocity.signum()
            * self
                .instantaneous_abs_drag(radius, velocity)
                .min(Self::abs_drag_that_will_stop_the_cell(mass, velocity))
    }

    fn instantaneous_abs_drag(&self, radius: Length, velocity: f64) -> f64 {
        self.viscosity * radius.value() * sqr(velocity)
    }

    fn abs_drag_that_will_stop_the_cell(mass: Mass, velocity: f64) -> f64 {
        mass.value() * velocity.abs()
    }
}

impl SimpleInfluenceForce for DragForce {
    fn calc_force(&self, cell: &Cell) -> Force {
        let force = Force::new(
            self.calc_drag(cell.mass(), cell.radius(), cell.velocity().x()),
            self.calc_drag(cell.mass(), cell.radius(), cell.velocity().y()),
        );
        trace!("Cell {} Drag {:?}", cell.node_handle(), force);
        force
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
        for cell in cell_graph.nodes_mut() {
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
        for cell in cell_graph.nodes_mut() {
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
    fn wall_collisions_add_overlap_and_force_old() {
        let mut cell_graph = SortableGraph::new();
        let wall_collisions =
            WallCollisions::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        let ball_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(9.5, 9.5),
            Velocity::new(1.0, 1.0),
        ));

        wall_collisions.apply(&mut cell_graph);

        let ball = cell_graph.node(ball_handle);
        assert_eq!(ball.environment().overlaps().len(), 1);
        assert_ne!(ball.forces().net_force().x(), 0.0);
        assert_ne!(ball.forces().net_force().y(), 0.0);
    }

    #[test]
    fn wall_collisions_add_overlap_and_force() {
        let mut cell_graph = SortableGraph::new();
        let wall_collisions =
            WallCollisions::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        let ball_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(9.5, 9.5),
            Velocity::new(1.0, 1.0),
        ));

        wall_collisions.apply(&mut cell_graph);

        let ball = cell_graph.node(ball_handle);
        assert_eq!(ball.environment().overlaps().len(), 1);
        assert_ne!(ball.forces().net_force().x(), 0.0);
        assert_ne!(ball.forces().net_force().y(), 0.0);
    }

    #[test]
    fn no_walls_collision_force() {
        assert_eq!(
            WallCollisions::collision_force(
                Mass::new(2.0),
                Velocity::new(3.0, 2.0),
                Displacement::new(0.0, 0.0)
            ),
            Force::new(0.0, 0.0)
        );
    }

    #[test]
    fn top_right_walls_fast_collision_force() {
        assert_eq!(
            WallCollisions::collision_force(
                Mass::new(2.0),
                Velocity::new(3.0, 4.0),
                Displacement::new(2.0, 3.0)
            ),
            Force::new(-12.0, -16.0)
        );
    }

    #[test]
    fn top_right_walls_slow_collision_force() {
        assert_eq!(
            WallCollisions::collision_force(
                Mass::new(2.0),
                Velocity::new(1.0, 0.5),
                Displacement::new(2.0, 1.5)
            ),
            Force::new(-6.0, -4.0)
        );
    }

    #[test]
    fn bottom_left_walls_fast_collision_force() {
        assert_eq!(
            WallCollisions::collision_force(
                Mass::new(2.0),
                Velocity::new(-3.0, -4.0),
                Displacement::new(-2.0, -3.0)
            ),
            Force::new(12.0, 16.0)
        );
    }

    #[test]
    fn bottom_left_walls_slow_collision_force() {
        assert_eq!(
            WallCollisions::collision_force(
                Mass::new(2.0),
                Velocity::new(-1.0, -0.5),
                Displacement::new(-2.0, -1.5)
            ),
            Force::new(6.0, 4.0)
        );
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
        assert_eq!(ball1.environment().overlaps().len(), 1);
        assert_ne!(ball1.forces().net_force().x(), 0.0);
        assert_ne!(ball1.forces().net_force().y(), 0.0);

        let ball2 = cell_graph.node(ball2_handle);
        assert_eq!(ball2.environment().overlaps().len(), 1);
        assert_ne!(ball2.forces().net_force().x(), 0.0);
        assert_ne!(ball2.forces().net_force().y(), 0.0);
    }

    #[test]
    fn pair_not_in_collision_adds_no_force() {
        assert_eq!(
            PairCollisions::cell1_collision_force(
                &Cell::ball(
                    Length::new(2.0),
                    Mass::new(2.0),
                    Position::new(-3.0, 4.0),
                    Velocity::new(3.0, -4.0)
                ),
                Overlap::new(Displacement::new(0.0, 0.0), 2.0),
                &Cell::ball(
                    Length::new(3.0),
                    Mass::new(6.0),
                    Position::new(0.0, 0.0),
                    Velocity::new(-5.0, 6.0),
                ),
            ),
            Force::new(0.0, 0.0)
        );
    }

    #[test]
    fn pair_collision_force_reflects_incoming_velocity() {
        let cell1 = Cell::ball(
            Length::new(2.0),
            Mass::new(2.0),
            Position::new(-1.5, 2.0),
            Velocity::new(3.0, -4.0),
        );
        let cell2 = Cell::ball(
            Length::new(3.0),
            Mass::new(6.0),
            Position::new(0.0, 0.0),
            Velocity::new(-5.0, 6.0),
        );

        let force1 = PairCollisions::cell1_collision_force(
            &cell1,
            Overlap::new(Displacement::new(-1.5, 2.0), 2.0),
            &cell2,
        );
        assert_eq!(force1, Force::new(-23.04, 30.72));
    }

    #[test]
    fn pair_collision_force_undoes_overlap() {
        let cell1 = Cell::ball(
            Length::new(8.0),
            Mass::new(2.0),
            Position::new(-9.0, 12.0),
            Velocity::new(0.0, 0.0),
        );
        let cell2 = Cell::ball(
            Length::new(12.0),
            Mass::new(6.0),
            Position::new(0.0, 0.0),
            Velocity::new(0.0, 0.0),
        );

        let force1 = PairCollisions::cell1_collision_force(
            &cell1,
            Overlap::new(Displacement::new(-3.0, 4.0), 2.0),
            &cell2,
        );
        assert_eq!(force1, Force::new(-4.5, 6.0));

        let velocity1_after = cell1.velocity() + (force1 / cell1.mass()) * Duration::ONE;
        let position1_after = cell1.position() + velocity1_after * Duration::ONE;
        let velocity2_after = cell2.velocity() + (-force1 / cell2.mass()) * Duration::ONE;
        let position2_after = cell2.position() + velocity2_after * Duration::ONE;
        let relative_position1_after = position1_after - position2_after;
        assert_eq!(
            relative_position1_after.length(),
            cell1.radius() + cell2.radius()
        );
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
        cell_graph.add_edge(bond, 1, 0);

        bond_forces.apply(&mut cell_graph);

        let ball1 = cell_graph.node(ball1_handle);
        assert_ne!(ball1.forces().net_force().x(), 0.0);
        assert_ne!(ball1.forces().net_force().y(), 0.0);

        let ball2 = cell_graph.node(ball2_handle);
        assert_ne!(ball2.forces().net_force().x(), 0.0);
        assert_ne!(ball2.forces().net_force().y(), 0.0);
    }

    #[test]
    fn bond_with_no_strain_adds_no_force() {
        assert_eq!(
            BondForces::bond_force(
                Mass::new(2.0),
                Velocity::new(3.0, -4.0),
                Displacement::new(0.0, 0.0),
                Mass::new(6.0),
                Velocity::new(-5.0, 6.0),
            ),
            Force::new(0.0, 0.0)
        );
    }

    #[test]
    fn bond_force_undoes_compression_strain() {
        let mass1 = Mass::new(2.0);
        let velocity1 = Velocity::new(1.5, -2.5);
        let strain1 = Displacement::new(3.0, -5.0);
        let mass2 = Mass::new(6.0);
        let velocity2 = Velocity::new(-0.5, 1.5);

        let force1 = BondForces::bond_force(mass1, velocity1, strain1, mass2, velocity2);
        assert_eq!(force1, Force::new(-7.5, 13.5));

        let velocity1_after = velocity1 + (force1 / mass1) * Duration::ONE;
        let velocity2_after = velocity2 + (-force1 / mass2) * Duration::ONE;
        let relative_velocity1_after = velocity1_after - velocity2_after;
        assert_eq!(relative_velocity1_after * Duration::ONE, -strain1);
    }

    #[test]
    fn bond_force_undoes_extension_strain() {
        let mass1 = Mass::new(2.0);
        let velocity1 = Velocity::new(-1.5, 2.5);
        let strain1 = Displacement::new(-3.0, 5.0);
        let mass2 = Mass::new(6.0);
        let velocity2 = Velocity::new(0.5, -1.5);

        let force1 = BondForces::bond_force(mass1, velocity1, strain1, mass2, velocity2);
        assert_eq!(force1, Force::new(7.5, -13.5));

        let velocity1_after = velocity1 + (force1 / mass1) * Duration::ONE;
        let velocity2_after = velocity2 + (-force1 / mass2) * Duration::ONE;
        let relative_velocity1_after = velocity1_after - velocity2_after;
        assert_eq!(relative_velocity1_after * Duration::ONE, -strain1);
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
        let bond1_handle = cell_graph.add_edge(bond, 1, 0);
        let bond = Bond::new(cell_graph.node(ball2_handle), cell_graph.node(ball3_handle));
        let bond2_handle = cell_graph.add_edge(bond, 1, 0);

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
        assert_eq!(ball.forces().net_force(), force);
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
        assert_eq!(weight.calc_force(&ball), Force::new(0.0, -6.0));
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
        assert_eq!(force.x(), 0.0);
        assert_eq!(force.y().round(), 16.0);
    }

    #[test]
    fn drag_adds_force_proportional_to_radius_and_velocity_squared() {
        let drag = DragForce::new(0.5);
        let ball = Cell::ball(
            Length::new(2.0),
            Mass::new(10.0),
            Position::new(0.0, 0.0),
            Velocity::new(2.0, -3.0),
        );
        assert_eq!(drag.calc_force(&ball), Force::new(-4.0, 9.0));
    }

    #[test]
    fn drag_force_is_limited_to_force_that_will_stop_cell() {
        let drag = DragForce::new(0.5);
        let ball = Cell::ball(
            Length::new(10.0),
            Mass::new(0.01),
            Position::ORIGIN,
            Velocity::new(10.0, -10.0),
        );
        assert_eq!(drag.calc_force(&ball), Force::new(-0.1, 0.1));
    }

    #[test]
    fn sunlight_adds_light() {
        let sunlight = Sunlight::new(-10.0, 10.0, 10.0, 20.0);
        let mut cell_graph = SortableGraph::new();
        let cell_handle = cell_graph.add_node(simple_layered_cell(vec![simple_cell_layer(
            Area::new(PI),
            Density::new(1.0),
        )]));

        sunlight.apply(&mut cell_graph);

        let cell = cell_graph.node(cell_handle);
        assert_eq!(cell.environment().light_intensity(), 15.0);
    }

    #[test]
    fn sunlight_never_negative() {
        let sunlight = Sunlight::new(-10.0, 0.0, 0.0, 10.0);
        let mut cell_graph = SortableGraph::new();
        let cell_handle = cell_graph.add_node(
            simple_layered_cell(vec![simple_cell_layer(Area::new(1.0), Density::new(1.0))])
                .with_initial_position(Position::new(0.0, -11.0)),
        );

        sunlight.apply(&mut cell_graph);

        let cell = cell_graph.node(cell_handle);
        assert_eq!(cell.environment().light_intensity(), 0.0);
    }

    fn simple_layered_cell(layers: Vec<CellLayer>) -> Cell {
        Cell::new(Position::ORIGIN, Velocity::ZERO, layers)
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
