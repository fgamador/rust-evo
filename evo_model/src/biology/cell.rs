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
        let (expense, income) = Self::summarize_request_costs(costed_requests);
        let _budgeted_fraction = ((start_energy + income).value() / expense.value()).min(1.0);
        (start_energy + expense, vec! {})
    }

    fn summarize_request_costs(costed_requests: &[CostedControlRequest]) -> (BioEnergyDelta, BioEnergyDelta) {
        costed_requests.iter()
            .fold((BioEnergyDelta::new(0.0), BioEnergyDelta::new(0.0)),
                  |(expense, income), request| {
                      let energy_delta = request.energy_delta;
                      if energy_delta.value() < 0.0 {
                          (expense + energy_delta, income)
                      } else {
                          (expense, income + energy_delta)
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
        let _costed_requests = self.cost_control_requests(&control_requests);
        let (_end_energy, _budged_control_requests) =
            Cell::budget_control_requests(self.energy, &_costed_requests);

        for request in control_requests {
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
    fn budgeting_deducts_request_cost() {
        let dummy_control_request = ControlRequest::new(0, 0, 0.0);
        let costed_requests = vec![
            CostedControlRequest::new(dummy_control_request, BioEnergyDelta::new(-1.0)),
            CostedControlRequest::new(dummy_control_request, BioEnergyDelta::new(-1.5)),
        ];

        let (end_energy, _) =
            Cell::budget_control_requests(BioEnergy::new(3.0), &costed_requests);

        assert_eq!(BioEnergy::new(0.5), end_energy);
    }
}
