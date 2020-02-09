extern crate evo_main;
extern crate evo_model;

use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::biology::control::*;
use evo_model::biology::control_requests::*;
use evo_model::biology::layers::*;
use evo_model::genome::sparse_neural_net::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_cell(
            create_child(0, &MutationParameters::NO_MUTATION)
                .with_initial_position(Position::new(200.0, -100.0)),
        )
}

fn create_child(seed: u64, mutation_parameters: &'static MutationParameters) -> Cell {
    Cell::new(
        Position::ORIGIN,
        Velocity::ZERO,
        vec![
            CellLayer::new(
                Area::new(5.0 * PI),
                Density::new(1.0),
                Color::Green,
                Box::new(EnergyGeneratingCellLayerSpecialty::new()),
            ),
            CellLayer::new(
                Area::new(5.0 * PI),
                Density::new(1.0),
                Color::Yellow,
                Box::new(BuddingCellLayerSpecialty::new(
                    seed,
                    mutation_parameters,
                    create_child,
                )),
            ),
        ],
    )
    .with_control(Box::new(NeuralNetBuddingControl::new()))
}

#[derive(Debug)]
pub struct NeuralNetBuddingControl {
    nnet: SparseNeuralNet,
    budding_ticks: u32,
    tick: u32,
}

impl NeuralNetBuddingControl {
    fn new() -> Self {
        let mut nnet = SparseNeuralNet::new(TransferFn::IDENTITY);
        nnet.connect_node(1, -100.0, vec![(0, -1.0)]);
        NeuralNetBuddingControl {
            nnet,
            budding_ticks: 100,
            tick: 0,
        }
    }

    fn is_adult(cell_state: &CellStateSnapshot) -> bool {
        cell_state.area >= Area::new(1000.0)
    }

    fn adult_requests(&mut self) -> Vec<ControlRequest> {
        let mut requests = vec![EnergyGeneratingCellLayerSpecialty::energy_request(
            0,
            BioEnergy::new(1.0),
        )];
        requests.append(&mut self.budding_requests());
        requests
    }

    fn budding_requests(&mut self) -> Vec<ControlRequest> {
        self.tick += 1;
        if self.tick < self.budding_ticks {
            return vec![];
        }

        self.tick = 0;
        vec![
            BuddingCellLayerSpecialty::donation_energy_request(1, BioEnergy::new(1.0)),
        ]
    }

    fn youth_requests(&self) -> Vec<ControlRequest> {
        vec![
            CellLayer::resize_request(0, AreaDelta::new(5.0)),
            CellLayer::resize_request(1, AreaDelta::new(5.0)),
        ]
    }
}

impl CellControl for NeuralNetBuddingControl {
    //    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
    //        self.nnet.set_node_value(0, cell_state.center.y() as f32);
    //        self.nnet.run();
    //        vec![CellLayer::resize_request(
    //            0,
    //            AreaDelta::new(self.nnet.node_value(1) as f64),
    //        )]
    //    }

    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        if Self::is_adult(cell_state) {
            self.adult_requests()
        } else {
            self.youth_requests()
        }
    }
}
