extern crate evo_main;
extern crate evo_model;

use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::biology::control::*;
use evo_model::biology::control_requests::*;
use evo_model::biology::layers::*;
use evo_model::environment::influences::*;
use evo_model::genome::sparse_neural_net::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use std::f64::consts::PI;

type VecIndex = u16;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_sunlight(0.0, 1.0)
        .with_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.005)))),
        ])
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
                Box::new(PhotoCellLayerSpecialty::new(0.1)),
            )
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::new(-1.0),
                max_growth_rate: 10.0,
                shrinkage_energy_delta: BioEnergyDelta::new(0.0),
                max_shrinkage_rate: 0.1,
            }),
            CellLayer::new(
                Area::new(5.0 * PI),
                Density::new(1.0),
                Color::Yellow,
                Box::new(BuddingCellLayerSpecialty::new(
                    seed,
                    mutation_parameters,
                    create_child,
                )),
            )
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::new(-1.0),
                max_growth_rate: 10.0,
                shrinkage_energy_delta: BioEnergyDelta::new(0.0),
                max_shrinkage_rate: 0.1,
            }),
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
    const PHOTO_LAYER_AREA_INPUT_INDEX: VecIndex = 0;
    const CELL_ENERGY_INPUT_INDEX: VecIndex = 1;
    const PHOTO_LAYER_RESIZE_OUTPUT_INDEX: VecIndex = 2;
    const DONATION_ENERGY_OUTPUT_INDEX: VecIndex = 3;

    fn new() -> Self {
        let mut nnet = SparseNeuralNet::new(TransferFn::IDENTITY);
        nnet.connect_node(
            Self::PHOTO_LAYER_RESIZE_OUTPUT_INDEX,
            800.0,
            vec![(Self::PHOTO_LAYER_AREA_INPUT_INDEX, -1.0)],
        );
        nnet.connect_node(
            Self::DONATION_ENERGY_OUTPUT_INDEX,
            -100.0,
            vec![(Self::CELL_ENERGY_INPUT_INDEX, -1.0)],
        );
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
        vec![BuddingCellLayerSpecialty::donation_energy_request(
            1,
            BioEnergy::new(1.0),
        )]
    }

    fn youth_requests(&self) -> Vec<ControlRequest> {
        vec![
            CellLayer::resize_request(0, AreaDelta::new(5.0)),
            CellLayer::resize_request(1, AreaDelta::new(2.0)),
        ]
    }
}

impl CellControl for NeuralNetBuddingControl {
    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let energy_layer_area_input = cell_state.layers[0].area.value() as f32;
        let energy_input = cell_state.energy.value() as f32;
        self.nnet
            .set_node_value(Self::PHOTO_LAYER_AREA_INPUT_INDEX, energy_layer_area_input);
        self.nnet
            .set_node_value(Self::CELL_ENERGY_INPUT_INDEX, energy_input);

        self.nnet.run();

        let energy_layer_resize_output =
            self.nnet.node_value(Self::PHOTO_LAYER_RESIZE_OUTPUT_INDEX) as f64;
        let donation_energy_output =
            self.nnet.node_value(Self::DONATION_ENERGY_OUTPUT_INDEX) as f64;
        let _nnet_requests = vec![
            CellLayer::resize_request(0, AreaDelta::new(energy_layer_resize_output)),
            CellLayer::resize_request(1, AreaDelta::new(energy_layer_resize_output / 2.0)),
            //            BuddingCellLayerSpecialty::donation_energy_request(
            //                1,
            //                BioEnergy::new(donation_energy_output),
            //            ),
        ];
        return _nnet_requests;

        //        if Self::is_adult(cell_state) {
        //            self.adult_requests()
        //        } else {
        //            self.youth_requests()
        //        }
    }
}
