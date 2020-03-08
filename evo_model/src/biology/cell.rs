use crate::biology::control::*;
use crate::biology::control_requests::*;
use crate::biology::layers::*;
use crate::environment::local_environment::*;
use crate::physics::newtonian::*;
use crate::physics::quantities::*;
use crate::physics::shapes::*;
use crate::physics::sortable_graph::*;
use std::f64::consts::PI;
use std::ptr;

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
            energy: BioEnergy::new(0.0),
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
                Box::new(NullCellLayerSpecialty::new()),
            )],
        )
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
        }
    }

    pub fn layers(&self) -> &[CellLayer] {
        &self.layers
    }

    pub fn energy(&self) -> BioEnergy {
        self.energy
    }

    pub fn is_alive(&self) -> bool {
        self.layers.iter().any(|layer| layer.is_alive())
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

    pub fn after_influences(&mut self, subtick_duration: Duration) {
        let forces = self.newtonian_state.forces_mut();
        for layer in &mut self.layers {
            let (energy, force) = layer.after_influences(&self.environment, subtick_duration);
            self.energy += energy;
            forces.add_force(force);
        }
    }

    pub fn run_control(&mut self) -> Vec<Cell> {
        let (end_energy, budgeted_control_requests) = self.get_budgeted_control_requests();
        self.energy = end_energy;
        self.execute_control_requests(&budgeted_control_requests);
        self.after_control_requests()
    }

    fn get_budgeted_control_requests(&mut self) -> (BioEnergy, Vec<BudgetedControlRequest>) {
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
    ) -> (BioEnergy, Vec<BudgetedControlRequest>) {
        let (income, expense) = Self::summarize_request_energy_deltas(costed_requests);
        let available_energy = start_energy + income;
        let budgeted_fraction = (available_energy.value() / expense.value()).min(1.0);
        let adjusted_expense = (expense * budgeted_fraction).min(available_energy);
        let end_energy = available_energy - adjusted_expense;
        let budgeted_requests = costed_requests
            .iter()
            .map(|costed_request| {
                let request_budgeted_fraction = if costed_request.energy_delta.value() < 0.0 {
                    budgeted_fraction
                } else {
                    1.0
                };
                BudgetedControlRequest::new(*costed_request, request_budgeted_fraction)
            })
            .collect();
        (end_energy, budgeted_requests)
    }

    fn summarize_request_energy_deltas(
        costed_requests: &[CostedControlRequest],
    ) -> (BioEnergy, BioEnergy) {
        costed_requests.iter().fold(
            (BioEnergy::new(0.0), BioEnergy::new(0.0)),
            |(income, expense), request| {
                let energy_delta = request.energy_delta;
                if energy_delta.value() > 0.0 {
                    (income + energy_delta, expense)
                } else {
                    (income, expense - energy_delta)
                }
            },
        )
    }

    fn execute_control_requests(&mut self, budgeted_control_requests: &[BudgetedControlRequest]) {
        // TODO do healing first
        for request in budgeted_control_requests {
            let layer = &mut self.layers[request.layer_index()];
            layer.execute_control_request(*request);
        }
        self.radius = Self::update_layer_outer_radii(&mut self.layers);
        self.newtonian_state.mass = Self::calc_mass(&self.layers);
    }

    fn after_control_requests(&mut self) -> Vec<Cell> {
        // TODO test: inner layer grows while outer layer buds at correct distance
        let mut spawning_requests = vec![];
        for layer in &mut self.layers {
            spawning_requests.push(layer.after_control_requests());
        }

        let mut children = vec![];
        for spawning_request in spawning_requests {
            if spawning_request.donation_energy != BioEnergy::ZERO {
                let child = self.create_and_place_child_cell(
                    spawning_request.budding_angle,
                    spawning_request.donation_energy,
                );
                children.push(child);
            }
        }
        children
    }

    fn create_and_place_child_cell(
        &mut self,
        budding_angle: Angle,
        donation_energy: BioEnergy,
    ) -> Cell {
        let mut child = self.spawn(Area::new(10.0 * PI));
        let offset = Displacement::from_polar(self.radius + child.radius(), budding_angle);
        child.set_initial_position(self.center() + offset);
        child.set_initial_velocity(self.velocity());
        child.set_initial_energy(donation_energy);
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
    fn cell_with_continuous_growth_control_grows_on_first_tick() {
        let mut cell =
            simple_layered_cell(vec![simple_cell_layer(Area::new(10.0), Density::new(1.0))])
                .with_control(Box::new(ContinuousResizeControl::new(
                    0,
                    AreaDelta::new(0.5),
                )));
        cell.run_control();
        assert_eq!(Mass::new(10.5), cell.mass());
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

        cell.run_control();

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
        cell.run_control();
        cell.after_influences(Duration::new(1.0));
        assert_eq!(Force::new(1.0, -1.0), cell.forces().net_force());
    }

    #[test]
    fn photo_layer_adds_energy_to_cell() {
        let mut cell = simple_layered_cell(vec![CellLayer::new(
            Area::new(4.0),
            Density::new(1.0),
            Color::Green,
            Box::new(PhotoCellLayerSpecialty::new(0.5)),
        )]);
        cell.environment_mut().add_light_intensity(10.0);

        cell.after_influences(Duration::new(1.0));

        assert_eq!(BioEnergy::new(20.0), cell.energy());
    }

    #[test]
    fn zero_cost_request_gets_fully_budgeted() {
        let costed_request =
            CostedControlRequest::new(ControlRequest::ZEROS, BioEnergyDelta::new(0.0));

        let (_, budgeted_requests) =
            Cell::budget_control_requests(BioEnergy::new(0.0), &vec![costed_request]);

        assert_eq!(budgeted_requests[0].budgeted_fraction, 1.0);
    }

    #[test]
    fn energy_yielding_request_gets_fully_budgeted() {
        let costed_request =
            CostedControlRequest::new(ControlRequest::ZEROS, BioEnergyDelta::new(1.0));

        let (_, budgeted_requests) =
            Cell::budget_control_requests(BioEnergy::new(0.0), &vec![costed_request]);

        assert_eq!(budgeted_requests[0].budgeted_fraction, 1.0);
    }

    #[test]
    fn request_gets_fully_budgeted_if_cell_has_enough_energy() {
        let costed_request =
            CostedControlRequest::new(ControlRequest::ZEROS, BioEnergyDelta::new(-1.0));

        let (_, budgeted_requests) =
            Cell::budget_control_requests(BioEnergy::new(1.0), &vec![costed_request]);

        assert_eq!(budgeted_requests[0].budgeted_fraction, 1.0);
    }

    #[test]
    fn request_budget_gets_scaled_if_cell_does_not_have_enough_energy() {
        let costed_request =
            CostedControlRequest::new(ControlRequest::ZEROS, BioEnergyDelta::new(-2.0));

        let (_, budgeted_requests) =
            Cell::budget_control_requests(BioEnergy::new(1.0), &vec![costed_request]);

        assert_eq!(budgeted_requests[0].budgeted_fraction, 0.5);
    }

    #[test]
    fn budgeting_returns_remaining_energy() {
        let costed_request =
            CostedControlRequest::new(ControlRequest::ZEROS, BioEnergyDelta::new(-1.0));

        let (energy, _) = Cell::budget_control_requests(BioEnergy::new(2.0), &vec![costed_request]);

        assert_eq!(energy, BioEnergy::new(1.0));
    }

    #[test]
    fn energy_yielding_request_offsets_cost_of_other_request() {
        let costed_requests = vec![
            CostedControlRequest::new(ControlRequest::ZEROS, BioEnergyDelta::new(1.0)),
            CostedControlRequest::new(ControlRequest::ZEROS, BioEnergyDelta::new(-1.0)),
        ];

        let (_, budgeted_requests) =
            Cell::budget_control_requests(BioEnergy::new(0.0), &costed_requests);

        assert_eq!(
            budgeted_requests,
            vec![
                BudgetedControlRequest::new(costed_requests[0], 1.0),
                BudgetedControlRequest::new(costed_requests[1], 1.0),
            ]
        );
    }

    #[test]
    fn energy_yielding_request_offsets_cost_of_other_request_with_scaling() {
        let costed_requests = vec![
            CostedControlRequest::new(ControlRequest::ZEROS, BioEnergyDelta::new(1.0)),
            CostedControlRequest::new(ControlRequest::ZEROS, BioEnergyDelta::new(-2.0)),
        ];

        let (_, budgeted_requests) =
            Cell::budget_control_requests(BioEnergy::new(0.0), &costed_requests);

        assert_eq!(
            budgeted_requests,
            vec![
                BudgetedControlRequest::new(costed_requests[0], 1.0),
                BudgetedControlRequest::new(costed_requests[1], 0.5),
            ]
        );
    }

    #[test]
    fn overlap_damages_all_layers() {
        const LAYER0_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
            overlap_damage_health_delta: -1.0,
            ..LayerHealthParameters::DEFAULT
        };
        const LAYER1_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
            overlap_damage_health_delta: -1.0,
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
        cell.after_influences(Duration::new(1.0));

        assert!(cell.layers()[0].health() < 1.0);
        assert!(cell.layers()[1].health() < 1.0);
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

        cell.run_control();

        assert_eq!(5.0, cell.layers()[0].area().value());
        assert_eq!(10.0, cell.layers()[1].area().value());
        assert_eq!(BioEnergy::new(5.0), cell.energy());
    }

    #[test]
    fn budding_creates_child_with_right_state() {
        let mut cell = Cell::new(
            Position::new(2.0, -2.0),
            Velocity::new(3.0, -3.0),
            vec![
                simple_cell_layer(Area::new(10.0), Density::new(1.0)),
                CellLayer::new(
                    Area::new(5.0),
                    Density::new(1.0),
                    Color::White,
                    Box::new(BuddingCellLayerSpecialty::new()),
                ),
            ],
        )
        .with_control(Box::new(ContinuousRequestsControl::new(vec![
            CellLayer::resize_request(0, AreaDelta::new(10.0)),
            BuddingCellLayerSpecialty::budding_angle_request(1, Angle::from_radians(0.0)),
            BuddingCellLayerSpecialty::donation_energy_request(1, BioEnergy::new(1.0)),
        ])));

        let children = cell.run_control();

        assert_eq!(children.len(), 1);
        let child = &children[0];
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
    fn budding_does_not_create_child_if_given_zero_energy() {
        let mut cell = Cell::new(
            Position::ORIGIN,
            Velocity::ZERO,
            vec![CellLayer::new(
                Area::new(1.0),
                Density::new(1.0),
                Color::White,
                Box::new(BuddingCellLayerSpecialty::new()),
            )],
        )
        .with_control(Box::new(ContinuousRequestsControl::new(vec![
            BuddingCellLayerSpecialty::donation_energy_request(0, BioEnergy::new(0.0)),
        ])));

        let children = cell.run_control();

        assert!(children.is_empty());
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
