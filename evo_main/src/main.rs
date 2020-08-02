use evo_domain::biology::cell::Cell;
use evo_domain::biology::control::*;
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
    let float_layer_health_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[FLOAT_LAYER_INDEX].health.value());
    let _float_layer_area_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[FLOAT_LAYER_INDEX].area.value());
    let photo_layer_health_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[PHOTO_LAYER_INDEX].health.value());
    let photo_layer_area_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[PHOTO_LAYER_INDEX].area.value());
    let bonding_layer_health_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[BONDING_LAYER_INDEX].health.value());
    let bonding_layer_area_input_index =
        builder.add_input_node(|cell_state| cell_state.layers[BONDING_LAYER_INDEX].area.value());
    let bond_0_exists_input_index =
        builder.add_input_node(|cell_state| if cell_state.bond_0_exists { 1.0 } else { 0.0 });

    let donation_energy_index = builder.add_hidden_node(&[(cell_energy_input_index, 0.1)], -100.0);

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
    builder.add_output_node(&[(donation_energy_index, 1.0)], 0.0, |value| {
        BondingCellLayerSpecialty::retain_bond_request(BONDING_LAYER_INDEX, 1, value > 0.0)
    });
    builder.add_output_node(&[(donation_energy_index, 1.0)], 0.0, |value| {
        BondingCellLayerSpecialty::donation_energy_request(
            BONDING_LAYER_INDEX,
            1,
            BioEnergy::new(value.max(0.0)),
        )
    });

    builder.build(randomness)
}
