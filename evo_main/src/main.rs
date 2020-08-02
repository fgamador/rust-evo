use evo_domain::biology::cell::Cell;
use evo_domain::biology::control::*;
use evo_domain::biology::control_requests::*;
use evo_domain::biology::genome::*;
use evo_domain::biology::layers::*;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::init_and_run;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

const FLUID_DENSITY: f64 = 0.001;
const FLOAT_LAYER_DENSITY: f64 = 0.0001;
const PHOTO_LAYER_DENSITY: f64 = 0.002;
const BONDING_LAYER_DENSITY: f64 = 0.002;
const GRAVITY: f64 = -0.05;
const OVERLAP_DAMAGE_HEALTH_DELTA: f64 = -0.1;

const FLOAT_LAYER_INDEX: usize = 0;
const PHOTO_LAYER_INDEX: usize = 1;
const BONDING_LAYER_INDEX: usize = 2;

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_bond_forces()
        .with_sunlight(0.0, 1.0)
        .with_per_cell_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(
                GRAVITY,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(
                GRAVITY,
                FLUID_DENSITY,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.005)))),
        ])
        .with_cell(
            create_cell()
                .with_initial_energy(BioEnergy::new(50.0))
                .with_initial_position(Position::new(200.0, -50.0)),
        )
}

fn create_cell() -> Cell {
    const SOME_MUTATION: MutationParameters = MutationParameters {
        weight_mutation_probability: 0.5,
        weight_mutation_stdev: 1.0,
        ..MutationParameters::NO_MUTATION
    };

    Cell::new(
        Position::ORIGIN,
        Velocity::ZERO,
        vec![
            create_float_layer(),
            create_photo_layer(),
            create_bonding_layer(),
        ],
    )
    .with_control(Box::new(create_control(SeededMutationRandomness::new(
        0,
        &SOME_MUTATION,
    ))))
    // .with_control(Box::new(NeuralNetBuddingControl::new(
    //     NeuralNetBuddingControl::new_genome(),
    //     SeededMutationRandomness::new(0, &SOME_MUTATION),
    // )))
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
        entropic_damage_health_delta: HealthDelta::new(-0.01),
        overlap_damage_health_delta: HealthDelta::new(OVERLAP_DAMAGE_HEALTH_DELTA),
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
        entropic_damage_health_delta: HealthDelta::new(-0.01),
        overlap_damage_health_delta: HealthDelta::new(OVERLAP_DAMAGE_HEALTH_DELTA),
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

fn create_bonding_layer() -> CellLayer {
    const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::new(-1.0),
        max_growth_rate: 10.0,
        shrinkage_energy_delta: BioEnergyDelta::new(0.0),
        max_shrinkage_rate: 0.1,
    };
    const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: HealthDelta::new(-0.01),
        overlap_damage_health_delta: HealthDelta::new(OVERLAP_DAMAGE_HEALTH_DELTA),
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(BONDING_LAYER_DENSITY),
        Color::Yellow,
        Box::new(BondingCellLayerSpecialty::new()),
    )
    .with_resize_parameters(&LAYER_RESIZE_PARAMS)
    .with_health_parameters(&LAYER_HEALTH_PARAMS)
}

fn create_control(randomness: SeededMutationRandomness) -> NeuralNetControl {
    let mut builder = NeuralNetControlBuilder::new(TransferFn::IDENTITY);

    let cell_energy_input_index = builder.add_input_node(|cell_state| cell_state.energy.value());
    let cell_y_input_index = builder.add_input_node(|cell_state| cell_state.center.y());
    // let float_layer_area_input_index = builder.add_input_node(|cell_state| {
    //     cell_state.layers[FLOAT_LAYER_INDEX].area.value()
    // });
    let float_layer_health_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[FLOAT_LAYER_INDEX].health.value());
    let photo_layer_area_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[PHOTO_LAYER_INDEX].area.value());
    let photo_layer_health_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[PHOTO_LAYER_INDEX].health.value());
    let bonding_layer_area_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[BONDING_LAYER_INDEX].area.value());
    let bonding_layer_health_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[BONDING_LAYER_INDEX].health.value());
    let bond_0_exists_input_index =
        builder.add_input_node(|cell_state| if cell_state.bond_0_exists { 1.0 } else { 0.0 });

    builder.add_output_node(&[(float_layer_health_input_index, -1.0)], 1.0, |value| {
        CellLayer::healing_request(FLOAT_LAYER_INDEX, HealthDelta::new(value.max(0.0)))
    });
    builder.add_output_node(&[(cell_y_input_index, -1.0)], -100.0, |value| {
        CellLayer::resize_request(FLOAT_LAYER_INDEX, AreaDelta::new(value))
    });
    builder.add_output_node(&[(photo_layer_health_input_index, -1.0)], 1.0, |value| {
        CellLayer::healing_request(PHOTO_LAYER_INDEX, HealthDelta::new(value.max(0.0)))
    });
    builder.add_output_node(&[(photo_layer_area_input_index, -1.0)], 800.0, |value| {
        CellLayer::resize_request(PHOTO_LAYER_INDEX, AreaDelta::new(value))
    });
    builder.add_output_node(&[(bonding_layer_health_input_index, -1.0)], 1.0, |value| {
        CellLayer::healing_request(BONDING_LAYER_INDEX, HealthDelta::new(value.max(0.0)))
    });
    builder.add_output_node(&[(bonding_layer_area_input_index, -1.0)], 200.0, |value| {
        CellLayer::resize_request(BONDING_LAYER_INDEX, AreaDelta::new(value))
    });
    builder.add_output_node(
        &[
            (cell_energy_input_index, -1.0),
            (bond_0_exists_input_index, 100.0),
        ],
        0.0,
        |value| BondingCellLayerSpecialty::retain_bond_request(BONDING_LAYER_INDEX, 0, value > 0.0),
    );
    builder.add_output_node(&[(cell_energy_input_index, 0.1)], -100.0, |value| {
        BondingCellLayerSpecialty::retain_bond_request(BONDING_LAYER_INDEX, 1, value > 0.0)
    });
    // TODO dedup: add hidden node
    builder.add_output_node(&[(cell_energy_input_index, 0.1)], -100.0, |value| {
        BondingCellLayerSpecialty::donation_energy_request(
            BONDING_LAYER_INDEX,
            1,
            BioEnergy::new(value.max(0.0)),
        )
    });

    builder.build(randomness)
}

#[derive(Debug)]
pub struct NeuralNetBuddingControl {
    nnet: SparseNeuralNet,
    randomness: SeededMutationRandomness,
}

impl NeuralNetBuddingControl {
    const CELL_ENERGY_INPUT_INDEX: VecIndex = 0;
    const CELL_Y_INPUT_INDEX: VecIndex = 1;
    const FLOAT_LAYER_AREA_INPUT_INDEX: VecIndex = 2;
    const FLOAT_LAYER_HEALTH_INPUT_INDEX: VecIndex = 3;
    const PHOTO_LAYER_AREA_INPUT_INDEX: VecIndex = 4;
    const PHOTO_LAYER_HEALTH_INPUT_INDEX: VecIndex = 5;
    const BONDING_LAYER_AREA_INPUT_INDEX: VecIndex = 6;
    const BONDING_LAYER_HEALTH_INPUT_INDEX: VecIndex = 7;
    const BOND_0_EXISTS_INPUT_INDEX: VecIndex = 8;

    const FLOAT_LAYER_RESIZE_OUTPUT_INDEX: VecIndex = 9;
    const FLOAT_LAYER_HEALING_OUTPUT_INDEX: VecIndex = 10;
    const PHOTO_LAYER_RESIZE_OUTPUT_INDEX: VecIndex = 11;
    const PHOTO_LAYER_HEALING_OUTPUT_INDEX: VecIndex = 12;
    const BONDING_LAYER_RESIZE_OUTPUT_INDEX: VecIndex = 13;
    const BONDING_LAYER_HEALING_OUTPUT_INDEX: VecIndex = 14;
    const BOND_0_RETAIN_OUTPUT_INDEX: VecIndex = 15;
    const BOND_1_DONATION_ENERGY_OUTPUT_INDEX: VecIndex = 16;

    pub fn new(genome: SparseNeuralNetGenome, randomness: SeededMutationRandomness) -> Self {
        NeuralNetBuddingControl {
            nnet: SparseNeuralNet::new(genome),
            randomness,
        }
    }

    pub fn new_genome() -> SparseNeuralNetGenome {
        let mut genome = SparseNeuralNetGenome::new(TransferFn::IDENTITY);
        genome.connect_node(
            Self::FLOAT_LAYER_RESIZE_OUTPUT_INDEX,
            -100.0,
            &[(Self::CELL_Y_INPUT_INDEX, -1.0)],
        );
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
            Self::BONDING_LAYER_RESIZE_OUTPUT_INDEX,
            200.0,
            &[(Self::BONDING_LAYER_AREA_INPUT_INDEX, -1.0)],
        );
        genome.connect_node(
            Self::BONDING_LAYER_HEALING_OUTPUT_INDEX,
            1.0,
            &[(Self::BONDING_LAYER_HEALTH_INPUT_INDEX, -1.0)],
        );
        genome.connect_node(
            Self::BOND_0_RETAIN_OUTPUT_INDEX,
            0.0,
            &[
                (Self::CELL_ENERGY_INPUT_INDEX, -1.0),
                (Self::BOND_0_EXISTS_INPUT_INDEX, 100.0),
            ],
        );
        genome.connect_node(
            Self::BOND_1_DONATION_ENERGY_OUTPUT_INDEX,
            -100.0,
            &[(Self::CELL_ENERGY_INPUT_INDEX, 0.1)],
        );
        genome
    }
}

impl CellControl for NeuralNetBuddingControl {
    fn run(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let cell_energy = cell_state.energy.value() as f32;
        let cell_y = cell_state.center.y() as f32;
        let float_layer_area = cell_state.layers[FLOAT_LAYER_INDEX].area.value() as f32;
        let float_layer_health = cell_state.layers[FLOAT_LAYER_INDEX].health.value() as f32;
        let photo_layer_area = cell_state.layers[PHOTO_LAYER_INDEX].area.value() as f32;
        let photo_layer_health = cell_state.layers[PHOTO_LAYER_INDEX].health.value() as f32;
        let bonding_layer_area = cell_state.layers[BONDING_LAYER_INDEX].area.value() as f32;
        let bonding_layer_health = cell_state.layers[BONDING_LAYER_INDEX].health.value() as f32;
        let bond_0_exists: f32 = if cell_state.bond_0_exists { 1.0 } else { 0.0 };

        self.nnet
            .set_node_value(Self::CELL_ENERGY_INPUT_INDEX, cell_energy);
        self.nnet.set_node_value(Self::CELL_Y_INPUT_INDEX, cell_y);
        self.nnet
            .set_node_value(Self::FLOAT_LAYER_AREA_INPUT_INDEX, float_layer_area);
        self.nnet
            .set_node_value(Self::FLOAT_LAYER_HEALTH_INPUT_INDEX, float_layer_health);
        self.nnet
            .set_node_value(Self::PHOTO_LAYER_AREA_INPUT_INDEX, photo_layer_area);
        self.nnet
            .set_node_value(Self::PHOTO_LAYER_HEALTH_INPUT_INDEX, photo_layer_health);
        self.nnet
            .set_node_value(Self::BONDING_LAYER_AREA_INPUT_INDEX, bonding_layer_area);
        self.nnet
            .set_node_value(Self::BONDING_LAYER_HEALTH_INPUT_INDEX, bonding_layer_health);
        self.nnet
            .set_node_value(Self::BOND_0_EXISTS_INPUT_INDEX, bond_0_exists);

        self.nnet.run();

        let float_layer_area_delta =
            self.nnet.node_value(Self::FLOAT_LAYER_RESIZE_OUTPUT_INDEX) as f64;
        let float_layer_healing =
            self.nnet.node_value(Self::FLOAT_LAYER_HEALING_OUTPUT_INDEX) as f64;
        let photo_layer_area_delta =
            self.nnet.node_value(Self::PHOTO_LAYER_RESIZE_OUTPUT_INDEX) as f64;
        let photo_layer_healing =
            self.nnet.node_value(Self::PHOTO_LAYER_HEALING_OUTPUT_INDEX) as f64;
        let bonding_layer_area_delta =
            self.nnet
                .node_value(Self::BONDING_LAYER_RESIZE_OUTPUT_INDEX) as f64;
        let bonding_layer_healing =
            self.nnet
                .node_value(Self::BONDING_LAYER_HEALING_OUTPUT_INDEX) as f64;
        let retain_bond_0 = self.nnet.node_value(Self::BOND_0_RETAIN_OUTPUT_INDEX) as f64;
        let donation_energy =
            self.nnet
                .node_value(Self::BOND_1_DONATION_ENERGY_OUTPUT_INDEX) as f64;

        vec![
            CellLayer::healing_request(
                FLOAT_LAYER_INDEX,
                HealthDelta::new(float_layer_healing.max(0.0)),
            ),
            CellLayer::resize_request(FLOAT_LAYER_INDEX, AreaDelta::new(float_layer_area_delta)),
            CellLayer::healing_request(
                PHOTO_LAYER_INDEX,
                HealthDelta::new(photo_layer_healing.max(0.0)),
            ),
            CellLayer::resize_request(PHOTO_LAYER_INDEX, AreaDelta::new(photo_layer_area_delta)),
            CellLayer::healing_request(
                BONDING_LAYER_INDEX,
                HealthDelta::new(bonding_layer_healing.max(0.0)),
            ),
            CellLayer::resize_request(
                BONDING_LAYER_INDEX,
                AreaDelta::new(bonding_layer_area_delta),
            ),
            BondingCellLayerSpecialty::retain_bond_request(
                BONDING_LAYER_INDEX,
                0,
                retain_bond_0 > 0.0,
            ),
            BondingCellLayerSpecialty::retain_bond_request(
                BONDING_LAYER_INDEX,
                1,
                donation_energy > 0.0,
            ),
            BondingCellLayerSpecialty::budding_angle_request(
                BONDING_LAYER_INDEX,
                1,
                Angle::from_radians(0.0),
            ),
            BondingCellLayerSpecialty::donation_energy_request(
                BONDING_LAYER_INDEX,
                1,
                BioEnergy::new(donation_energy.max(0.0)),
            ),
        ]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(NeuralNetBuddingControl {
            nnet: self.nnet.spawn(&mut self.randomness),
            randomness: self.randomness.clone(),
        })
    }
}
