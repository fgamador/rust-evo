use crate::biology::cell::Cell;
use crate::environment::local_environment::*;
use crate::physics::bond::*;
use crate::physics::newtonian::*;
use crate::physics::node_graph::*;
use crate::physics::overlap::*;
use crate::physics::quantities::*;
use crate::physics::shapes::Circle;
use crate::physics::util::*;

pub trait CrossCellInfluence {
    fn apply_to(
        &self,
        cell_graph: &mut NodeGraph<Cell, Bond, AngleGusset>,
        cell_handles: &mut SortableHandles,
    );
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
        cell.net_force_mut()
            .add_dominant_force(force, "wall collision");
    }

    fn collision_force(mass: Mass, velocity: Velocity, overlap: Displacement) -> Force {
        Force::new(
            Self::x_or_y_collision_force(mass, velocity.x(), overlap.x()),
            Self::x_or_y_collision_force(mass, velocity.y(), overlap.y()),
        )
    }

    fn x_or_y_collision_force(mass: Mass, velocity: Value1D, overlap: Value1D) -> Value1D {
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

impl CrossCellInfluence for WallCollisions {
    fn apply_to(
        &self,
        cell_graph: &mut NodeGraph<Cell, Bond, AngleGusset>,
        _cell_handles: &mut SortableHandles,
    ) {
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

    fn add_overlap(cell: &mut Cell, overlap: Overlap) {
        cell.environment_mut().add_overlap(overlap);
    }

    fn add_forces(cell1: &mut Cell, cell2: &mut Cell, overlap1: Overlap) {
        let mass_factor = Self::mass_factor(cell1.mass(), cell2.mass());
        let relative_velocity1 = cell1.velocity() - cell2.velocity();
        let relative_position1 = cell1.position() - cell2.position();
        let relative_position1_unit = relative_position1.value().to_unit_vector();
        let closing_speed = -relative_velocity1.value().dot(relative_position1_unit);

        let cell1_collision_force = if closing_speed > 0.0 {
            Self::body1_elastic_collision_force(
                mass_factor,
                relative_velocity1,
                relative_position1_unit,
            )
        } else {
            Force::ZERO
        };

        let cell1_overlap_force =
            Self::body1_undo_overlap_force(mass_factor, overlap1, closing_speed);

        Self::update_net_force(cell1, cell1_collision_force, cell1_overlap_force);
        Self::update_net_force(cell2, -cell1_collision_force, -cell1_overlap_force);
    }

    fn mass_factor(mass1: Mass, mass2: Mass) -> Value1D {
        mass1.value() * mass2.value() / (mass1 + mass2).value()
    }

    // Derived from Wikipedia's "Elastic collision" page, the "angle-free representation"
    // at the end of the two-dimensional collision section. This is the force needed to
    // produce Wikipedia's post-elastic-collision velocity.
    fn body1_elastic_collision_force(
        mass_factor: Value1D,
        relative_velocity1: DeltaV,
        relative_position1_unit: Value2D,
    ) -> Force {
        Force::from(
            -2.0 * mass_factor
                * relative_velocity1.value().dot(relative_position1_unit)
                * relative_position1_unit,
        )
    }

    fn body1_undo_overlap_force(
        mass_factor: Value1D,
        overlap1: Overlap,
        closing_speed: Value1D,
    ) -> Force {
        let incursion = overlap1.incursion().value();
        if incursion.length() + closing_speed > 0.0 {
            let needed_velocity = incursion + closing_speed * incursion.to_unit_vector();
            Force::from(mass_factor * needed_velocity)
        } else {
            Force::ZERO
        }
    }

    fn update_net_force(cell: &mut Cell, collision_force: Force, overlap_force: Force) {
        let net_force = cell.net_force_mut();
        net_force.add_dominant_force(collision_force, "pair collision velocity");
        net_force.add_dominant_force(overlap_force, "pair collision overlap");
    }
}

impl CrossCellInfluence for PairCollisions {
    fn apply_to(
        &self,
        cell_graph: &mut NodeGraph<Cell, Bond, AngleGusset>,
        cell_handles: &mut SortableHandles,
    ) {
        let overlaps = find_pair_overlaps(cell_graph, cell_handles);
        for ((handle1, overlap1), (handle2, overlap2)) in overlaps {
            Self::add_overlap(cell_graph.node_mut(handle1), overlap1);
            Self::add_overlap(cell_graph.node_mut(handle2), overlap2);

            if overlap1.incursion() == Displacement::ZERO {
                continue;
            }

            cell_graph.with_nodes(handle1, handle2, |cell1, cell2| {
                Self::add_forces(cell1, cell2, overlap1);
            });
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

    fn add_forces(cell1: &mut Cell, cell2: &mut Cell, strain1: BondStrain) {
        let (cell1_velocity_force, cell1_strain_force) = Self::cell1_forces(cell1, cell2, strain1);
        Self::update_net_force(cell1, cell1_velocity_force, cell1_strain_force);
        Self::update_net_force(cell2, -cell1_velocity_force, -cell1_strain_force);
    }

    fn cell1_forces(cell1: &Cell, cell2: &Cell, strain1: BondStrain) -> (Force, Force) {
        let velocity_force = Self::body1_stop_velocity_force(
            cell1.mass(),
            cell2.mass(),
            cell1.velocity(),
            cell2.velocity(),
            cell1.position() - cell2.position(),
        );
        let strain_force = Self::body1_undo_strain_force(cell1.mass(), cell2.mass(), strain1);
        (velocity_force, strain_force)
    }

    fn body1_stop_velocity_force(
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

    fn body1_undo_strain_force(mass1: Mass, mass2: Mass, strain1: BondStrain) -> Force {
        Force::from(
            (mass1.value() * mass2.value() / (mass1 + mass2).value()) * strain1.strain().value(),
        )
    }

    fn update_net_force(cell: &mut Cell, velocity_force: Force, strain_force: Force) {
        let net_force = cell.net_force_mut();
        net_force.add_dominant_force(velocity_force, "pair bond velocity");
        net_force.add_dominant_force(strain_force, "pair bond strain");
    }
}

impl CrossCellInfluence for BondForces {
    fn apply_to(
        &self,
        cell_graph: &mut NodeGraph<Cell, Bond, AngleGusset>,
        _cell_handles: &mut SortableHandles,
    ) {
        let strains = calc_bond_strains(cell_graph);
        for ((handle1, strain1), (handle2, _strain2)) in strains {
            cell_graph.with_nodes(handle1, handle2, |cell1, cell2| {
                Self::add_forces(cell1, cell2, strain1);
            });
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

impl CrossCellInfluence for BondAngleForces {
    fn apply_to(
        &self,
        cell_graph: &mut NodeGraph<Cell, Bond, AngleGusset>,
        _cell_handles: &mut SortableHandles,
    ) {
        let forces = calc_bond_angle_forces(cell_graph);
        for (handle, force) in forces {
            let cell = cell_graph.node_mut(handle);
            cell.net_force_mut()
                .add_non_dominant_force(force, "bond angle");
        }
    }
}

pub trait PerCellInfluence: Send + Sync {
    fn apply_to(&self, cell: &mut Cell);
}

pub struct SimpleForceInfluence {
    influence_force: Box<dyn SimpleInfluenceForce>,
}

impl SimpleForceInfluence {
    pub fn new(influence_force: Box<dyn SimpleInfluenceForce>) -> Self {
        SimpleForceInfluence { influence_force }
    }
}

impl PerCellInfluence for SimpleForceInfluence {
    fn apply_to(&self, cell: &mut Cell) {
        let force = self.influence_force.calc_force(cell);
        cell.net_force_mut()
            .add_non_dominant_force(force, self.influence_force.label());
    }
}

pub trait SimpleInfluenceForce: Send + Sync {
    fn calc_force(&self, cell: &Cell) -> Force;

    fn label(&self) -> &'static str;
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

    fn label(&self) -> &'static str {
        "constant"
    }
}

#[derive(Debug)]
pub struct WeightForce {
    gravity: Acceleration,
}

impl WeightForce {
    pub fn new(gravity: Value1D) -> Self {
        WeightForce {
            gravity: Acceleration::new(0.0, gravity),
        }
    }
}

impl SimpleInfluenceForce for WeightForce {
    fn calc_force(&self, cell: &Cell) -> Force {
        cell.mass() * self.gravity
    }

    fn label(&self) -> &'static str {
        "weight"
    }
}

#[derive(Debug)]
pub struct BuoyancyForce {
    gravity: Acceleration,
    fluid_density: Density,
}

impl BuoyancyForce {
    pub fn new(gravity: Value1D, fluid_density: Value1D) -> Self {
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

    fn label(&self) -> &'static str {
        "buoyancy"
    }
}

#[derive(Debug)]
pub struct DragForce {
    viscosity: Value1D,
}

impl DragForce {
    pub fn new(viscosity: Value1D) -> Self {
        DragForce { viscosity }
    }

    fn calc_drag(&self, mass: Mass, radius: Length, velocity: Value1D) -> Value1D {
        -velocity.signum()
            * self
                .instantaneous_abs_drag(radius, velocity)
                .min(Self::abs_drag_that_will_stop_the_cell(mass, velocity))
    }

    fn instantaneous_abs_drag(&self, radius: Length, velocity: Value1D) -> Value1D {
        self.viscosity * radius.value() * sqr(velocity)
    }

    fn abs_drag_that_will_stop_the_cell(mass: Mass, velocity: Value1D) -> Value1D {
        mass.value() * velocity.abs()
    }
}

impl SimpleInfluenceForce for DragForce {
    fn calc_force(&self, cell: &Cell) -> Force {
        Force::new(
            self.calc_drag(cell.mass(), cell.radius(), cell.velocity().x()),
            self.calc_drag(cell.mass(), cell.radius(), cell.velocity().y()),
        )
    }

    fn label(&self) -> &'static str {
        "drag"
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

impl PerCellInfluence for UniversalOverlap {
    fn apply_to(&self, cell: &mut Cell) {
        cell.environment_mut().add_overlap(self.overlap);
    }
}

#[derive(Debug)]
pub struct Sunlight {
    slope: Value1D,
    intercept: Value1D,
}

impl Sunlight {
    pub fn new(
        min_y: Value1D,
        max_y: Value1D,
        min_intensity: Value1D,
        max_intensity: Value1D,
    ) -> Self {
        let slope = (max_intensity - min_intensity) / (max_y - min_y);
        Sunlight {
            slope,
            intercept: max_intensity - slope * max_y,
        }
    }

    fn calc_light_intensity(&self, y: Value1D) -> Value1D {
        (self.slope * y + self.intercept).max(0.0)
    }
}

impl PerCellInfluence for Sunlight {
    fn apply_to(&self, cell: &mut Cell) {
        let y = cell.center().y();
        cell.environment_mut()
            .add_light_intensity(self.calc_light_intensity(y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::layers::*;
    use std::f64::consts::PI;

    #[test]
    fn wall_collisions_add_overlap_and_force() {
        let mut cell_graph = NodeGraph::new();
        let wall_collisions =
            WallCollisions::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        let ball_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(9.5, 9.5),
            Velocity::new(1.0, 1.0),
        ));

        wall_collisions.apply_to(&mut cell_graph, &mut SortableHandles::new());

        let ball = cell_graph.node(ball_handle);
        assert_eq!(ball.environment().overlaps().len(), 1);
        assert_ne!(ball.net_force().net_force().x(), 0.0);
        assert_ne!(ball.net_force().net_force().y(), 0.0);
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
        let mut cell_graph = NodeGraph::new();
        let mut cell_handles = SortableHandles::new();
        let pair_collisions = PairCollisions::new();
        let cell1_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(0.0, 0.0),
            Velocity::new(1.0, 1.0),
        ));
        let cell2_handle = cell_graph.add_node(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(1.4, 1.4),
            Velocity::new(-1.0, -1.0),
        ));
        cell_handles.add_handle(SortableHandle::GraphNode(cell1_handle));
        cell_handles.add_handle(SortableHandle::GraphNode(cell2_handle));

        pair_collisions.apply_to(&mut cell_graph, &mut cell_handles);

        let cell1 = cell_graph.node(cell1_handle);
        assert_eq!(cell1.environment().overlaps().len(), 1);
        assert_ne!(cell1.net_force().net_force().x(), 0.0);
        assert_ne!(cell1.net_force().net_force().y(), 0.0);

        let cell2 = cell_graph.node(cell2_handle);
        assert_eq!(cell2.environment().overlaps().len(), 1);
        assert_ne!(cell2.net_force().net_force().x(), 0.0);
        assert_ne!(cell2.net_force().net_force().y(), 0.0);
    }

    // TODO fn pair_not_in_collision_adds_no_force()

    #[test]
    fn pair_collision_force_transfers_momentum_of_matched_cells() {
        let mut cell1 = Cell::ball(
            Length::new(3.5),
            Mass::new(2.0),
            Position::new(-3.0, 4.0),
            Velocity::new(1.5, -2.0),
        );
        let mut cell2 = Cell::ball(
            Length::new(2.0),
            Mass::new(2.0),
            Position::ORIGIN,
            Velocity::ZERO,
        );
        let overlap1 = calc_overlap(&cell1, &cell2).unwrap();

        PairCollisions::add_forces(&mut cell1, &mut cell2, overlap1);

        cell1.tick();
        assert_eq!(cell1.velocity(), Velocity::ZERO);
        cell2.tick();
        assert_eq!(cell2.velocity(), Velocity::new(1.5, -2.0));
    }

    #[test]
    fn pair_collision_force_reflects_velocity_angle_for_collision_at_tangent() {
        let initial_velocity1 = Velocity::new(3.0, -4.0);
        let mut cell1 = Cell::ball(
            Length::new(5.0),
            Mass::new(1.0),
            Position::new(-3.95, 4.0),
            initial_velocity1,
        );
        let mut cell2 = Cell::ball(
            Length::new(15.0),
            Mass::new(100.0),
            Position::new(10.0, -10.0),
            Velocity::ZERO,
        );
        let overlap1 = calc_overlap(&cell1, &cell2).unwrap();

        PairCollisions::add_forces(&mut cell1, &mut cell2, overlap1);

        cell1.tick();
        assert_eq!(
            cell1.velocity().x().signum(),
            -initial_velocity1.x().signum()
        );
        assert_eq!(
            cell1.velocity().y().signum(),
            -initial_velocity1.y().signum()
        );
        assert_eq!(
            ((cell1.velocity().y() / cell1.velocity().x()).abs() * 100.0).round(),
            ((initial_velocity1.x() / initial_velocity1.y()).abs() * 100.0).round()
        );
    }

    #[test]
    fn pair_collision_force_is_zero_when_pair_is_separating_quickly() {
        let initial_velocity1 = Velocity::new(-10.0, 10.0);
        let initial_velocity2 = Velocity::ZERO;
        let mut cell1 = Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(-0.5, 0.5),
            initial_velocity1,
        );
        let mut cell2 = Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::ORIGIN,
            initial_velocity2,
        );
        let overlap1 = calc_overlap(&cell1, &cell2).unwrap();

        PairCollisions::add_forces(&mut cell1, &mut cell2, overlap1);

        cell1.tick();
        assert_eq!(cell1.velocity(), initial_velocity1);
        cell2.tick();
        assert_eq!(cell2.velocity(), initial_velocity2);
    }

    #[test]
    fn pair_collision_force_undoes_overlap_of_unmoving_cells() {
        let mut cell1 = Cell::ball(
            Length::new(8.0),
            Mass::new(2.0),
            Position::new(-9.0, 12.0),
            Velocity::ZERO,
        );
        let mut cell2 = Cell::ball(
            Length::new(12.0),
            Mass::new(6.0),
            Position::ORIGIN,
            Velocity::ZERO,
        );
        let overlap1 = calc_overlap(&cell1, &cell2).unwrap();

        PairCollisions::add_forces(&mut cell1, &mut cell2, overlap1);

        cell1.tick();
        cell2.tick();
        assert_just_touching(&cell1, &cell2);
    }

    #[test]
    fn pair_collision_force_undoes_overlap_of_slowly_separating_cells() {
        let mut cell1 = Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(0.0, 0.5),
            Velocity::new(0.0, 0.2),
        );
        let mut cell2 = Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::ORIGIN,
            Velocity::ZERO,
        );
        let overlap1 = calc_overlap(&cell1, &cell2).unwrap();

        PairCollisions::add_forces(&mut cell1, &mut cell2, overlap1);

        cell1.tick();
        cell2.tick();
        assert_just_touching(&cell1, &cell2);
    }

    #[test]
    fn bond_forces_add_forces() {
        let mut cell_graph = NodeGraph::new();
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

        bond_forces.apply_to(&mut cell_graph, &mut SortableHandles::new());

        let ball1 = cell_graph.node(ball1_handle);
        assert_ne!(ball1.net_force().net_force().x(), 0.0);
        assert_ne!(ball1.net_force().net_force().y(), 0.0);

        let ball2 = cell_graph.node(ball2_handle);
        assert_ne!(ball2.net_force().net_force().x(), 0.0);
        assert_ne!(ball2.net_force().net_force().y(), 0.0);
    }

    #[test]
    fn bond_with_no_velocity_and_no_strain_adds_no_force() {
        let mut cell1 = Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(-2.0, 0.0),
            Velocity::ZERO,
        );
        let mut cell2 = Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::ORIGIN,
            Velocity::ZERO,
        );
        let strain1 = calc_bond_strain(&cell1, &cell2);

        BondForces::add_forces(&mut cell1, &mut cell2, strain1);

        cell1.tick();
        assert_eq!(cell1.velocity(), Velocity::ZERO);
        cell2.tick();
        assert_eq!(cell2.velocity(), Velocity::ZERO);
    }

    #[test]
    fn bond_clears_velocity_component_aligned_with_bond() {
        let mut cell1 = Cell::ball(
            Length::new(1.0),
            Mass::new(2.0),
            Position::new(-1.0, 0.0),
            Velocity::new(1.5, -0.5),
        );
        let mut cell2 = Cell::ball(
            Length::new(1.0),
            Mass::new(4.0),
            Position::ORIGIN,
            Velocity::ZERO,
        );
        let strain1 = calc_bond_strain(&cell1, &cell2);

        BondForces::add_forces(&mut cell1, &mut cell2, strain1);

        cell1.tick();
        cell2.tick();
        assert_eq!(cell1.velocity().x(), cell2.velocity().x());
    }

    #[test]
    fn bond_clears_strain() {
        let mut cell1 = Cell::ball(
            Length::new(1.0),
            Mass::new(2.0),
            Position::new(-3.5, 4.0),
            Velocity::ZERO,
        );
        let mut cell2 = Cell::ball(
            Length::new(1.0),
            Mass::new(6.0),
            Position::ORIGIN,
            Velocity::ZERO,
        );
        let strain1 = calc_bond_strain(&cell1, &cell2);

        BondForces::add_forces(&mut cell1, &mut cell2, strain1);

        cell1.tick();
        cell2.tick();
        assert_just_touching(&cell1, &cell2);
    }

    #[test]
    #[ignore]
    fn bond_clears_strain_even_with_distant_fast_cell() {
        let mut cell1 = Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::new(-10.0, 0.0),
            Velocity::new(-10.0, 0.0),
        );
        let mut cell2 = Cell::ball(
            Length::new(5.0),
            Mass::new(10.0),
            Position::ORIGIN,
            Velocity::ZERO,
        );
        let strain1 = calc_bond_strain(&cell1, &cell2);

        BondForces::add_forces(&mut cell1, &mut cell2, strain1);

        cell1.tick();
        cell2.tick();
        assert_just_touching(&cell1, &cell2);
    }

    #[test]
    fn bond_angle_forces_add_forces() {
        let mut cell_graph = NodeGraph::new();

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

        BondAngleForces::new().apply_to(&mut cell_graph, &mut SortableHandles::new());

        let ball3 = cell_graph.node(ball3_handle);
        assert!(ball3.net_force().net_force().x() < 0.0);
    }

    #[test]
    fn simple_force_influence_adds_force() {
        let force = Force::new(2.0, -3.0);
        let influence = SimpleForceInfluence::new(Box::new(ConstantForce::new(force)));
        let mut ball = Cell::ball(
            Length::new(1.0),
            Mass::new(3.0),
            Position::new(0.0, 0.0),
            Velocity::ZERO,
        );

        influence.apply_to(&mut ball);

        assert_eq!(ball.net_force().net_force(), force);
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
        let mut cell =
            simple_layered_cell(vec![simple_cell_layer(Area::new(PI), Density::new(1.0))]);

        sunlight.apply_to(&mut cell);

        assert_eq!(cell.environment().light_intensity(), 15.0);
    }

    #[test]
    fn sunlight_never_negative() {
        let sunlight = Sunlight::new(-10.0, 0.0, 0.0, 10.0);
        let mut cell =
            simple_layered_cell(vec![simple_cell_layer(Area::new(1.0), Density::new(1.0))])
                .with_initial_position(Position::new(0.0, -11.0));

        sunlight.apply_to(&mut cell);

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

    fn assert_just_touching(cell1: &Cell, cell2: &Cell) {
        assert_eq_within(
            (cell1.center() - cell2.center()).length().value(),
            (cell1.radius() + cell2.radius()).value(),
            0.0001,
        );
    }

    fn assert_eq_within(val1: Value1D, val2: Value1D, fraction: Value1D) {
        let difference = (val1 - val2).abs();
        let average = (val1.abs() + val2.abs()) / 2.0;
        assert!(
            difference <= fraction * average,
            "Left:  {}\nRight: {}",
            val1,
            val2
        );
    }
}
