use evo_domain::biology::cell::Cell;
use evo_domain::biology::layers::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;
use std::f64::consts::PI;

fn main() {
    let args = parse_command_line();
    init_and_run_old(create_world(), args);
}

fn create_world() -> World {
    const LAYER_PARAMS: LayerParameters = LayerParameters {
        overlap_damage_health_delta: HealthDelta::new(-2.0),
        ..LayerParameters::DEFAULT
    };

    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_standard_influences()
        .with_cell(Cell::new(
            Position::new(200.0, -50.0),
            Velocity::new(2.0, 0.0),
            vec![CellLayer::new(
                Area::new(200.0 * PI),
                Density::new(0.001),
                Tissue::Photosynthetic,
                Box::new(NullCellLayerSpecialty::new()),
            )
            .with_parameters(&LAYER_PARAMS)],
        ))
}
