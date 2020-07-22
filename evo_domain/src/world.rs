use crate::biology::cell::Cell;
use crate::biology::changes::*;
use crate::environment::influences::*;
use crate::physics::bond::*;
use crate::physics::quantities::*;
use crate::physics::sortable_graph::*;
use std::collections::HashSet;
use std::iter::FromIterator;

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
            .with_pair_collisions()
            .with_influences(vec![
                Box::new(BondForces::new()),
                Box::new(BondAngleForces::new()),
            ])
    }

    pub fn with_perimeter_walls(self) -> Self {
        let world_min_corner = self.min_corner();
        let world_max_corner = self.max_corner();
        self.with_influence(Box::new(WallCollisions::new(
            world_min_corner,
            world_max_corner,
        )))
    }

    pub fn with_pair_collisions(self) -> Self {
        self.with_influence(Box::new(PairCollisions::new()))
    }

    pub fn with_sunlight(self, min_intensity: f64, max_intensity: f64) -> Self {
        let world_min_corner = self.min_corner();
        let world_max_corner = self.max_corner();
        self.with_influence(Box::new(Sunlight::new(
            world_min_corner.y(),
            world_max_corner.y(),
            min_intensity,
            max_intensity,
        )))
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

    pub fn add_cell(&mut self, cell: Cell) -> NodeHandle {
        self.cell_graph.add_node(cell)
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cell_graph.nodes()
    }

    pub fn cell(&self, handle: NodeHandle) -> &Cell {
        self.cell_graph.node(handle)
    }

    fn cell_mut(&mut self, handle: NodeHandle) -> &mut Cell {
        self.cell_graph.node_mut(handle)
    }

    pub fn with_bonds(mut self, index_pairs: Vec<(usize, usize)>) -> Self {
        for pair in index_pairs {
            let bond = Bond::new(&self.cells()[pair.0], &self.cells()[pair.1]);
            self.add_bond(bond, 1, 0);
        }
        self
    }

    pub fn add_bond(&mut self, bond: Bond, bond_index_on_cell1: usize, bond_index_on_cell2: usize) {
        self.cell_graph
            .add_edge(bond, bond_index_on_cell1, bond_index_on_cell2);
    }

    pub fn bonds(&self) -> &[Bond] {
        &self.cell_graph.edges()
    }

    pub fn bond(&self, handle: EdgeHandle) -> &Bond {
        &self.cell_graph.edge(handle)
    }

    pub fn with_angle_gussets(mut self, index_pairs_with_angles: Vec<(usize, usize, f64)>) -> Self {
        for tuple in index_pairs_with_angles {
            let gusset = AngleGusset::new(
                &self.bonds()[tuple.0],
                &self.bonds()[tuple.1],
                Angle::from_radians(tuple.2),
            );
            self.add_angle_gusset(gusset);
        }
        self
    }

    pub fn add_angle_gusset(&mut self, gusset: AngleGusset) {
        self.cell_graph.add_meta_edge(gusset);
    }

    pub fn debug_print_cells(&self) {
        println!("{:#?}", self.cell_graph);
    }

    pub fn toggle_select_cell_at(&mut self, pos: Position) {
        for cell in self.cell_graph.nodes_mut() {
            if cell.overlaps(pos) {
                cell.set_selected(!cell.is_selected());
                return;
            }
        }
    }

    pub fn tick(&mut self) {
        self.apply_influences();
        let cell_bond_requests = self.tick_cells();
        self.apply_world_changes(&cell_bond_requests);
    }

    fn apply_influences(&mut self) {
        for influence in &self.influences {
            influence.apply(&mut self.cell_graph);
        }
    }

    fn tick_cells(&mut self) -> Vec<BondRequests> {
        let mut cell_bond_requests = vec![];
        for cell in self.cell_graph.nodes_mut() {
            cell_bond_requests.push(cell.tick());
        }
        cell_bond_requests
    }

    fn apply_world_changes(&mut self, cell_bond_requests: &[BondRequests]) {
        let mut donated_energy = vec![];
        let mut new_children = vec![];
        let mut broken_bond_handles = HashSet::new();
        let mut dead_cell_handles = vec![];
        self.cell_graph.for_each_node(|index, cell, edge_source| {
            Self::execute_bond_requests(
                cell,
                edge_source,
                &cell_bond_requests[index],
                &mut donated_energy,
                &mut new_children,
                &mut broken_bond_handles,
            );
            if !cell.is_alive() {
                dead_cell_handles.push(cell.node_handle());
            }
        });
        self.apply_donated_energy(donated_energy);
        self.update_cell_graph(new_children, broken_bond_handles, dead_cell_handles);
    }

    fn execute_bond_requests(
        cell: &mut Cell,
        edge_source: &mut EdgeSource<Bond>,
        bond_requests: &BondRequests,
        donated_energy: &mut Vec<(NodeHandle, BioEnergy)>,
        new_children: &mut Vec<NewChildData>,
        broken_bond_handles: &mut HashSet<EdgeHandle>,
    ) {
        for (index, bond_request) in bond_requests.iter().enumerate() {
            if bond_request.retain_bond {
                if bond_request.donation_energy != BioEnergy::ZERO {
                    if cell.has_edge(index) {
                        let bond = edge_source.edge(cell.edge_handle(index));
                        donated_energy.push((
                            bond.other_node_handle(cell.node_handle()),
                            bond_request.donation_energy,
                        ));
                    } else {
                        let child = cell.create_and_place_child_cell(
                            bond_request.budding_angle,
                            bond_request.donation_energy,
                        );
                        new_children.push(NewChildData {
                            parent: cell.node_handle(),
                            bond_index: index,
                            child,
                        });
                    }
                }
            } else if cell.has_edge(index) {
                broken_bond_handles.insert(cell.edge_handle(index));
            }
        }
    }

    fn update_cell_graph(
        &mut self,
        new_children: Vec<NewChildData>,
        broken_bond_handles: HashSet<EdgeHandle>,
        dead_cell_handles: Vec<NodeHandle>,
    ) {
        self.add_children(new_children);
        self.remove_bonds(&broken_bond_handles);
        self.cell_graph.remove_nodes(&dead_cell_handles);
    }

    fn add_children(&mut self, new_children: Vec<NewChildData>) {
        for new_child_data in new_children {
            let child_handle = self.add_cell(new_child_data.child);
            let bond = Bond::new(self.cell(new_child_data.parent), self.cell(child_handle));
            self.add_bond(bond, new_child_data.bond_index, 0);
        }
    }

    fn apply_donated_energy(&mut self, donated_energy: Vec<(NodeHandle, BioEnergy)>) {
        for (cell_handle, donation) in donated_energy {
            self.cell_mut(cell_handle).add_energy(donation);
        }
    }

    fn remove_bonds(&mut self, bond_handles: &HashSet<EdgeHandle>) {
        let mut sorted_bond_handles = Vec::from_iter(bond_handles.iter().cloned());
        sorted_bond_handles.sort_unstable();
        self.cell_graph.remove_edges(&sorted_bond_handles);
    }
}

struct NewChildData {
    parent: NodeHandle,
    bond_index: usize,
    child: Cell,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::control::*;
    use crate::biology::layers::*;
    use crate::environment::local_environment::*;
    use crate::physics::newtonian::NewtonianBody;
    use crate::physics::overlap::Overlap;
    use crate::physics::shapes::*;

    #[test]
    fn tick_moves_ball() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN).with_cell(Cell::ball(
            Length::new(1.0),
            Mass::new(1.0),
            Position::ORIGIN,
            Velocity::new(1.0, 1.0),
        ));

        world.tick();

        let ball = &world.cells()[0];
        assert!(ball.position().x() > 0.0);
        assert!(ball.position().y() > 0.0);
    }

    #[test]
    fn tick_with_force_accelerates_ball() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_influence(Box::new(SimpleForceInfluence::new(Box::new(
                ConstantForce::new(Force::new(1.0, 1.0)),
            ))))
            .with_cell(Cell::ball(
                Length::new(1.0),
                Mass::new(1.0),
                Position::ORIGIN,
                Velocity::ZERO,
            ));

        world.tick();

        let ball = &world.cells()[0];
        assert!(ball.velocity().x() > 0.0);
        assert!(ball.velocity().y() > 0.0);
    }

    #[test]
    fn overlaps_do_not_persist() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_influence(Box::new(UniversalOverlap::new(Overlap::new(
                Displacement::new(1.0, 1.0),
                1.0,
            ))))
            .with_cell(Cell::ball(
                Length::new(1.0),
                Mass::new(1.0),
                Position::ORIGIN,
                Velocity::ZERO,
            ));

        world.tick();

        let ball = &world.cells()[0];
        assert!(ball.environment().overlaps().is_empty());
    }

    #[test]
    fn forces_do_not_persist() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_influence(Box::new(SimpleForceInfluence::new(Box::new(
                ConstantForce::new(Force::new(1.0, 1.0)),
            ))))
            .with_cell(Cell::ball(
                Length::new(1.0),
                Mass::new(1.0),
                Position::ORIGIN,
                Velocity::ZERO,
            ));

        world.tick();

        let ball = &world.cells()[0];
        assert_eq!(ball.forces().net_force(), Force::new(0.0, 0.0));
    }

    #[test]
    fn cannot_bounce_off_drag_force() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_cell(Cell::ball(
                Length::new(10.0),
                Mass::new(0.01),
                Position::ORIGIN,
                Velocity::new(10.0, 10.0),
            ))
            .with_influence(Box::new(SimpleForceInfluence::new(Box::new(
                DragForce::new(0.01),
            ))));

        world.tick();

        let ball = &world.cells()[0];
        assert!(ball.velocity().x() >= 0.0);
        assert!(ball.velocity().y() >= 0.0);
    }

    #[test]
    fn tick_runs_photo_layer() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_influence(Box::new(Sunlight::new(-10.0, 10.0, 0.0, 10.0)))
            .with_cell(simple_layered_cell(vec![CellLayer::new(
                Area::new(10.0),
                Density::new(1.0),
                Color::Green,
                Box::new(PhotoCellLayerSpecialty::new(1.0)),
            )]));

        world.tick();

        let cell = &world.cells()[0];
        assert_eq!(cell.energy().value().round(), 50.0);
    }

    #[test]
    fn tick_runs_cell_growth() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN).with_cell(
            simple_layered_cell(vec![CellLayer::new(
                Area::new(1.0),
                Density::new(1.0),
                Color::Green,
                Box::new(NullCellLayerSpecialty::new()),
            )])
            .with_control(Box::new(ContinuousResizeControl::new(
                0,
                AreaDelta::new(2.0),
            ))),
        );

        world.tick();

        let cell = &world.cells()[0];
        assert_eq!(cell.area(), Area::new(3.0));
    }

    #[test]
    fn tick_runs_cell_thruster() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0))
            .with_cell(
                simple_layered_cell(vec![CellLayer::new(
                    Area::new(1.0),
                    Density::new(1.0),
                    Color::Green,
                    Box::new(ThrusterCellLayerSpecialty::new()),
                )])
                .with_control(Box::new(SimpleThrusterControl::new(
                    0,
                    Force::new(1.0, -1.0),
                ))),
            );

        world.tick();
        world.tick();

        let cell = &world.cells()[0];
        assert!(cell.velocity().x() > 0.0);
        assert!(cell.velocity().y() < 0.0);
    }

    #[test]
    fn growth_is_limited_by_energy() {
        const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
            growth_energy_delta: BioEnergyDelta::new(-10.0),
            ..LayerResizeParameters::UNLIMITED
        };

        let mut world = World::new(Position::ORIGIN, Position::ORIGIN).with_cell(
            simple_layered_cell(vec![simple_cell_layer(Area::new(10.0), Density::new(1.0))
                .with_resize_parameters(&LAYER_RESIZE_PARAMS)])
            .with_control(Box::new(ContinuousResizeControl::new(
                0,
                AreaDelta::new(100.0),
            )))
            .with_initial_energy(BioEnergy::new(10.0)),
        );

        world.tick();

        let cell = &world.cells()[0];
        assert_eq!(cell.area().value().round(), 11.0);
    }

    #[test]
    fn new_cell_is_added_to_world_with_bond_to_parent() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN).with_cell(
            Cell::new(
                Position::ORIGIN,
                Velocity::ZERO,
                vec![CellLayer::new(
                    Area::new(1.0),
                    Density::new(1.0),
                    Color::Green,
                    Box::new(BondingCellLayerSpecialty::new()),
                )],
            )
            .with_control(Box::new(ContinuousRequestsControl::new(vec![
                BondingCellLayerSpecialty::retain_bond_request(0, 1, true),
                BondingCellLayerSpecialty::donation_energy_request(0, 1, BioEnergy::new(1.0)),
            ])))
            .with_initial_energy(BioEnergy::new(10.0)),
        );

        world.tick();

        assert_eq!(world.cells().len(), 2);
        assert_eq!(world.bonds().len(), 1);
        let parent = &world.cells()[0];
        assert!(parent.has_edge(1));
        let child = &world.cells()[1];
        assert!(child.has_edge(0));
        assert_eq!(parent.energy(), BioEnergy::new(9.0)); // 10 - 1
        assert_eq!(child.energy(), BioEnergy::new(1.0)); // 0 + 1
    }

    #[test]
    fn cells_can_pass_energy_through_bond() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_cells(vec![
                Cell::new(
                    Position::ORIGIN,
                    Velocity::ZERO,
                    vec![CellLayer::new(
                        Area::new(1.0),
                        Density::new(1.0),
                        Color::Green,
                        Box::new(BondingCellLayerSpecialty::new()),
                    )],
                )
                .with_control(Box::new(ContinuousRequestsControl::new(vec![
                    BondingCellLayerSpecialty::retain_bond_request(0, 1, true),
                    BondingCellLayerSpecialty::donation_energy_request(0, 1, BioEnergy::new(2.0)),
                ])))
                .with_initial_energy(BioEnergy::new(10.0)),
                Cell::new(
                    Position::ORIGIN,
                    Velocity::ZERO,
                    vec![CellLayer::new(
                        Area::new(1.0),
                        Density::new(1.0),
                        Color::Green,
                        Box::new(BondingCellLayerSpecialty::new()),
                    )],
                )
                .with_control(Box::new(ContinuousRequestsControl::new(vec![
                    BondingCellLayerSpecialty::retain_bond_request(0, 0, true),
                    BondingCellLayerSpecialty::donation_energy_request(0, 0, BioEnergy::new(3.0)),
                ])))
                .with_initial_energy(BioEnergy::new(10.0)),
            ])
            .with_bonds(vec![(0, 1)]);

        world.tick();

        assert_eq!(world.cells().len(), 2);
        assert_eq!(world.bonds().len(), 1);
        let cell1 = &world.cells()[0];
        assert_eq!(cell1.energy(), BioEnergy::new(11.0)); // 10 - 2 + 3
        let cell2 = &world.cells()[1];
        assert_eq!(cell2.energy(), BioEnergy::new(9.0)); // 10 - 3 + 2
    }

    #[test]
    fn world_breaks_bond_when_requested() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_cells(vec![
                simple_layered_cell(vec![CellLayer::new(
                    Area::new(1.0),
                    Density::new(1.0),
                    Color::Green,
                    Box::new(BondingCellLayerSpecialty::new()),
                )])
                .with_control(Box::new(ContinuousRequestsControl::new(vec![
                    BondingCellLayerSpecialty::retain_bond_request(0, 1, false),
                ]))),
                simple_layered_cell(vec![simple_cell_layer(Area::new(1.0), Density::new(1.0))]),
            ])
            .with_bonds(vec![(0, 1)]);

        world.tick();

        assert_eq!(world.bonds().len(), 0);
    }

    #[test]
    fn dead_cells_get_removed_from_world() {
        let mut world =
            World::new(Position::ORIGIN, Position::ORIGIN).with_cell(simple_layered_cell(vec![
                simple_cell_layer(Area::new(1.0), Density::new(1.0)).dead(),
            ]));

        world.tick();

        assert_eq!(world.cells().len(), 0);
    }

    #[test]
    fn tick_applies_all_changes() {
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN).with_cell(
            Cell::new(
                Position::ORIGIN,
                Velocity::ZERO,
                vec![CellLayer::new(
                    Area::new(1.0),
                    Density::new(1.0),
                    Color::Green,
                    Box::new(NullCellLayerSpecialty::new()),
                )
                .with_health(Health::new(0.5))],
            )
            .with_control(Box::new(ContinuousRequestsControl::new(vec![
                CellLayer::healing_request(0, HealthDelta::new(0.5)),
                CellLayer::resize_request(0, AreaDelta::new(4.0)),
            ])))
            .with_initial_energy(BioEnergy::new(10.0)),
        );

        world.tick();

        let cell = &world.cells()[0];
        let layer = &cell.layers()[0];
        assert_eq!(layer.health(), Health::FULL);
        assert_eq!(layer.area(), Area::new(3.0));
        // assert_eq!(cell.energy(), BioEnergy::ZERO);
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
