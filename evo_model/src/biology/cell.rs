use TickCallbacks;
use biology::control::*;
use biology::control_requests::*;
use biology::layers::*;
use environment::environment::*;
use physics::newtonian::*;
use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;
use std::ptr;

#[derive(Debug, GraphNode, HasLocalEnvironment, NewtonianBody)]
pub struct Cell {
    graph_node_data: GraphNodeData,
    radius: Length,
    newtonian_state: NewtonianState,
    environment: LocalEnvironment,
    layers: Vec<Box<CellLayer>>,
    control: Box<CellControl>,
    energy: BioEnergy,
}

impl Cell {
    pub fn new(position: Position, velocity: Velocity, mut layers: Vec<Box<CellLayer>>) -> Self {
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

    pub fn with_control(mut self, control: Box<CellControl>) -> Self {
        self.control = control;
        self
    }

    pub fn with_initial_energy(mut self, energy: BioEnergy) -> Self {
        self.energy = energy;
        self
    }

    pub fn energy(&self) -> BioEnergy {
        self.energy
    }

    fn update_layer_outer_radii(layers: &mut Vec<Box<CellLayer>>) -> Length {
        layers.iter_mut().fold(
            Length::new(0.0),
            |inner_radius, layer| {
                layer.update_outer_radius(inner_radius);
                layer.outer_radius()
            })
    }

    fn calc_mass(layers: &Vec<Box<CellLayer>>) -> Mass {
        layers.iter().fold(
            Mass::new(0.0), |mass, layer| mass + layer.mass())
    }

    fn get_state_snapshot(&self) -> CellStateSnapshot {
        CellStateSnapshot {
            center: self.center(),
            velocity: self.velocity(),
            layers: self.layers.iter()
                .map(|layer| {
                    CellLayerStateSnapshot { area: layer.area() }
                })
                .collect(),
        }
    }

    fn cost_control_requests(&mut self, control_requests: &Vec<ControlRequest>) -> Vec<CostedControlRequest> {
        control_requests.iter()
            .map(|req| self.layers[req.layer_index].cost_control_request(*req))
            .collect()
    }

    fn budget_control_requests(start_energy: BioEnergy, costed_requests: &[CostedControlRequest])
                               -> (BioEnergy, Vec<BudgetedControlRequest>) {
        let (income, expense) = Self::summarize_request_energy_deltas(costed_requests);
        let available_energy = start_energy + income;
        let budgeted_fraction = (available_energy.value() / expense.value()).min(1.0);
        let adjusted_expense = (expense * budgeted_fraction).min(available_energy);
        let end_energy = available_energy - adjusted_expense;
        let budgeted_requests = costed_requests.iter()
            .map(|costed_request| {
                let request_budgeted_fraction = if costed_request.energy_delta.value() < 0.0 { budgeted_fraction } else { 1.0 };
                BudgetedControlRequest::new(*costed_request, request_budgeted_fraction)
            })
            .collect();
        (end_energy, budgeted_requests)
    }

    fn summarize_request_energy_deltas(costed_requests: &[CostedControlRequest]) -> (BioEnergy, BioEnergy) {
        costed_requests.iter()
            .fold((BioEnergy::new(0.0), BioEnergy::new(0.0)),
                  |(income, expense), request| {
                      let energy_delta = request.energy_delta;
                      if energy_delta.value() > 0.0 {
                          (income + energy_delta, expense)
                      } else {
                          (income, expense - energy_delta)
                      }
                  })
    }
}

impl TickCallbacks for Cell {
    fn after_influences(&mut self, _subtick_duration: Duration) {
        let forces = &mut self.newtonian_state.forces_mut();
        for layer in &mut self.layers {
            let (energy, force) = layer.after_influences(&self.environment);
            self.energy += energy;
            forces.add_force(force);
        }
    }

    fn after_movement(&mut self) {
        let cell_state = self.get_state_snapshot();

        let control_requests = self.control.get_control_requests(&cell_state);
        let costed_requests = self.cost_control_requests(&control_requests);
        let (end_energy, budgeted_control_requests) =
            Cell::budget_control_requests(self.energy, &costed_requests);
        self.energy = end_energy;

        for request in budgeted_control_requests {
            self.layers[request.layer_index].execute_control_request(request);
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

impl Onion for Cell {
    type Layer = CellLayer;

    fn layers(&self) -> &[Box<Self::Layer>] {
        &self.layers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evo_view_model::Color;
    use std::f64;
    use std::f64::consts::PI;

    #[test]
    fn cells_use_pointer_equality() {
        let cell1 = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                              vec![
                                  Box::new(SimpleCellLayer::new(
                                      Area::new(PI), Density::new(1.0), Color::Green))
                              ]);
        let cell2 = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                              vec![
                                  Box::new(SimpleCellLayer::new(
                                      Area::new(PI), Density::new(1.0), Color::Green))
                              ]);
        assert_eq!(cell1, cell1);
        assert_ne!(cell1, cell2);
    }

    #[test]
    #[should_panic]
    fn cell_must_have_layers() {
        Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0), vec![]);
    }

    #[test]
    fn cell_has_radius_of_outer_layer() {
        let cell = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                             vec![
                                 Box::new(SimpleCellLayer::new(
                                     Area::new(PI), Density::new(1.0), Color::Green)),
                                 Box::new(SimpleCellLayer::new(
                                     Area::new(3.0 * PI), Density::new(1.0), Color::Green))
                             ]);
        assert_eq!(Length::new(2.0), cell.radius());
    }

    #[test]
    fn cell_has_mass_of_all_layers() {
        let cell = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                             vec![
                                 Box::new(SimpleCellLayer::new(
                                     Area::new(PI), Density::new(1.0), Color::Green)),
                                 Box::new(SimpleCellLayer::new(
                                     Area::new(2.0 * PI), Density::new(2.0), Color::Green))
                             ]);
        assert_eq!(Mass::new(5.0 * PI), cell.mass());
    }

    #[test]
    fn cell_with_continuous_growth_control_grows_on_first_tick() {
        let mut cell = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                                 vec![
                                     Box::new(SimpleCellLayer::new(
                                         Area::new(10.0), Density::new(1.0), Color::Green)),
                                 ])
            .with_control(Box::new(ContinuousGrowthControl::new(0, Area::new(0.5))));
        cell.after_movement();
        assert_eq!(Mass::new(10.5), cell.mass());
    }

    #[test]
    fn layer_growth_cost_reduces_cell_energy() {
        let mut cell = Cell::new(Position::ORIGIN, Velocity::ZERO,
                                 vec![
                                     Box::new(SimpleCellLayer::new(
                                         Area::new(1.0), Density::new(1.0), Color::Green)
                                         .with_resize_parameters(LayerResizeParameters {
                                             growth_energy_delta: BioEnergyDelta::new(-1.0),
                                             max_growth_rate: f64::INFINITY,
                                             shrinkage_energy_delta: BioEnergyDelta::ZERO,
                                             max_shrinkage_rate: f64::INFINITY,
                                         })),
                                 ])
            .with_control(Box::new(ContinuousGrowthControl::new(0, Area::new(2.0))))
            .with_initial_energy(BioEnergy::new(10.0));

        cell.after_movement();

        assert_eq!(BioEnergy::new(8.0), cell.energy());
    }

    #[test]
    fn thruster_layer_adds_force_to_cell() {
        let mut cell = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                                 vec![
                                     Box::new(ThrusterLayer::new(Area::new(1.0))),
                                 ])
            .with_control(Box::new(SimpleThrusterControl::new(0, Force::new(1.0, -1.0))));
        cell.after_movement();
        cell.after_influences(Duration::new(1.0));
        assert_eq!(Force::new(1.0, -1.0), cell.forces().net_force());
    }

    #[test]
    fn photo_layer_adds_energy_to_cell() {
        let mut cell = Cell::new(Position::new(1.0, 1.0), Velocity::new(1.0, 1.0),
                                 vec![
                                     Box::new(PhotoLayer::new(Area::new(4.0), 0.5)),
                                 ]);
        cell.environment_mut().add_light_intensity(10.0);

        cell.after_influences(Duration::new(1.0));

        assert_eq!(BioEnergy::new(20.0), cell.energy());
    }

    #[test]
    fn budgeting_permits_full_request_expenses_if_there_is_enough_energy() {
        let dummy_control_request = ControlRequest::new(0, 0, 0.0);
        let costed_requests = vec![
            CostedControlRequest::new(dummy_control_request, BioEnergyDelta::new(-1.5)),
            CostedControlRequest::new(dummy_control_request, BioEnergyDelta::new(1.0)),
        ];

        let result = Cell::budget_control_requests(BioEnergy::new(3.0), &costed_requests);

        assert_eq!(result, (BioEnergy::new(2.5), vec![
            BudgetedControlRequest::new(costed_requests[0], 1.0),
            BudgetedControlRequest::new(costed_requests[1], 1.0),
        ]));
    }

    #[test]
    fn budgeting_scales_request_expenses_if_there_is_not_enough_energy() {
        let dummy_control_request = ControlRequest::new(0, 0, 0.0);
        let costed_requests = vec![
            CostedControlRequest::new(dummy_control_request, BioEnergyDelta::new(-6.0)),
            CostedControlRequest::new(dummy_control_request, BioEnergyDelta::new(1.0)),
        ];

        let result = Cell::budget_control_requests(BioEnergy::new(2.0), &costed_requests);

        assert_eq!(result, (BioEnergy::new(0.0), vec![
            BudgetedControlRequest::new(costed_requests[0], 0.5),
            BudgetedControlRequest::new(costed_requests[1], 1.0),
        ]));
    }
}
