use evo_domain::biology::cell::Cell;
use evo_domain::biology::cloud::CloudParameters;
use evo_domain::biology::layers::*;
use evo_domain::environment::influences::{
    BuoyancyForce, DragForce, SimpleForceInfluence, WeightForce,
};
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_domain::Parameters;
use evo_main::main_support::*;
use std::f64;
use std::f64::consts::PI;

const FLUID_DENSITY: f64 = 0.001;
const FLOAT_LAYER_DENSITY: f64 = 0.0001;
const PHOTO_LAYER_DENSITY: f64 = 0.002;
const CELL_WALL_DENSITY: f64 = 0.002;
const GRAVITY: f64 = -0.05;

fn main() {
    let args = parse_command_line();
    init_and_run_old(create_world(), args);
}

fn create_world() -> World {
    let parameters = Parameters {
        cloud_params: CloudParameters {
            resize_factor: Positive::new(1.01),
            minimum_concentration: Fraction::new(0.1),
        },
    };
    const FLOAT_LAYER_PARAMS: LayerParameters = LayerParameters {
        entropic_damage_health_delta: HealthDelta::new(-0.05),
        decay_rate: Fraction::unchecked(0.05),
        ..LayerParameters::DEFAULT
    };
    const PHOTO_LAYER_PARAMS: LayerParameters = LayerParameters {
        entropic_damage_health_delta: HealthDelta::new(-0.05),
        decay_rate: Fraction::unchecked(0.001),
        ..LayerParameters::DEFAULT
    };
    const CELL_WALL_PARAMS: LayerParameters = LayerParameters {
        entropic_damage_health_delta: HealthDelta::new(-0.05),
        decay_rate: Fraction::unchecked(0.005),
        minimum_intact_thickness: Fraction::unchecked(0.01),
        ..LayerParameters::DEFAULT
    };

    World::new(Position::new(-200.0, -400.0), Position::new(200.0, 0.0))
        .with_parameters(parameters)
        .with_standard_influences()
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
        .with_cell(Cell::new(
            Position::new(0.0, -50.0),
            Velocity::ZERO,
            vec![
                CellLayer::new(
                    Area::new(233.5 * PI),
                    Density::new(FLOAT_LAYER_DENSITY),
                    Tissue::AirBubble,
                    Box::new(NullCellLayerSpecialty::new()),
                )
                .with_parameters(&FLOAT_LAYER_PARAMS),
                CellLayer::new(
                    Area::new(200.0 * PI),
                    Density::new(PHOTO_LAYER_DENSITY),
                    Tissue::Photosynthetic,
                    Box::new(NullCellLayerSpecialty::new()),
                )
                .with_parameters(&PHOTO_LAYER_PARAMS),
                CellLayer::new(
                    Area::new(100.0 * PI),
                    Density::new(CELL_WALL_DENSITY),
                    Tissue::CellWall,
                    Box::new(NullCellLayerSpecialty::new()),
                )
                .with_parameters(&CELL_WALL_PARAMS),
            ],
        ))
}
