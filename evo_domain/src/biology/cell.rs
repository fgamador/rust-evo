use crate::biology::changes::*;
use crate::biology::control::*;
use crate::biology::control_requests::*;
use crate::biology::layers::*;
use crate::environment::local_environment::*;
use crate::physics::newtonian::*;
use crate::physics::quantities::*;
use crate::physics::shapes::*;
use crate::physics::sortable_graph::*;
use evo_domain_derive::*;
use std::f64::consts::PI;
use std::ptr;
use std::usize;

#[allow(clippy::vec_box)]
#[derive(Debug, GraphNode, HasLocalEnvironment, NewtonianBody)]
pub struct Cell {
    graph_node_data: GraphNodeData,
    radius: Length,
    newtonian_state: NewtonianState,
    environment: LocalEnvironment,
    layers: Vec<CellLayer>, // TODO array? smallvec?
    control: Box<dyn CellControl>,
    energy: BioEnergy,
    received_donated_energy: BioEnergy,
    thrust: Force,
    selected: bool,
}

impl Cell {
    pub fn new(position: Position, velocity: Velocity, mut layers: Vec<CellLayer>) -> Self {
        if layers.is_empty() {
            panic!("Cell must have at least one layer");
        }

        let radius = Self::update_layer_outer_radii(&mut layers);
        Cell {
            graph_node_data: GraphNodeData::new(),
            radius,
            newtonian_state: NewtonianState::new(Self::calc_mass(&layers), position, velocity),
            environment: LocalEnvironment::new(),
            layers,
            control: Box::new(NullControl::new()),
            energy: BioEnergy::ZERO,
            received_donated_energy: BioEnergy::ZERO,
            thrust: Force::ZERO,
            selected: false,
        }
    }

    pub fn ball(radius: Length, mass: Mass, position: Position, velocity: Velocity) -> Self {
        let area = PI * radius.sqr();
        Self::new(
            position,
            velocity,
            vec![CellLayer::new(
                area,
                mass / area,
                Color::Green,
                Box::new(BondingCellLayerSpecialty::new()),
            )],
        )
        .with_control(Box::new(ContinuousRequestsControl::new(vec![
            BondingCellLayerSpecialty::retain_bond_request(0, 0, true),
            BondingCellLayerSpecialty::retain_bond_request(0, 1, true),
        ])))
    }

    pub fn with_control(mut self, control: Box<dyn CellControl>) -> Self {
        self.control = control;
        self
    }

    pub fn with_initial_position(mut self, position: Position) -> Self {
        self.newtonian_state.position = position;
        self
    }

    pub fn with_initial_energy(mut self, energy: BioEnergy) -> Self {
        self.energy = energy;
        self
    }

    pub fn spawn(&mut self, layer_area: Area) -> Self {
        let mut layers = self
            .layers
            .iter()
            .map(|layer| layer.spawn(layer_area))
            .collect();
        let radius = Self::update_layer_outer_radii(&mut layers);
        Cell {
            graph_node_data: GraphNodeData::new(),
            radius,
            newtonian_state: NewtonianState::new(
                Self::calc_mass(&layers),
                Position::ORIGIN,
                Velocity::ZERO,
            ),
            environment: LocalEnvironment::new(),
            layers,
            control: self.control.spawn(),
            energy: BioEnergy::ZERO,
            received_donated_energy: BioEnergy::ZERO,
            thrust: Force::ZERO,
            selected: false,
        }
    }

    pub fn layers(&self) -> &[CellLayer] {
        &self.layers
    }

    pub fn energy(&self) -> BioEnergy {
        self.energy
    }

    pub fn add_received_donated_energy(&mut self, energy: BioEnergy) {
        self.add_energy(energy);
        self.received_donated_energy += energy;
    }

    pub fn add_energy(&mut self, energy: BioEnergy) {
        self.energy += energy;
    }

    pub fn is_alive(&self) -> bool {
        self.layers.iter().any(|layer| layer.is_alive())
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, is_selected: bool) {
        self.selected = is_selected;
        if is_selected {
            self.net_force_mut().start_recording_force_additions();
        } else {
            self.net_force_mut().stop_recording_force_additions();
        }
    }

    pub fn set_initial_position(&mut self, position: Position) {
        self.newtonian_state.position = position;
    }

    pub fn set_initial_velocity(&mut self, velocity: Velocity) {
        self.newtonian_state.velocity = velocity;
    }

    pub fn set_initial_energy(&mut self, energy: BioEnergy) {
        self.energy = energy;
    }

    pub fn overlaps(&self, pos: Position) -> bool {
        (self.position() - pos).length() <= self.radius
    }

    pub fn tick(&mut self) -> BondRequests {
        let start_snapshot = self.get_state_snapshot();
        let mut changes = CellChanges::new(self.layers.len(), self.is_selected());
        self.calculate_automatic_changes(&mut changes);
        self.calculate_requested_changes(&mut changes);
        self.apply_changes(&changes);
        self.print_tick_info(&start_snapshot, &changes);
        self.clear_environment();
        changes.bond_requests
    }

    pub fn calculate_automatic_changes(&mut self, changes: &mut CellChanges) {
        for (index, layer) in self.layers.iter_mut().enumerate() {
            layer.calculate_automatic_changes(&self.environment, changes, index);
        }
        self.newtonian_state
            .net_force_mut()
            .add_non_dominant_force(self.thrust, "thrust");
    }

    pub fn calculate_requested_changes(&mut self, changes: &mut CellChanges) {
        let budgeted_control_requests = self.get_budgeted_control_requests();
        self.execute_control_requests(&budgeted_control_requests, changes);
    }

    fn get_budgeted_control_requests(&mut self) -> Vec<BudgetedControlRequest> {
        let cell_state = self.get_state_snapshot();
        let control_requests = self.control.run(&cell_state);
        let costed_requests = self.cost_control_requests(&control_requests);
        Self::budget_control_requests(self.energy, &costed_requests)
    }

    fn get_state_snapshot(&self) -> CellStateSnapshot {
        CellStateSnapshot {
            radius: self.radius(),
            area: self.area(),
            mass: self.mass(),
            center: self.center(),
            velocity: self.velocity(),
            energy: self.energy(),
            bond_0_exists: self.has_edge(0),
            layers: self.get_layer_state_snapshots(),
        }
    }

    fn get_layer_state_snapshots(&self) -> Vec<CellLayerStateSnapshot> {
        let mut result = Vec::with_capacity(self.layers.len());
        for layer in &self.layers {
            result.push(CellLayerStateSnapshot {
                area: layer.area(),
                mass: layer.mass(),
                health: layer.health(),
            });
        }
        result
    }

    fn cost_control_requests(
        &mut self,
        control_requests: &[ControlRequest],
    ) -> Vec<CostedControlRequest> {
        control_requests
            .iter()
            .map(|req| self.layers[req.layer_index()].cost_control_request(*req))
            .collect()
    }

    fn budget_control_requests(
        start_energy: BioEnergy,
        costed_requests: &[CostedControlRequest],
    ) -> Vec<BudgetedControlRequest> {
        let (income, expense) = Self::summarize_request_energy_deltas(costed_requests);
        let available_energy = start_energy + income;
        let budgeted_fraction =
            Fraction::new((available_energy.value() / expense.value()).min(1.0));
        costed_requests
            .iter()
            .map(|costed_request| {
                let request_budgeted_fraction = if costed_request.energy_delta().value() < 0.0 {
                    budgeted_fraction
                } else {
                    Fraction::ONE
                };
                BudgetedControlRequest::new(*costed_request, request_budgeted_fraction)
            })
            .collect()
    }

    fn summarize_request_energy_deltas(
        costed_requests: &[CostedControlRequest],
    ) -> (BioEnergy, BioEnergy) {
        costed_requests.iter().fold(
            (BioEnergy::ZERO, BioEnergy::ZERO),
            |(income, expense), request| {
                let energy_delta = request.energy_delta();
                if energy_delta.value() > 0.0 {
                    (income + energy_delta, expense)
                } else {
                    (income, expense - energy_delta)
                }
            },
        )
    }

    fn execute_control_requests(
        &mut self,
        budgeted_control_requests: &[BudgetedControlRequest],
        changes: &mut CellChanges,
    ) {
        for request in budgeted_control_requests {
            let layer = &mut self.layers[request.layer_index()];
            layer.execute_control_request(request, changes);
        }
    }

    fn move_from_forces(&mut self) {
        self.exert_net_force_for_one_tick();
        self.move_for_one_tick();
    }

    fn clear_environment(&mut self) {
        self.environment_mut().clear();
        self.net_force_mut().clear();
        self.received_donated_energy = BioEnergy::ZERO;
    }

    fn print_tick_info(&self, start_snapshot: &CellStateSnapshot, changes: &CellChanges) {
        if self.is_selected() {
            self.print_id_info();
            self.print_force_info();
            self.print_other_quantities_info(start_snapshot);
            self.print_energy_info(start_snapshot, changes);
            self.print_layers_info(start_snapshot, changes);
            Cell::print_bond_request_info(changes)
        }
    }

    fn print_id_info(&self) {
        println!(
            "Cell {}{}:",
            self.node_handle(),
            if self.is_alive() { "" } else { " (DEAD)" }
        );
    }

    fn print_force_info(&self) {
        println!("  net force {}", self.net_force().net_force());
        if self.net_force().dominant_x_force() != 0.0 {
            println!(
                "    {} x {:.4}",
                self.net_force().dominant_x_force_label(),
                self.net_force().dominant_x_force(),
            );
        }
        if self.net_force().dominant_y_force() != 0.0 {
            println!(
                "    {} y {:.4}",
                self.net_force().dominant_y_force_label(),
                self.net_force().dominant_y_force(),
            );
        }
        if let Some(force_additions) = &self.net_force().non_dominant_force_additions() {
            for force_addition in force_additions {
                if force_addition.force != Force::ZERO {
                    println!("    {} {:+.4}", force_addition.label, force_addition.force,);
                }
            }
        }
    }

    fn print_other_quantities_info(&self, start_snapshot: &CellStateSnapshot) {
        println_value2d_change_info(
            "  position",
            start_snapshot.center.value(),
            self.position().value(),
        );
        println_value2d_change_info(
            "  velocity",
            start_snapshot.velocity.value(),
            self.velocity().value(),
        );
        println_value1d_change_info("  mass", start_snapshot.mass.value(), self.mass().value());
        println_value1d_change_info(
            "  radius",
            start_snapshot.radius.value(),
            self.radius().value(),
        );
    }

    fn print_energy_info(&self, start_snapshot: &CellStateSnapshot, changes: &CellChanges) {
        println_value1d_change_info(
            "  energy",
            start_snapshot.energy.value(),
            self.energy().value(),
        );
        println!("    received {:+.4}", self.received_donated_energy.value());
        if let Some(energy_changes) = &changes.energy_changes {
            for energy_change in energy_changes {
                println!(
                    "    {}{} {:+.4}",
                    energy_change.label,
                    if energy_change.index < usize::MAX {
                        format!("[{}]", energy_change.index)
                    } else {
                        "".to_string()
                    },
                    energy_change.energy_delta.value()
                );
            }
        }
    }

    fn print_layers_info(&self, start_snapshot: &CellStateSnapshot, changes: &CellChanges) {
        for (index, layer) in self.layers.iter().enumerate() {
            layer.print_tick_info(index, &start_snapshot.layers[index], &changes.layers[index]);
        }
    }

    fn print_bond_request_info(changes: &CellChanges) {
        for (index, request) in changes.bond_requests.iter().enumerate() {
            if request.retain_bond {
                println!("  bond request {}: {}", index, request);
            }
        }
    }

    pub fn create_and_place_child_cell(
        &mut self,
        budding_angle: Angle,
        initial_energy: BioEnergy,
    ) -> Cell {
        let mut child = self.spawn(Area::new(10.0 * PI));
        let offset = Displacement::from_polar(self.radius + child.radius(), budding_angle);
        child.set_initial_position(self.center() + offset);
        child.set_initial_velocity(self.velocity());
        child.set_initial_energy(initial_energy);
        child
    }

    #[allow(clippy::vec_box)]
    fn update_layer_outer_radii(layers: &mut Vec<CellLayer>) -> Length {
        layers
            .iter_mut()
            .fold(Length::new(0.0), |inner_radius, layer| {
                layer.update_outer_radius(inner_radius);
                layer.outer_radius()
            })
    }

    fn calc_mass(layers: &[CellLayer]) -> Mass {
        layers
            .iter()
            .fold(Mass::new(0.0), |mass, layer| mass + layer.mass())
    }

    pub fn apply_changes(&mut self, changes: &CellChanges) {
        self.move_from_forces();
        self.energy += changes.energy;
        self.thrust = changes.thrust;
        for (index, layer) in self.layers.iter_mut().enumerate() {
            layer.apply_changes(&changes.layers[index]);
        }
        self.radius = Self::update_layer_outer_radii(&mut self.layers);
        self.newtonian_state.mass = Self::calc_mass(&self.layers);
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Circle for Cell {
    fn radius(&self) -> Length {
        self.radius
    }

    fn center(&self) -> Position {
        self.newtonian_state.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::overlap::Overlap;

    #[test]
    fn cells_use_pointer_equality() {
        let cell1 = simple_layered_cell(vec![simple_cell_layer(Area::new(PI), Density::new(1.0))]);
        let cell2 = simple_layered_cell(vec![simple_cell_layer(Area::new(PI), Density::new(1.0))]);
        assert_eq!(cell1, cell1);
        assert_ne!(cell1, cell2);
    }

    #[test]
    #[should_panic]
    fn cell_must_have_layers() {
        simple_layered_cell(vec![]);
    }

    #[test]
    fn cell_has_radius_of_outer_layer() {
        let cell = simple_layered_cell(vec![
            simple_cell_layer(Area::new(PI), Density::new(1.0)),
            simple_cell_layer(Area::new(3.0 * PI), Density::new(1.0)),
        ]);
        assert_eq!(Length::new(2.0), cell.radius());
    }

    #[test]
    fn cell_has_mass_of_all_layers() {
        let cell = simple_layered_cell(vec![
            simple_cell_layer(Area::new(PI), Density::new(1.0)),
            simple_cell_layer(Area::new(2.0 * PI), Density::new(2.0)),
        ]);
        assert_eq!(Mass::new(5.0 * PI), cell.mass());
    }

    #[test]
    fn cell_with_all_dead_layers_is_dead() {
        let cell = simple_layered_cell(vec![
            simple_cell_layer(Area::new(1.0), Density::new(1.0)).dead(),
            simple_cell_layer(Area::new(1.0), Density::new(1.0)).dead(),
        ]);
        assert!(!cell.is_alive());
    }

    #[test]
    fn cell_with_one_live_layer_is_alive() {
        let cell = simple_layered_cell(vec![
            simple_cell_layer(Area::new(1.0), Density::new(1.0)),
            simple_cell_layer(Area::new(1.0), Density::new(1.0)).dead(),
        ]);
        assert!(cell.is_alive());
    }

    #[test]
    fn cell_as_ball() {
        let ball = Cell::ball(
            Length::new(2.0),
            Mass::new(3.0),
            Position::new(1.0, -1.0),
            Velocity::new(-2.0, 3.0),
        );
        assert_eq!(Length::new(2.0), ball.radius());
        assert_eq!(Mass::new(3.0), ball.mass());
        assert_eq!(Position::new(1.0, -1.0), ball.center());
        assert_eq!(Velocity::new(-2.0, 3.0), ball.velocity());
    }

    #[test]
    fn cell_overlaps_position() {
        let cell = Cell::ball(
            Length::new(2.0),
            Mass::ZERO,
            Position::new(10.0, 20.0),
            Velocity::ZERO,
        );
        assert!(cell.overlaps(Position::new(11.0, 19.0)));
        assert!(!cell.overlaps(Position::new(11.9, 18.1)));
    }

    #[test]
    fn applying_cell_changes_updates_cell_radius() {
        let mut cell =
            simple_layered_cell(vec![simple_cell_layer(Area::new(PI), Density::new(1.0))])
                .with_control(Box::new(ContinuousResizeControl::new(
                    0,
                    AreaDelta::new(3.0 * PI),
                )));
        let mut changes = CellChanges::new(cell.layers.len(), false);
        cell.calculate_requested_changes(&mut changes);
        cell.apply_changes(&changes);
        assert_eq!(cell.radius(), Length::new(2.0));
    }

    #[test]
    fn applying_cell_changes_updates_cell_mass() {
        let mut cell =
            simple_layered_cell(vec![simple_cell_layer(Area::new(10.0), Density::new(1.0))])
                .with_control(Box::new(ContinuousResizeControl::new(
                    0,
                    AreaDelta::new(0.5),
                )));
        let mut changes = CellChanges::new(cell.layers.len(), false);
        cell.calculate_requested_changes(&mut changes);
        cell.apply_changes(&changes);
        assert_eq!(cell.mass(), Mass::new(10.5));
    }

    #[test]
    fn cell_force_applies_to_pretick_mass() {
        let mut cell =
            simple_layered_cell(vec![simple_cell_layer(Area::new(1.0), Density::new(1.0))])
                .with_control(Box::new(ContinuousResizeControl::new(
                    0,
                    AreaDelta::new(3.0),
                )));
        cell.net_force_mut()
            .add_non_dominant_force(Force::new(1.0, 0.0), "test");

        cell.tick();

        assert_eq!(cell.velocity(), Velocity::new(1.0, 0.0));
    }

    #[test]
    fn layer_growth_cost_reduces_cell_energy() {
        const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
            growth_energy_delta: BioEnergyDelta::new(-1.0),
            ..LayerResizeParameters::UNLIMITED
        };

        let mut cell =
            simple_layered_cell(vec![simple_cell_layer(Area::new(1.0), Density::new(1.0))
                .with_resize_parameters(&LAYER_RESIZE_PARAMS)])
            .with_control(Box::new(ContinuousResizeControl::new(
                0,
                AreaDelta::new(2.0),
            )))
            .with_initial_energy(BioEnergy::new(10.0));

        let mut changes = CellChanges::new(cell.layers.len(), false);
        cell.calculate_requested_changes(&mut changes);
        cell.apply_changes(&changes);

        assert_eq!(BioEnergy::new(8.0), cell.energy());
    }

    #[test]
    fn thruster_layer_adds_force_to_cell() {
        let mut cell = simple_layered_cell(vec![CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(ThrusterCellLayerSpecialty::new()),
        )])
        .with_control(Box::new(SimpleThrusterControl::new(
            0,
            Force::new(1.0, -1.0),
        )));
        let mut changes = CellChanges::new(cell.layers.len(), false);
        cell.calculate_requested_changes(&mut changes);
        cell.apply_changes(&changes);

        // next tick
        let mut changes2 = CellChanges::new(cell.layers.len(), false);
        cell.calculate_automatic_changes(&mut changes2);
        assert_eq!(Force::new(1.0, -1.0), cell.net_force().net_force());
    }

    #[test]
    fn photo_layer_adds_energy_to_cell() {
        let mut cell = simple_layered_cell(vec![CellLayer::new(
            Area::new(4.0),
            Density::new(1.0),
            Color::Green,
            Box::new(PhotoCellLayerSpecialty::new(Fraction::new(0.5))),
        )]);
        cell.environment_mut().add_light_intensity(10.0);

        let mut changes = CellChanges::new(cell.layers.len(), false);
        cell.calculate_automatic_changes(&mut changes);
        cell.apply_changes(&changes);

        assert_eq!(BioEnergy::new(20.0), cell.energy());
    }

    #[test]
    fn budding_creates_child_with_right_state() {
        let mut cell = Cell::new(
            Position::new(2.0, -2.0),
            Velocity::new(3.0, -3.0),
            vec![simple_cell_layer(Area::new(10.0), Density::new(1.0))],
        );

        let child = cell.create_and_place_child_cell(Angle::from_radians(0.0), BioEnergy::new(1.0));

        assert_eq!(
            child.center(),
            Position::new(
                cell.center().x() + cell.radius().value() + child.radius().value(),
                cell.center().y()
            )
        );
        assert_eq!(child.velocity(), cell.velocity());
        assert_eq!(child.energy(), BioEnergy::new(1.0));
    }

    #[test]
    fn zero_cost_request_gets_fully_budgeted() {
        let costed_request =
            CostedControlRequest::unlimited(ControlRequest::NULL_REQUEST, BioEnergyDelta::new(0.0));

        let budgeted_requests =
            Cell::budget_control_requests(BioEnergy::new(0.0), &vec![costed_request]);

        assert_eq!(budgeted_requests[0].budgeted_fraction(), Fraction::ONE);
    }

    #[test]
    fn energy_yielding_request_gets_fully_budgeted() {
        let costed_request =
            CostedControlRequest::unlimited(ControlRequest::NULL_REQUEST, BioEnergyDelta::new(1.0));

        let budgeted_requests =
            Cell::budget_control_requests(BioEnergy::new(0.0), &vec![costed_request]);

        assert_eq!(budgeted_requests[0].budgeted_fraction(), Fraction::ONE);
    }

    #[test]
    fn request_gets_fully_budgeted_if_cell_has_enough_energy() {
        let costed_request = CostedControlRequest::unlimited(
            ControlRequest::NULL_REQUEST,
            BioEnergyDelta::new(-1.0),
        );

        let budgeted_requests =
            Cell::budget_control_requests(BioEnergy::new(1.0), &vec![costed_request]);

        assert_eq!(budgeted_requests[0].budgeted_fraction(), Fraction::ONE);
    }

    #[test]
    fn request_budget_gets_scaled_if_cell_does_not_have_enough_energy() {
        let costed_request = CostedControlRequest::unlimited(
            ControlRequest::NULL_REQUEST,
            BioEnergyDelta::new(-2.0),
        );

        let budgeted_requests =
            Cell::budget_control_requests(BioEnergy::new(1.0), &vec![costed_request]);

        assert_eq!(budgeted_requests[0].budgeted_fraction(), Fraction::new(0.5));
    }

    #[test]
    fn energy_yielding_request_offsets_cost_of_other_request() {
        let costed_requests = vec![
            CostedControlRequest::unlimited(ControlRequest::NULL_REQUEST, BioEnergyDelta::new(1.0)),
            CostedControlRequest::unlimited(
                ControlRequest::NULL_REQUEST,
                BioEnergyDelta::new(-1.0),
            ),
        ];

        let budgeted_requests =
            Cell::budget_control_requests(BioEnergy::new(0.0), &costed_requests);

        assert_eq!(
            budgeted_requests,
            vec![
                BudgetedControlRequest::new(costed_requests[0], Fraction::ONE),
                BudgetedControlRequest::new(costed_requests[1], Fraction::ONE),
            ]
        );
    }

    #[test]
    fn energy_yielding_request_offsets_cost_of_other_request_with_scaling() {
        let costed_requests = vec![
            CostedControlRequest::unlimited(ControlRequest::NULL_REQUEST, BioEnergyDelta::new(1.0)),
            CostedControlRequest::unlimited(
                ControlRequest::NULL_REQUEST,
                BioEnergyDelta::new(-2.0),
            ),
        ];

        let budgeted_requests =
            Cell::budget_control_requests(BioEnergy::new(0.0), &costed_requests);

        assert_eq!(
            budgeted_requests,
            vec![
                BudgetedControlRequest::new(costed_requests[0], Fraction::ONE),
                BudgetedControlRequest::new(costed_requests[1], Fraction::new(0.5)),
            ]
        );
    }

    #[test]
    fn overlap_damages_all_layers() {
        const LAYER0_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
            overlap_damage_health_delta: HealthDelta::new(-1.0),
            ..LayerHealthParameters::DEFAULT
        };
        const LAYER1_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
            overlap_damage_health_delta: HealthDelta::new(-1.0),
            ..LayerHealthParameters::DEFAULT
        };

        let mut cell = simple_layered_cell(vec![
            simple_cell_layer(Area::new(1.0), Density::new(1.0))
                .with_health_parameters(&LAYER0_HEALTH_PARAMS),
            simple_cell_layer(Area::new(1.0), Density::new(1.0))
                .with_health_parameters(&LAYER1_HEALTH_PARAMS),
        ]);

        cell.environment_mut()
            .add_overlap(Overlap::new(Displacement::new(1.0, 0.0), 1.0));
        let mut changes = CellChanges::new(cell.layers.len(), false);
        cell.calculate_automatic_changes(&mut changes);
        cell.apply_changes(&changes);

        assert!(cell.layers()[0].health() < Health::FULL);
        assert!(cell.layers()[1].health() < Health::FULL);
    }

    #[test]
    fn layer_shrinkage_allows_layer_growth_within_limits() {
        const LAYER0_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
            shrinkage_energy_delta: BioEnergyDelta::new(2.0),
            max_shrinkage_rate: 0.5,
            ..LayerResizeParameters::UNLIMITED
        };
        const LAYER1_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
            growth_energy_delta: BioEnergyDelta::new(-1.0),
            max_growth_rate: 1.0,
            ..LayerResizeParameters::UNLIMITED
        };

        let mut cell = simple_layered_cell(vec![
            simple_cell_layer(Area::new(10.0), Density::new(1.0))
                .with_resize_parameters(&LAYER0_RESIZE_PARAMS),
            simple_cell_layer(Area::new(5.0), Density::new(1.0))
                .with_resize_parameters(&LAYER1_RESIZE_PARAMS),
        ])
        .with_control(Box::new(ContinuousRequestsControl::new(vec![
            CellLayer::resize_request(0, AreaDelta::new(-100.0)),
            CellLayer::resize_request(1, AreaDelta::new(100.0)),
        ])));

        let mut changes = CellChanges::new(cell.layers.len(), false);
        cell.calculate_requested_changes(&mut changes);
        cell.apply_changes(&changes);

        assert_eq!(5.0, cell.layers()[0].area().value());
        assert_eq!(10.0, cell.layers()[1].area().value());
        assert_eq!(BioEnergy::new(5.0), cell.energy());
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
