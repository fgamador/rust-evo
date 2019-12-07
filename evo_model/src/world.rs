use crate::biology::cell::Cell;
use crate::environment::local_environment::*;
use crate::environment::influences::*;
use crate::physics::bond::*;
use crate::physics::newtonian::NewtonianBody;
use crate::physics::quantities::*;
use crate::physics::sortable_graph::*;
use crate::physics::spring::*;
use crate::TickCallbacks;

pub struct World {
    min_corner: Position,
    max_corner: Position,
    cell_graph: SortableGraph<Cell, Bond, AngleGusset>,
    influences: Vec<Box<dyn Influence>>,
}

impl World {
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        World {
            min_corner,
            max_corner,
            cell_graph: SortableGraph::new(),
            influences: vec![],
        }
    }

    pub fn with_standard_influences(self) -> Self {
        self.with_perimeter_walls()
            .with_influences(vec![
                Box::new(PairCollisions::new()),
                Box::new(BondForces::new()),
                Box::new(BondAngleForces::new()),
            ])
    }

    pub fn with_perimeter_walls(self) -> Self {
        let world_min_corner = self.min_corner();
        let world_max_corner = self.max_corner();
        self.with_influence(Box::new(
            WallCollisions::new(world_min_corner, world_max_corner,
                                Box::new(LinearSpring::new(1.0)))))
    }

    pub fn with_sunlight(self, min_intensity: f64, max_intensity: f64) -> Self {
        let world_min_corner = self.min_corner();
        let world_max_corner = self.max_corner();
        self.with_influence(Box::new(
            Sunlight::new(world_min_corner.y(), world_max_corner.y(),
                          min_intensity, max_intensity)))
    }

    pub fn with_influence(mut self, influence: Box<dyn Influence>) -> Self {
        self.influences.push(influence);
        self
    }

    pub fn with_influences(mut self, mut influences: Vec<Box<dyn Influence>>) -> Self {
        self.influences.append(&mut influences);
        self
    }

    pub fn min_corner(&self) -> Position {
        self.min_corner
    }

    pub fn max_corner(&self) -> Position {
        self.max_corner
    }

    pub fn with_cell(mut self, cell: Cell) -> Self {
        self.add_cell(cell);
        self
    }

    pub fn with_cells(mut self, cells: Vec<Cell>) -> Self {
        for cell in cells {
            self.add_cell(cell);
        }
        self
    }

    pub fn add_cell(&mut self, cell: Cell) {
        self.cell_graph.add_node(cell);
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cell_graph.unsorted_nodes()
    }

    pub fn with_bonds(mut self, index_pairs: Vec<(usize, usize)>) -> Self {
        for pair in index_pairs {
            let bond = Bond::new(&self.cells()[pair.0], &self.cells()[pair.1]);
            self.add_bond(bond);
        }
        self
    }

    pub fn add_bond(&mut self, bond: Bond) {
        self.cell_graph.add_edge(bond);
    }

    pub fn bonds(&self) -> &[Bond] {
        &self.cell_graph.edges()
    }

    pub fn with_angle_gussets(mut self, index_pairs_with_angles: Vec<(usize, usize, f64)>) -> Self {
        for tuple in index_pairs_with_angles {
            let gusset = AngleGusset::new(&self.bonds()[tuple.0], &self.bonds()[tuple.1], Angle::from_radians(tuple.2));
            self.add_angle_gusset(gusset);
        }
        self
    }

    pub fn add_angle_gusset(&mut self, gusset: AngleGusset) {
        self.cell_graph.add_meta_edge(gusset);
    }

    pub fn tick(&mut self) {
        let tick_duration = Duration::new(1.0);
        let subticks_per_tick = 2;
        let subtick_duration = tick_duration / (subticks_per_tick as f64);

        for _subtick in 0..subticks_per_tick {
            self.apply_influences();
            self.after_influences(subtick_duration);
            self.exert_forces(subtick_duration);
            self.clear_influences();
        }

        self.after_movement();
    }

    fn apply_influences(&mut self) {
        for influence in &self.influences {
            influence.apply(&mut self.cell_graph);
        }
    }

    fn after_influences(&mut self, subtick_duration: Duration) {
        for cell in self.cell_graph.unsorted_nodes_mut() {
            cell.after_influences(subtick_duration);
        }
    }

    fn exert_forces(&mut self, subtick_duration: Duration) {
        for cell in self.cell_graph.unsorted_nodes_mut() {
            cell.exert_forces(subtick_duration);
            cell.move_for(subtick_duration);
        }
    }

    fn clear_influences(&mut self) {
        for cell in self.cell_graph.unsorted_nodes_mut() {
            cell.environment_mut().clear();
            cell.forces_mut().clear();
        }
    }

    fn after_movement(&mut self) {
        let mut new_cells: Vec<Cell> = vec![];
        for cell in self.cell_graph.unsorted_nodes_mut() {
            let (_alive, mut cell_children) = cell.after_movement();
            new_cells.append(&mut cell_children);
        }
        for child in new_cells {
            self.add_cell(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::control::*;
    use crate::biology::layers::*;
    use crate::physics::overlap::Overlap;
    use crate::physics::shapes::*;

    #[test]
    fn tick_moves_ball() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0))
            .with_cell(Cell::ball(Length::new(1.0), Mass::new(1.0),
                                  Position::new(0.0, 0.0), Velocity::new(1.0, 1.0)));

        world.tick();

        let ball = &world.cells()[0];
        assert!(ball.position().x() > 0.0);
        assert!(ball.position().y() > 0.0);
    }

    #[test]
    fn tick_with_force_accelerates_ball() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0))
            .with_influence(Box::new(SimpleForceInfluence::new(Box::new(ConstantForce::new(Force::new(1.0, 1.0))))))
            .with_cell(Cell::ball(Length::new(1.0), Mass::new(1.0),
                                  Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));

        world.tick();

        let ball = &world.cells()[0];
        assert!(ball.velocity().x() > 0.0);
        assert!(ball.velocity().y() > 0.0);
    }

    #[test]
    fn overlaps_do_not_persist() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0))
            .with_influence(Box::new(UniversalOverlap::new(Overlap::new(Displacement::new(1.0, 1.0)))))
            .with_cell(Cell::ball(Length::new(1.0), Mass::new(1.0),
                                  Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));

        world.tick();

        let ball = &world.cells()[0];
        assert!(ball.environment().overlaps().is_empty());
    }

    #[test]
    fn forces_do_not_persist() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0))
            .with_influence(Box::new(SimpleForceInfluence::new(Box::new(ConstantForce::new(Force::new(1.0, 1.0))))))
            .with_cell(Cell::ball(Length::new(1.0), Mass::new(1.0),
                                  Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));

        world.tick();

        let ball = &world.cells()[0];
        assert_eq!(ball.forces().net_force(), Force::new(0.0, 0.0));
    }

    #[test]
    fn tick_runs_photo_layer() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0))
            .with_influence(Box::new(Sunlight::new(-10.0, 10.0, 0.0, 10.0)))
            .with_cell(Cell::new(Position::new(0.0, 0.0), Velocity::ZERO,
                                 vec![
                                     Box::new(CellLayer::new(Area::new(10.0), Density::new(1.0), Color::Green, Box::new(PhotoCellLayerSpecialty::new(1.0)))),
                                 ]));

        world.tick();

        let cell = &world.cells()[0];
        assert_eq!(cell.energy().value().round(), 50.0);
    }

    #[test]
    fn tick_runs_cell_growth() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0))
            .with_cell(Cell::new(Position::new(0.0, 0.0), Velocity::new(0.0, 0.0),
                                 vec![
                                     Box::new(CellLayer::new(
                                         Area::new(1.0), Density::new(1.0), Color::Green, Box::new(NullCellLayerSpecialty::new()))),
                                 ])
                .with_control(Box::new(ContinuousResizeControl::new(0, AreaDelta::new(2.0)))));

        world.tick();

        let cell = &world.cells()[0];
        assert_eq!(cell.area(), Area::new(3.0));
    }

    #[test]
    fn tick_runs_cell_thruster() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0))
            .with_cell(Cell::new(Position::new(0.0, 0.0), Velocity::new(0.0, 0.0),
                                 vec![
                                     Box::new(CellLayer::new(Area::new(1.0), Density::new(1.0), Color::Green,
                                                             Box::new(ThrusterCellLayerSpecialty::new()))),
                                 ])
                .with_control(Box::new(SimpleThrusterControl::new(0, Force::new(1.0, -1.0)))));

        world.tick();
        world.tick();

        let cell = &world.cells()[0];
        assert!(cell.velocity().x() > 0.0);
        assert!(cell.velocity().y() < 0.0);
    }

    #[test]
    fn growth_is_limited_by_energy() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0))
            .with_influence(Box::new(Sunlight::new(-10.0, 10.0, 0.0, 10.0)))
            .with_cell(Cell::new(Position::new(0.0, 0.0), Velocity::ZERO,
                                 vec![
                                     Box::new(CellLayer::new(Area::new(10.0), Density::new(1.0), Color::Green,
                                                             Box::new(PhotoCellLayerSpecialty::new(1.0)))
                                         .with_resize_parameters(LayerResizeParameters {
                                             growth_energy_delta: BioEnergyDelta::new(-10.0),
                                             ..LayerResizeParameters::UNLIMITED
                                         }))
                                 ])
                .with_control(Box::new(ContinuousResizeControl::new(0, AreaDelta::new(100.0)))));

        world.tick();

        let cell = &world.cells()[0];
        assert_eq!(cell.area().value().round(), 15.0);
    }

    #[test]
    fn new_cells_get_added_to_world() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_cell(Cell::new(Position::ORIGIN, Velocity::ZERO,
                                 vec![
                                     Box::new(CellLayer::new(Area::new(1.0), Density::new(1.0), Color::Green,
                                                             Box::new(BuddingCellLayerSpecialty::new(create_child))))
                                 ])
                .with_control(Box::new(ContinuousRequestsControl::new(vec![
                    BuddingCellLayerSpecialty::donation_energy_request(0, BioEnergy::new(1.0)),
                ]))));

        world.tick();

        assert_eq!(world.cells().len(), 2);
    }

    #[test]
    #[ignore]
    fn dead_cells_get_removed_from_world() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_cell(Cell::new(Position::ORIGIN, Velocity::ZERO,
                                 vec![
                                     Box::new(simple_cell_layer(Area::new(1.0), Density::new(1.0)).dead())
                                 ]));

        world.tick();

        assert_eq!(world.cells().len(), 0);
    }

    fn create_child() -> Cell {
        Cell::new(Position::ORIGIN, Velocity::ZERO,
                  vec![Box::new(simple_cell_layer(Area::new(1.0), Density::new(1.0)))])
    }

    fn simple_cell_layer(area: Area, density: Density) -> CellLayer {
        CellLayer::new(area, density, Color::Green, Box::new(NullCellLayerSpecialty::new()))
    }
}
