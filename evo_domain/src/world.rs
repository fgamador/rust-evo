use crate::biology::cell::Cell;
use crate::biology::changes::*;
use crate::biology::cloud::Cloud;
use crate::environment::influences::*;
use crate::physics::bond::*;
use crate::physics::handles::*;
use crate::physics::node_graph::*;
use crate::physics::overlap::{SortableHandle, SortableHandles};
use crate::physics::quantities::*;
use crate::physics::shapes::Circle;
use crate::Parameters;
use rayon::prelude::*;
use std::collections::HashSet;
use std::io;
use std::io::{Result, StdoutLock, Write};

pub struct World {
    parameters: Parameters,
    min_corner: Position,
    max_corner: Position,
    cell_graph: NodeGraph<Cell, Bond<Cell>, AngleGusset>,
    clouds: ObjectsWithHandles<Cloud>,
    circle_handles: SortableHandles<Cell>,
    cross_cell_influences: Vec<Box<dyn CrossCellInfluence>>,
    per_cell_influences: Vec<Box<dyn PerCellInfluence>>,
    num_selected_cells: u32,
}

impl World {
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        World {
            parameters: Parameters::DEFAULT,
            min_corner,
            max_corner,
            cell_graph: NodeGraph::new(),
            clouds: ObjectsWithHandles::new(),
            circle_handles: SortableHandles::new(),
            cross_cell_influences: vec![],
            per_cell_influences: vec![],
            num_selected_cells: 0,
        }
    }

    pub fn with_parameters(mut self, parameters: Parameters) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn with_standard_influences(self) -> Self {
        self.with_perimeter_walls()
            .with_pair_collisions(Fraction::ONE)
            .with_bond_forces()
    }

    pub fn with_perimeter_walls(self) -> Self {
        let world_min_corner = self.min_corner();
        let world_max_corner = self.max_corner();
        self.with_cross_cell_influence(Box::new(WallCollisions::new(
            world_min_corner,
            world_max_corner,
        )))
    }

    pub fn with_pair_collisions(self, force_adjustment_factor: Fraction) -> Self {
        self.with_cross_cell_influence(Box::new(PairCollisions::new(force_adjustment_factor)))
    }

    pub fn with_bond_forces(self) -> Self {
        self.with_cross_cell_influence(Box::new(BondForces::new()))
    }

    pub fn with_sunlight(self, min_intensity: Value1D, max_intensity: Value1D) -> Self {
        let world_min_corner = self.min_corner();
        let world_max_corner = self.max_corner();
        self.with_per_cell_influence(Box::new(Sunlight::new(
            world_min_corner.y(),
            world_max_corner.y(),
            min_intensity,
            max_intensity,
        )))
    }

    pub fn with_cross_cell_influence(mut self, influence: Box<dyn CrossCellInfluence>) -> Self {
        self.cross_cell_influences.push(influence);
        self
    }

    pub fn with_cross_cell_influences(
        mut self,
        mut influences: Vec<Box<dyn CrossCellInfluence>>,
    ) -> Self {
        self.cross_cell_influences.append(&mut influences);
        self
    }

    pub fn with_per_cell_influence(mut self, influence: Box<dyn PerCellInfluence>) -> Self {
        self.per_cell_influences.push(influence);
        self
    }

    pub fn with_per_cell_influences(
        mut self,
        mut influences: Vec<Box<dyn PerCellInfluence>>,
    ) -> Self {
        self.per_cell_influences.append(&mut influences);
        self
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

    pub fn with_bonds(mut self, index_pairs: Vec<(usize, usize)>) -> Self {
        for pair in index_pairs {
            let bond = Bond::new(&self.cells()[pair.0], &self.cells()[pair.1]);
            self.add_bond(bond, 1, 0);
        }
        self
    }

    pub fn with_angle_gussets(
        mut self,
        index_pairs_with_angles: Vec<(usize, usize, Value1D)>,
    ) -> Self {
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

    pub fn with_clouds(mut self, clouds: Vec<Cloud>) -> Self {
        for cloud in clouds {
            self.add_cloud(cloud);
        }
        self
    }

    pub fn min_corner(&self) -> Position {
        self.min_corner
    }

    pub fn max_corner(&self) -> Position {
        self.max_corner
    }

    pub fn add_cell(&mut self, cell: Cell) -> Handle<Cell> {
        let handle = self.cell_graph.add_node(cell);
        self.circle_handles
            .add_handle(SortableHandle::GraphNode(handle));
        handle
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cell_graph.nodes()
    }

    pub fn cell(&self, handle: Handle<Cell>) -> &Cell {
        self.cell_graph.node(handle)
    }

    fn cell_mut(&mut self, handle: Handle<Cell>) -> &mut Cell {
        self.cell_graph.node_mut(handle)
    }

    pub fn add_bond(
        &mut self,
        bond: Bond<Cell>,
        bond_index_on_cell1: usize,
        bond_index_on_cell2: usize,
    ) {
        self.cell_graph
            .add_edge(bond, bond_index_on_cell1, bond_index_on_cell2);
    }

    pub fn bonds(&self) -> &[Bond<Cell>] {
        &self.cell_graph.edges()
    }

    pub fn bond(&self, handle: EdgeHandle) -> &Bond<Cell> {
        &self.cell_graph.edge(handle)
    }

    pub fn add_angle_gusset(&mut self, gusset: AngleGusset) {
        self.cell_graph.add_meta_edge(gusset);
    }

    pub fn add_cloud(&mut self, cloud: Cloud) -> Handle<Cloud> {
        self.clouds.add(cloud)
        // TODO self.circle_handles
        //     .add_handle(SortableHandle::Cloud(handle));
        // handle
    }

    pub fn clouds(&self) -> &[Cloud] {
        &self.clouds.objects()
    }

    pub fn debug_print_cells(&self) {
        println!("{:#?}", self.cell_graph);
    }

    pub fn toggle_select_cell_at(&mut self, pos: Position) {
        for cell in self.cell_graph.nodes_mut() {
            if cell.overlaps(pos) {
                if cell.is_selected() {
                    cell.set_selected(false);
                    self.num_selected_cells -= 1;
                } else {
                    cell.set_selected(true);
                    self.num_selected_cells += 1;
                }
            }
        }
    }

    pub fn tick(&mut self) {
        self.apply_cross_cell_influences();
        let cell_bond_requests = self.tick_cells();
        self.tick_clouds();
        self.apply_world_changes(&cell_bond_requests);
        self.print_end_tick_info().unwrap();
    }

    fn apply_cross_cell_influences(&mut self) {
        for influence in &self.cross_cell_influences {
            influence.apply_to(&mut self.cell_graph, &mut self.circle_handles);
        }
    }

    fn tick_cells(&mut self) -> Vec<BondRequests> {
        let per_cell_influences = &self.per_cell_influences;
        self.cell_graph
            .nodes_mut()
            .par_iter_mut()
            .map(|cell| {
                for influence in per_cell_influences {
                    influence.apply_to(cell);
                }
                cell.tick()
            })
            .collect()
    }

    fn tick_clouds(&mut self) {
        for cloud in self.clouds.objects_mut() {
            cloud.tick(&self.parameters.cloud_params);
        }
    }

    fn apply_world_changes(&mut self, cell_bond_requests: &[BondRequests]) {
        let parameters = &self.parameters;
        let mut donated_energy = vec![];
        let mut new_children = vec![];
        let mut broken_bond_handles = HashSet::new();
        let mut burst_cell_handles = vec![];
        self.cell_graph.for_each_node(|index, cell, edge_source| {
            Self::execute_bond_requests(
                parameters,
                cell,
                edge_source,
                &cell_bond_requests[index],
                &mut donated_energy,
                &mut new_children,
                &mut broken_bond_handles,
            );
            if !cell.is_intact() {
                burst_cell_handles.push(cell.node_handle());
            }
        });
        self.apply_donated_energy(donated_energy);
        for burst_cell_handle in &burst_cell_handles {
            if self.cell(*burst_cell_handle).is_selected() {
                self.num_selected_cells -= 1;
            }
        }
        self.add_clouds_for_burst_cells(&burst_cell_handles);
        self.update_cell_graph(new_children, broken_bond_handles, burst_cell_handles);
        self.remove_nonexistent_clouds();
        self.update_circle_handles();
    }

    fn execute_bond_requests(
        parameters: &Parameters,
        cell: &mut Cell,
        edge_source: &mut EdgeSource<Cell, Bond<Cell>>,
        bond_requests: &BondRequests,
        donated_energy: &mut Vec<(Handle<Cell>, BioEnergy)>,
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
                            parameters.initial_layer_area,
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

    fn add_clouds_for_burst_cells(&mut self, burst_cell_handles: &[Handle<Cell>]) {
        for handle in burst_cell_handles {
            self.clouds
                .add(Self::cloud_for_burst_cell(self.cell(*handle)));
        }
    }

    fn cloud_for_burst_cell(cell: &Cell) -> Cloud {
        Cloud::new(cell.center(), cell.radius())
    }

    fn update_cell_graph(
        &mut self,
        new_children: Vec<NewChildData>,
        broken_bond_handles: HashSet<EdgeHandle>,
        burst_cell_handles: Vec<Handle<Cell>>,
    ) {
        self.add_children(new_children);
        self.remove_bonds(&broken_bond_handles);
        self.cell_graph.remove_nodes(&burst_cell_handles);
    }

    fn add_children(&mut self, new_children: Vec<NewChildData>) {
        for new_child_data in new_children {
            let child_handle = self.add_cell(new_child_data.child);
            let bond = Bond::new(self.cell(new_child_data.parent), self.cell(child_handle));
            self.add_bond(bond, new_child_data.bond_index, 0);
        }
    }

    fn apply_donated_energy(&mut self, donated_energy: Vec<(Handle<Cell>, BioEnergy)>) {
        for (cell_handle, donation) in donated_energy {
            self.cell_mut(cell_handle)
                .add_received_donated_energy(donation);
        }
    }

    fn remove_bonds(&mut self, bond_handles: &HashSet<EdgeHandle>) {
        let mut sorted_bond_handles: Vec<EdgeHandle> = bond_handles.iter().cloned().collect();
        sorted_bond_handles.sort_unstable();
        self.cell_graph.remove_edges(&sorted_bond_handles);
    }

    fn remove_nonexistent_clouds(&mut self) {
        let non_existent_cloud_handles: Vec<Handle<Cloud>> = self
            .clouds
            .iter()
            .filter_map(|cloud| {
                if cloud.exists(&self.parameters.cloud_params) {
                    None
                } else {
                    Some(cloud.handle())
                }
            })
            .collect();
        self.clouds
            .remove_all(&non_existent_cloud_handles, |_, _| {});
    }

    fn update_circle_handles(&mut self) {
        let cell_graph = &self.cell_graph;
        self.circle_handles.remove_invalid_handles(|h| match h {
            SortableHandle::GraphNode(h) => cell_graph.is_valid_handle(h),
            SortableHandle::Cloud => false,
        });
    }

    fn print_end_tick_info(&self) -> Result<()> {
        if self.num_selected_cells == 0 {
            return Ok(());
        }

        let stdout = io::stdout();
        let mut out = stdout.lock();

        self.print_bonds_info(&mut out)?;
        writeln!(
            out,
            "End of tick: {} cells, {} bonds",
            self.cells().len(),
            self.bonds().len()
        )
    }

    fn print_bonds_info(&self, out: &mut StdoutLock) -> Result<()> {
        for bond in self.bonds() {
            let cell1 = self.cell(bond.node1_handle());
            let cell2 = self.cell(bond.node2_handle());
            if cell1.is_selected() || cell2.is_selected() {
                Self::print_bond_info(out, cell1, cell2, bond)?;
            }
        }
        Ok(())
    }

    fn print_bond_info(
        out: &mut StdoutLock,
        cell1: &Cell,
        cell2: &Cell,
        bond: &Bond<Cell>,
    ) -> Result<()> {
        let bond_index1 = cell1
            .graph_node_data()
            .index_of_edge_handle(bond.edge_handle())
            .unwrap();
        let bond_index2 = cell2
            .graph_node_data()
            .index_of_edge_handle(bond.edge_handle())
            .unwrap();
        writeln!(
            out,
            "Bond: Cell {} bond {} - Cell {} bond {}",
            cell1.node_handle(),
            bond_index1,
            cell2.node_handle(),
            bond_index2
        )
    }
}

struct NewChildData {
    parent: Handle<Cell>,
    bond_index: usize,
    child: Cell,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::cloud::CloudParameters;
    use crate::biology::control::*;
    use crate::biology::layers::*;
    use crate::environment::local_environment::*;
    use crate::physics::newtonian::NewtonianBody;
    use crate::physics::overlap::Overlap;
    use std::f64::consts::PI;

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
            .with_per_cell_influence(Box::new(SimpleForceInfluence::new(Box::new(
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
            .with_per_cell_influence(Box::new(UniversalOverlap::new(Overlap::new(
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
            .with_per_cell_influence(Box::new(SimpleForceInfluence::new(Box::new(
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
        assert_eq!(ball.net_force().net_force(), Force::new(0.0, 0.0));
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
            .with_per_cell_influence(Box::new(SimpleForceInfluence::new(Box::new(
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
            .with_per_cell_influence(Box::new(Sunlight::new(-10.0, 10.0, 0.0, 10.0)))
            .with_cell(simple_layered_cell(vec![CellLayer::new(
                Area::new(10.0),
                Density::new(1.0),
                Tissue::Photosynthetic,
                Box::new(PhotoCellLayerSpecialty::new(Fraction::ONE)),
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
                Tissue::Photosynthetic,
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
                    Tissue::Photosynthetic,
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
        const LAYER_PARAMS: LayerParameters = LayerParameters {
            growth_energy_delta: BioEnergyDelta::new(-10.0),
            ..LayerParameters::DEFAULT
        };

        let mut world = World::new(Position::ORIGIN, Position::ORIGIN).with_cell(
            simple_layered_cell(vec![simple_cell_layer(Area::new(10.0), Density::new(1.0))
                .with_parameters(&LAYER_PARAMS)])
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
                    Tissue::Photosynthetic,
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
                        Tissue::Photosynthetic,
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
                        Tissue::Photosynthetic,
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
                    Tissue::Photosynthetic,
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
    fn world_removes_burst_cells() {
        const LAYER_PARAMS: LayerParameters = LayerParameters {
            minimum_intact_thickness: Fraction::unchecked(0.5),
            ..LayerParameters::DEFAULT
        };

        let mut world =
            World::new(Position::ORIGIN, Position::ORIGIN).with_cell(simple_layered_cell(vec![
                simple_cell_layer(Area::new(1.0), Density::new(1.0)),
                simple_cell_layer(Area::new(0.1), Density::new(1.0))
                    .with_parameters(&LAYER_PARAMS)
                    .dead(),
            ]));

        world.tick();

        assert_eq!(world.cells().len(), 0);
    }

    #[test]
    fn world_replaces_burst_cell_with_cloud() {
        const LAYER_PARAMS: LayerParameters = LayerParameters {
            minimum_intact_thickness: Fraction::unchecked(0.5),
            ..LayerParameters::DEFAULT
        };

        let mut world = World::new(Position::ORIGIN, Position::ORIGIN).with_cell(Cell::new(
            Position::new(3.5, -1.5),
            Velocity::ZERO,
            vec![
                simple_cell_layer(Area::new(4.0 * PI), Density::new(1.0)),
                simple_cell_layer(Area::new(0.1), Density::new(1.0))
                    .with_parameters(&LAYER_PARAMS)
                    .dead(),
            ],
        ));

        world.tick();

        assert_eq!(world.clouds().len(), 1);
        let cloud = &world.clouds()[0];
        assert_eq!(cloud.center(), Position::new(3.5, -1.5));
        assert_eq!((cloud.radius() * 10.0).value().round(), 20.0);
    }

    #[test]
    fn tick_resizes_cloud() {
        let parameters = Parameters {
            cloud_params: CloudParameters {
                resize_factor: Positive::new(1.5),
                ..CloudParameters::DEFAULT
            },
            ..Parameters::DEFAULT
        };
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_parameters(parameters)
            .with_clouds(vec![Cloud::new(Position::ORIGIN, Length::new(1.0))]);

        world.tick();

        let cloud = &world.clouds()[0];
        assert_eq!(cloud.radius(), Length::new(1.5));
    }

    #[test]
    fn world_removes_nonexistent_clouds() {
        let parameters = Parameters {
            cloud_params: CloudParameters {
                resize_factor: Positive::new(10.0),
                minimum_concentration: Fraction::new(0.1),
            },
            ..Parameters::DEFAULT
        };
        let mut world = World::new(Position::ORIGIN, Position::ORIGIN)
            .with_parameters(parameters)
            .with_clouds(vec![Cloud::new(Position::ORIGIN, Length::new(1.0))]);

        world.tick();

        assert_eq!(world.clouds().len(), 0);
    }

    fn simple_layered_cell(layers: Vec<CellLayer>) -> Cell {
        Cell::new(Position::ORIGIN, Velocity::ZERO, layers)
    }

    fn simple_cell_layer(area: Area, density: Density) -> CellLayer {
        CellLayer::new(
            area,
            density,
            Tissue::Photosynthetic,
            Box::new(NullCellLayerSpecialty::new()),
        )
    }
}
