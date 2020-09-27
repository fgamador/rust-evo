use evo_domain::biology::cell::Cell;
use evo_domain::biology::cloud::CloudParameters;
use evo_domain::biology::control::*;
use evo_domain::biology::genome::*;
use evo_domain::biology::layers::*;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_domain::Parameters;
use evo_main::main_support::*;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world);
}

const FLUID_DENSITY: f64 = 0.001;
const FLOAT_LAYER_DENSITY: f64 = 0.0001;
const PHOTO_LAYER_DENSITY: f64 = 0.002;
const BONDING_LAYER_DENSITY: f64 = 0.002;
const CELL_WALL_DENSITY: f64 = 0.002;
const GRAVITY: f64 = -0.05;
const OVERLAP_DAMAGE_HEALTH_DELTA: f64 = -0.1;

const FLOAT_LAYER_INDEX: usize = 0;
const PHOTO_LAYER_INDEX: usize = 1;
const BONDING_LAYER_INDEX: usize = 2;
const CELL_WALL_INDEX: usize = 3;

fn create_world(seed: u64) -> World {
    let parameters = Parameters {
        cloud_params: CloudParameters {
            resize_factor: Positive::new(1.01),
            minimum_concentration: Fraction::new(0.1),
        },
    };
    World::new(Position::new(0.0, -400.0), Position::new(1200.0, 0.0))
        .with_parameters(parameters)
        .with_standard_influences()
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
            create_cell(seed)
                .with_initial_energy(BioEnergy::new(50.0))
                .with_initial_position(Position::new(200.0, -50.0)),
        )
}

fn create_cell(seed: u64) -> Cell {
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
            create_cell_wall(),
        ],
    )
    .with_control(Box::new(create_control(SeededMutationRandomness::new(
        seed,
        &SOME_MUTATION,
    ))))
}

fn create_float_layer() -> CellLayer {
    const PARAMS: LayerParameters = LayerParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: HealthDelta::new(-0.01),
        overlap_damage_health_delta: HealthDelta::new(OVERLAP_DAMAGE_HEALTH_DELTA),
        growth_energy_delta: BioEnergyDelta::new(-0.1),
        max_growth_rate: Positive::unchecked(10.0),
        shrinkage_energy_delta: BioEnergyDelta::new(-0.01),
        max_shrinkage_rate: 0.5,
        decay_rate: Fraction::unchecked(0.05),
        ..LayerParameters::DEFAULT
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(FLOAT_LAYER_DENSITY),
        Tissue::AirBubble,
        Box::new(NullCellLayerSpecialty::new()),
    )
    .with_parameters(&PARAMS)
}

fn create_photo_layer() -> CellLayer {
    const PARAMS: LayerParameters = LayerParameters {
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
    .with_parameters(&PARAMS)
}

fn create_bonding_layer() -> CellLayer {
    const PARAMS: LayerParameters = LayerParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: HealthDelta::new(-0.01),
        overlap_damage_health_delta: HealthDelta::new(OVERLAP_DAMAGE_HEALTH_DELTA),
        growth_energy_delta: BioEnergyDelta::new(-1.0),
        max_growth_rate: Positive::unchecked(10.0),
        shrinkage_energy_delta: BioEnergyDelta::new(0.0),
        max_shrinkage_rate: 0.1,
        ..LayerParameters::DEFAULT
    };
    const BONDING_PARAMS: BondingLayerParameters = BondingLayerParameters {
        max_donation_energy_per_unit_area: BioEnergy::unchecked(0.5),
        donation_energy_tax_rate: Fraction::unchecked(0.1),
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(BONDING_LAYER_DENSITY),
        Tissue::Bonding,
        Box::new(BondingCellLayerSpecialty::new().with_parameters(&BONDING_PARAMS)),
    )
    .with_parameters(&PARAMS)
}

fn create_cell_wall() -> CellLayer {
    const PARAMS: LayerParameters = LayerParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: HealthDelta::new(-0.01),
        overlap_damage_health_delta: HealthDelta::new(OVERLAP_DAMAGE_HEALTH_DELTA),
        growth_energy_delta: BioEnergyDelta::new(-0.1),
        max_growth_rate: Positive::unchecked(10.0),
        shrinkage_energy_delta: BioEnergyDelta::new(-0.01),
        max_shrinkage_rate: 0.5,
        decay_rate: Fraction::unchecked(0.005),
        minimum_intact_thickness: Fraction::unchecked(0.01),
        ..LayerParameters::DEFAULT
    };

    CellLayer::new(
        Area::new(2.0 * PI),
        Density::new(CELL_WALL_DENSITY),
        Tissue::CellWall,
        Box::new(NullCellLayerSpecialty::new()),
    )
    .with_parameters(&PARAMS)
}

fn create_control(randomness: SeededMutationRandomness) -> NeuralNetControl {
    let mut builder = NeuralNetControlBuilder::new(TransferFn::IDENTITY);

    let cell_energy_input_index =
        builder.add_input_node("<energy", |cell_state| cell_state.energy.value());
    let cell_y_input_index =
        builder.add_input_node("<center y", |cell_state| cell_state.center.y());
    let float_layer_health_input_index = builder.add_input_node("<float health", |cell_state| {
        cell_state.layers[FLOAT_LAYER_INDEX].health.value()
    });
    let _float_layer_area_input_index = builder.add_input_node("<float area", |cell_state| {
        cell_state.layers[FLOAT_LAYER_INDEX].area.value()
    });
    let photo_layer_health_input_index = builder.add_input_node("<photo health", |cell_state| {
        cell_state.layers[PHOTO_LAYER_INDEX].health.value()
    });
    let photo_layer_area_input_index = builder.add_input_node("<photo area", |cell_state| {
        cell_state.layers[PHOTO_LAYER_INDEX].area.value()
    });
    let bonding_layer_health_input_index = builder
        .add_input_node("<bonding health", |cell_state| {
            cell_state.layers[BONDING_LAYER_INDEX].health.value()
        });
    let bonding_layer_area_input_index = builder.add_input_node("<bonding area", |cell_state| {
        cell_state.layers[BONDING_LAYER_INDEX].area.value()
    });
    let cell_wall_health_input_index = builder.add_input_node("<cell-wall health", |cell_state| {
        cell_state.layers[CELL_WALL_INDEX].health.value()
    });
    let bond_0_exists_input_index = builder.add_input_node("<bond 0 exists", |cell_state| {
        bool_to_value1d(cell_state.bond_0_exists)
    });

    let donation_energy_index =
        builder.add_hidden_node("donation", &[(cell_energy_input_index, 0.1)], -100.0);

    builder.add_output_node(
        ">float healing",
        &[(float_layer_health_input_index, -1.0)],
        1.0,
        |value| CellLayer::healing_request(FLOAT_LAYER_INDEX, HealthDelta::new(value.max(0.0))),
    );
    builder.add_output_node(
        ">float resize",
        &[(cell_y_input_index, -1.0)],
        -100.0,
        |value| CellLayer::resize_request(FLOAT_LAYER_INDEX, AreaDelta::new(value)),
    );
    builder.add_output_node(
        ">photo healing",
        &[(photo_layer_health_input_index, -1.0)],
        1.0,
        |value| CellLayer::healing_request(PHOTO_LAYER_INDEX, HealthDelta::new(value.max(0.0))),
    );
    builder.add_output_node(
        ">photo resize",
        &[(photo_layer_area_input_index, -1.0)],
        800.0,
        |value| CellLayer::resize_request(PHOTO_LAYER_INDEX, AreaDelta::new(value)),
    );
    builder.add_output_node(
        ">bonding healing",
        &[(bonding_layer_health_input_index, -1.0)],
        1.0,
        |value| CellLayer::healing_request(BONDING_LAYER_INDEX, HealthDelta::new(value.max(0.0))),
    );
    builder.add_output_node(
        ">bonding resize",
        &[(bonding_layer_area_input_index, -1.0)],
        200.0,
        |value| CellLayer::resize_request(BONDING_LAYER_INDEX, AreaDelta::new(value)),
    );
    builder.add_output_node(
        ">cell-wall healing",
        &[(cell_wall_health_input_index, -1.0)],
        1.0,
        |value| CellLayer::healing_request(CELL_WALL_INDEX, HealthDelta::new(value.max(0.0))),
    );
    builder.add_output_node(
        ">retain 0",
        &[
            (cell_energy_input_index, -1.0),
            (bond_0_exists_input_index, 100.0),
        ],
        0.0,
        |value| BondingCellLayerSpecialty::retain_bond_request(BONDING_LAYER_INDEX, 0, value > 0.0),
    );
    builder.add_output_node(">retain 1", &[(donation_energy_index, 1.0)], 0.0, |value| {
        BondingCellLayerSpecialty::retain_bond_request(BONDING_LAYER_INDEX, 1, value > 0.0)
    });
    builder.add_output_node(">donate 1", &[(donation_energy_index, 1.0)], 0.0, |value| {
        BondingCellLayerSpecialty::donation_energy_request(
            BONDING_LAYER_INDEX,
            1,
            BioEnergy::new(value.max(0.0)),
        )
    });

    builder.build(randomness)
}
