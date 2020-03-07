extern crate evo_main;
extern crate evo_model;

use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::biology::control::*;
use evo_model::biology::control_requests::*;
use evo_model::biology::genome::*;
use evo_model::biology::layers::*;
use evo_model::environment::influences::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use std::f64::consts::PI;
use std::rc::Rc;

type VecIndex = u16;

fn main() {
    init_and_run(create_world());
}

//const FLUID_DENSITY: f64 = 0.001;
const FLOAT_LAYER_DENSITY: f64 = 0.0001;
const PHOTO_LAYER_DENSITY: f64 = 0.002;
const BUDDING_LAYER_DENSITY: f64 = 0.002;
//const GRAVITY: f64 = -0.05;
const OVERLAP_DAMAGE_HEALTH_DELTA: f64 = -0.1;

const FLOAT_LAYER_INDEX: usize = 0;
const PHOTO_LAYER_INDEX: usize = 1;
const BUDDING_LAYER_INDEX: usize = 2;

fn create_world() -> World {
    const SOME_MUTATION: MutationParameters = MutationParameters {
        weight_mutation_probability: 0.5,
        weight_mutation_stdev: 1.0,
        ..MutationParameters::NO_MUTATION
    };

    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_sunlight(0.0, 1.0)
        .with_influences(vec![Box::new(SimpleForceInfluence::new(Box::new(
            DragForce::new(0.005),
        )))])
        .with_cell(
            create_cell(
                NeuralNetBuddingControl::new_genome(),
                SeededMutationRandomness::new(0, &SOME_MUTATION),
            )
            .with_initial_energy(BioEnergy::new(50.0))
            .with_initial_position(Position::new(200.0, -50.0)),
        )
}

fn create_cell(genome: SparseNeuralNetGenome, randomness: SeededMutationRandomness) -> Cell {
    let genome = Rc::new(genome);
    Cell::new(
        Position::ORIGIN,
        Velocity::ZERO,
        vec![
            create_float_layer(),
            create_photo_layer(),
            create_budding_layer(),
        ],
        Rc::clone(&genome),
        randomness.clone(),
        create_cell,
    )
    .with_control(Box::new(NeuralNetBuddingControl::new(genome, randomness)))
}

fn create_float_layer() -> CellLayer {
    const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::new(-0.1),
        max_growth_rate: 10.0,
        shrinkage_energy_delta: BioEnergyDelta::new(-0.01),
        max_shrinkage_rate: 0.5,
    };
    const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: -0.01,
        overlap_damage_health_delta: OVERLAP_DAMAGE_HEALTH_DELTA,
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(FLOAT_LAYER_DENSITY),
        Color::White,
        Box::new(NullCellLayerSpecialty::new()),
    )
    .with_resize_parameters(&LAYER_RESIZE_PARAMS)
    .with_health_parameters(&LAYER_HEALTH_PARAMS)
}

fn create_photo_layer() -> CellLayer {
    const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::new(-1.0),
        max_growth_rate: 10.0,
        shrinkage_energy_delta: BioEnergyDelta::new(0.0),
        max_shrinkage_rate: 0.1,
    };
    const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: -0.01,
        overlap_damage_health_delta: OVERLAP_DAMAGE_HEALTH_DELTA,
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(PHOTO_LAYER_DENSITY),
        Color::Green,
        Box::new(PhotoCellLayerSpecialty::new(0.1)), // 0.02
    )
    .with_resize_parameters(&LAYER_RESIZE_PARAMS)
    .with_health_parameters(&LAYER_HEALTH_PARAMS)
}

fn create_budding_layer() -> CellLayer {
    const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::new(-1.0),
        max_growth_rate: 10.0,
        shrinkage_energy_delta: BioEnergyDelta::new(0.0),
        max_shrinkage_rate: 0.1,
    };
    const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: -0.01,
        overlap_damage_health_delta: OVERLAP_DAMAGE_HEALTH_DELTA,
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(BUDDING_LAYER_DENSITY),
        Color::Yellow,
        Box::new(BuddingCellLayerSpecialty::new()),
    )
    .with_resize_parameters(&LAYER_RESIZE_PARAMS)
    .with_health_parameters(&LAYER_HEALTH_PARAMS)
}

#[derive(Debug)]
pub struct NeuralNetBuddingControl {
    genome: Rc<SparseNeuralNetGenome>,
    randomness: SeededMutationRandomness,
    nnet: SparseNeuralNet,
}

impl NeuralNetBuddingControl {
    const CELL_ENERGY_INPUT_INDEX: VecIndex = 0;
    const FLOAT_LAYER_AREA_INPUT_INDEX: VecIndex = 1;
    const FLOAT_LAYER_HEALTH_INPUT_INDEX: VecIndex = 2;
    const PHOTO_LAYER_AREA_INPUT_INDEX: VecIndex = 3;
    const PHOTO_LAYER_HEALTH_INPUT_INDEX: VecIndex = 4;
    const BUDDING_LAYER_AREA_INPUT_INDEX: VecIndex = 5;
    const BUDDING_LAYER_HEALTH_INPUT_INDEX: VecIndex = 6;

    const FLOAT_LAYER_RESIZE_OUTPUT_INDEX: VecIndex = 7;
    const FLOAT_LAYER_HEALING_OUTPUT_INDEX: VecIndex = 8;
    const PHOTO_LAYER_RESIZE_OUTPUT_INDEX: VecIndex = 9;
    const PHOTO_LAYER_HEALING_OUTPUT_INDEX: VecIndex = 10;
    const BUDDING_LAYER_RESIZE_OUTPUT_INDEX: VecIndex = 11;
    const BUDDING_LAYER_HEALING_OUTPUT_INDEX: VecIndex = 12;
    const DONATION_ENERGY_OUTPUT_INDEX: VecIndex = 13;

    fn new(genome: Rc<SparseNeuralNetGenome>, randomness: SeededMutationRandomness) -> Self {
        NeuralNetBuddingControl {
            genome: Rc::clone(&genome),
            randomness,
            nnet: SparseNeuralNet::new(genome),
        }
    }

    fn new_genome() -> SparseNeuralNetGenome {
        let mut genome = SparseNeuralNetGenome::new(TransferFn::IDENTITY);
        genome.connect_node(
            Self::FLOAT_LAYER_HEALING_OUTPUT_INDEX,
            1.0,
            &[(Self::FLOAT_LAYER_HEALTH_INPUT_INDEX, -1.0)],
        );
        genome.connect_node(
            Self::PHOTO_LAYER_RESIZE_OUTPUT_INDEX,
            800.0,
            &[(Self::PHOTO_LAYER_AREA_INPUT_INDEX, -1.0)],
        );
        genome.connect_node(
            Self::PHOTO_LAYER_HEALING_OUTPUT_INDEX,
            1.0,
            &[(Self::PHOTO_LAYER_HEALTH_INPUT_INDEX, -1.0)],
        );
        genome.connect_node(
            Self::BUDDING_LAYER_RESIZE_OUTPUT_INDEX,
            200.0,
            &[(Self::BUDDING_LAYER_AREA_INPUT_INDEX, -1.0)],
        );
        genome.connect_node(
            Self::BUDDING_LAYER_HEALING_OUTPUT_INDEX,
            1.0,
            &[(Self::BUDDING_LAYER_HEALTH_INPUT_INDEX, -1.0)],
        );
        genome.connect_node(
            Self::DONATION_ENERGY_OUTPUT_INDEX,
            -100.0,
            &[(Self::CELL_ENERGY_INPUT_INDEX, 0.1)],
        );
        genome
    }
}

impl CellControl for NeuralNetBuddingControl {
    fn run(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let cell_energy = cell_state.energy.value() as f32;
        let float_layer_area = cell_state.layers[FLOAT_LAYER_INDEX].area.value() as f32;
        let float_layer_health = cell_state.layers[FLOAT_LAYER_INDEX].health as f32;
        let photo_layer_area = cell_state.layers[PHOTO_LAYER_INDEX].area.value() as f32;
        let photo_layer_health = cell_state.layers[PHOTO_LAYER_INDEX].health as f32;
        let budding_layer_area = cell_state.layers[BUDDING_LAYER_INDEX].area.value() as f32;
        let budding_layer_health = cell_state.layers[BUDDING_LAYER_INDEX].health as f32;

        self.nnet
            .set_node_value(Self::CELL_ENERGY_INPUT_INDEX, cell_energy);
        self.nnet
            .set_node_value(Self::FLOAT_LAYER_AREA_INPUT_INDEX, float_layer_area);
        self.nnet
            .set_node_value(Self::FLOAT_LAYER_HEALTH_INPUT_INDEX, float_layer_health);
        self.nnet
            .set_node_value(Self::PHOTO_LAYER_AREA_INPUT_INDEX, photo_layer_area);
        self.nnet
            .set_node_value(Self::PHOTO_LAYER_HEALTH_INPUT_INDEX, photo_layer_health);
        self.nnet
            .set_node_value(Self::BUDDING_LAYER_AREA_INPUT_INDEX, budding_layer_area);
        self.nnet
            .set_node_value(Self::BUDDING_LAYER_HEALTH_INPUT_INDEX, budding_layer_health);

        self.nnet.run();

        let float_layer_area_delta =
            self.nnet.node_value(Self::FLOAT_LAYER_RESIZE_OUTPUT_INDEX) as f64;
        let float_layer_healing =
            self.nnet.node_value(Self::FLOAT_LAYER_HEALING_OUTPUT_INDEX) as f64;
        let photo_layer_area_delta =
            self.nnet.node_value(Self::PHOTO_LAYER_RESIZE_OUTPUT_INDEX) as f64;
        let photo_layer_healing =
            self.nnet.node_value(Self::PHOTO_LAYER_HEALING_OUTPUT_INDEX) as f64;
        let budding_layer_area_delta =
            self.nnet
                .node_value(Self::BUDDING_LAYER_RESIZE_OUTPUT_INDEX) as f64;
        let budding_layer_healing =
            self.nnet
                .node_value(Self::BUDDING_LAYER_HEALING_OUTPUT_INDEX) as f64;
        let donation_energy = self.nnet.node_value(Self::DONATION_ENERGY_OUTPUT_INDEX) as f64;

        vec![
            CellLayer::resize_request(FLOAT_LAYER_INDEX, AreaDelta::new(float_layer_area_delta)),
            CellLayer::healing_request(FLOAT_LAYER_INDEX, float_layer_healing.max(0.0).min(1.0)),
            CellLayer::resize_request(PHOTO_LAYER_INDEX, AreaDelta::new(photo_layer_area_delta)),
            CellLayer::healing_request(PHOTO_LAYER_INDEX, photo_layer_healing.max(0.0).min(1.0)),
            CellLayer::resize_request(
                BUDDING_LAYER_INDEX,
                AreaDelta::new(budding_layer_area_delta),
            ),
            CellLayer::healing_request(
                BUDDING_LAYER_INDEX,
                budding_layer_healing.max(0.0).min(1.0),
            ),
            BuddingCellLayerSpecialty::donation_energy_request(
                BUDDING_LAYER_INDEX,
                BioEnergy::new(donation_energy.max(0.0)),
            ),
        ]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(Self::new(
            Rc::new(self.genome.spawn(&mut self.randomness)),
            self.randomness.spawn(),
        ))
    }
}
