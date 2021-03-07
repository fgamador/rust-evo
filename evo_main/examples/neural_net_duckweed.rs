use evo_domain::biology::cell::Cell;
use evo_domain::biology::control::*;
use evo_domain::biology::genome::*;
use evo_domain::biology::layers::*;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world);
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

fn create_world(seed: u64) -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_standard_influences()
        .with_sunlight(0.0, 1.0)
        .with_per_cell_influence(Box::new(SimpleForceInfluence::new(Box::new(
            DragForce::new(0.005),
        ))))
        .with_cell(
            create_cell(seed)
                .with_initial_energy(BioEnergy::new(50.0))
                .with_initial_position(Position::new(200.0, -50.0)),
        )
}

fn create_cell(seed: u64) -> Cell {
    const SOME_MUTATION: MutationParameters = MutationParameters {
        // weight_mutation_probability: 0.5,
        // weight_mutation_stdev: 1.0,
        ..MutationParameters::NO_MUTATION
    };

    Cell::new(
        Position::ORIGIN,
        Velocity::ZERO,
        vec![
            create_float_layer(),
            create_photo_layer(),
            create_budding_layer(),
        ],
    )
    .with_control(Box::new(create_control(SeededMutationRandomness::new(
        seed,
        &SOME_MUTATION,
    ))))
}

fn create_float_layer() -> CellLayer {
    const LAYER_PARAMS: LayerParameters = LayerParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: HealthDelta::new(-0.01),
        overlap_damage_health_delta: HealthDelta::new(OVERLAP_DAMAGE_HEALTH_DELTA),
        growth_energy_delta: BioEnergyDelta::new(-0.1),
        max_growth_rate: Positive::unchecked(10.0),
        shrinkage_energy_delta: BioEnergyDelta::new(-0.01),
        max_shrinkage_rate: 0.5,
        ..LayerParameters::DEFAULT
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(FLOAT_LAYER_DENSITY),
        Tissue::AirBubble,
        Box::new(NullCellLayerSpecialty::new()),
    )
    .with_parameters(&LAYER_PARAMS)
}

fn create_photo_layer() -> CellLayer {
    const LAYER_PARAMS: LayerParameters = LayerParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: HealthDelta::new(-0.01),
        overlap_damage_health_delta: HealthDelta::new(OVERLAP_DAMAGE_HEALTH_DELTA),
        growth_energy_delta: BioEnergyDelta::new(-1.0),
        max_growth_rate: Positive::unchecked(10.0),
        shrinkage_energy_delta: BioEnergyDelta::new(0.0),
        max_shrinkage_rate: 0.1,
        ..LayerParameters::DEFAULT
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(PHOTO_LAYER_DENSITY),
        Tissue::Photosynthetic,
        Box::new(PhotoCellLayerSpecialty::new(Fraction::new(0.1))), // 0.02
    )
    .with_parameters(&LAYER_PARAMS)
}

fn create_budding_layer() -> CellLayer {
    const LAYER_PARAMS: LayerParameters = LayerParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: HealthDelta::new(-0.01),
        overlap_damage_health_delta: HealthDelta::new(OVERLAP_DAMAGE_HEALTH_DELTA),
        growth_energy_delta: BioEnergyDelta::new(-1.0),
        max_growth_rate: Positive::unchecked(10.0),
        shrinkage_energy_delta: BioEnergyDelta::new(0.0),
        max_shrinkage_rate: 0.1,
        ..LayerParameters::DEFAULT
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(BUDDING_LAYER_DENSITY),
        Tissue::Bonding,
        Box::new(BondingCellLayerSpecialty::new()),
    )
    .with_parameters(&LAYER_PARAMS)
}

fn create_control(randomness: SeededMutationRandomness) -> NeuralNetControl {
    let mut builder = NeuralNetControlBuilder::new(TransferFn::IDENTITY);

    let cell_energy_input_index =
        builder.add_input_node("<cell energy", |cell_state| cell_state.energy.value());
    // let float_layer_area_input_index = builder.add_input_node("<float area", |cell_state| {
    //     cell_state.layers[FLOAT_LAYER_INDEX].area.value()
    // });
    let float_layer_health_input_index = builder.add_input_node("<float health", |cell_state| {
        cell_state.layers[FLOAT_LAYER_INDEX].health.value()
    });
    let photo_layer_area_input_index = builder.add_input_node("<photo area", |cell_state| {
        cell_state.layers[PHOTO_LAYER_INDEX].area.value()
    });
    let photo_layer_health_input_index = builder.add_input_node("<photo health", |cell_state| {
        cell_state.layers[PHOTO_LAYER_INDEX].health.value()
    });
    let budding_layer_area_input_index = builder.add_input_node("<budding area", |cell_state| {
        cell_state.layers[BUDDING_LAYER_INDEX].area.value()
    });
    let budding_layer_health_input_index = builder
        .add_input_node("<budding health", |cell_state| {
            cell_state.layers[BUDDING_LAYER_INDEX].health.value()
        });

    // let desired_y_delta_index = builder.add_hidden_node(
    //     "desired y delta",
    //     &[(center_y_input_index, -1.0)],
    //     GOAL_DEPTH as f32,
    // );

    // builder.add_output_node(
    //     ">float resize",
    //     &[
    //         (
    //             photo_layer_area_input_index,
    //             NEUTRALLY_BUOYANT_AREA_RATIO as f32,
    //         ),
    //         (desired_y_delta_index, 100.0),
    //         (float_layer_area_input_index, -1.0),
    //     ],
    //     0.0,
    //     |value| CellLayer::resize_request(FLOAT_LAYER_INDEX, AreaDelta::new(value)),
    // );
    builder.add_output_node(
        ">float health",
        &[(float_layer_health_input_index, -1.0)],
        1.0,
        |value| CellLayer::healing_request(FLOAT_LAYER_INDEX, HealthDelta::new(value.max(0.0))),
    );
    builder.add_output_node(
        ">photo resize",
        &[(photo_layer_area_input_index, -1.0)],
        800.0,
        |value| CellLayer::resize_request(PHOTO_LAYER_INDEX, AreaDelta::new(value)),
    );
    builder.add_output_node(
        ">photo health",
        &[(photo_layer_health_input_index, -1.0)],
        1.0,
        |value| CellLayer::healing_request(PHOTO_LAYER_INDEX, HealthDelta::new(value.max(0.0))),
    );
    builder.add_output_node(
        ">budding resize",
        &[(budding_layer_area_input_index, -1.0)],
        200.0,
        |value| CellLayer::resize_request(BUDDING_LAYER_INDEX, AreaDelta::new(value)),
    );
    builder.add_output_node(
        ">budding health",
        &[(budding_layer_health_input_index, -1.0)],
        1.0,
        |value| CellLayer::healing_request(BUDDING_LAYER_INDEX, HealthDelta::new(value.max(0.0))),
    );

    builder.add_output_node(">budding angle", &[], 0.0, |value| {
        BondingCellLayerSpecialty::budding_angle_request(
            BUDDING_LAYER_INDEX,
            1,
            Angle::from_radians(value),
        )
    });
    let donation_energy_output_index = builder.add_output_node(
        ">donation energy",
        &[(cell_energy_input_index, 1.0)],
        -1000.0,
        |value| {
            BondingCellLayerSpecialty::donation_energy_request(
                BUDDING_LAYER_INDEX,
                1,
                BioEnergy::new(value.max(0.0)),
            )
        },
    );
    builder.add_node_output(donation_energy_output_index, |value| {
        BondingCellLayerSpecialty::retain_bond_request(BUDDING_LAYER_INDEX, 1, value > 0.0)
    });

    builder.add_output_node(
        ">retain parent",
        &[(cell_energy_input_index, 1.0)],
        -500.0,
        |value| BondingCellLayerSpecialty::retain_bond_request(BUDDING_LAYER_INDEX, 0, value < 0.0),
    );

    builder.build(randomness)
}
