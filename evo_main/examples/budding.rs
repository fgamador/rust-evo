extern crate evo_main;
extern crate evo_model;

use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::biology::control::*;
use evo_model::biology::control_requests::*;
use evo_model::biology::layers::*;
use evo_model::environment::influences::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_influence(Box::new(PairCollisions::new()))
        .with_cell(create_child().with_initial_position(Position::new(200.0, -100.0)))
}

fn create_child() -> Cell {
    Cell::new(
        Position::ORIGIN,
        Velocity::ZERO,
        vec![
            Box::new(CellLayer::new(
                Area::new(5.0 * PI),
                Density::new(1.0),
                Color::Yellow,
                Box::new(EnergyGeneratingCellLayerSpecialty::new()),
            )),
            Box::new(CellLayer::new(
                Area::new(5.0 * PI),
                Density::new(1.0),
                Color::Green,
                Box::new(BuddingCellLayerSpecialty::new(create_child)),
            )),
        ],
    )
    .with_control(Box::new(BuddingControl::new(1)))
}

#[derive(Debug)]
pub struct BuddingControl {
    budding_layer_index: usize,
    budding_ticks: u32,
    budding_angle: Angle,
    tick: u32,
}

impl BuddingControl {
    fn new(budding_layer_index: usize) -> Self {
        BuddingControl {
            budding_layer_index,
            budding_ticks: 100,
            budding_angle: Angle::from_radians(0.0),
            tick: 0,
        }
    }

    fn is_adult(cell_state: &CellStateSnapshot) -> bool {
        cell_state.area >= Area::new(1000.0)
    }

    fn adult_requests(&mut self) -> Vec<ControlRequest> {
        let mut requests = vec![
            EnergyGeneratingCellLayerSpecialty::energy_request(0, BioEnergy::new(1.0)),
        ];
        requests.append(&mut self.budding_requests());
        requests
    }

    fn budding_requests(&mut self) -> Vec<ControlRequest> {
        self.tick += 1;
        if self.tick == self.budding_ticks {
            self.tick = 0;
            self.budding_angle += Deflection::from_radians(PI / 4.0);
            vec![
                BuddingCellLayerSpecialty::budding_angle_request(self.budding_layer_index, self.budding_angle),
                BuddingCellLayerSpecialty::donation_energy_request(self.budding_layer_index, BioEnergy::new(1.0)),
            ]
        } else {
            vec![
                BuddingCellLayerSpecialty::budding_angle_request(self.budding_layer_index, self.budding_angle),
                BuddingCellLayerSpecialty::donation_energy_request(self.budding_layer_index, BioEnergy::new(0.0)),
            ]
        }
    }

    fn youth_requests(budding_layer_index: usize) -> Vec<ControlRequest> {
        vec![
            CellLayer::resize_request(0, AreaDelta::new(5.0)),
            CellLayer::resize_request(budding_layer_index, AreaDelta::new(5.0)),
        ]
    }
}

impl CellControl for BuddingControl {
    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        if Self::is_adult(cell_state) {
            self.adult_requests()
        } else {
            Self::youth_requests(self.budding_layer_index)
        }
    }
}
