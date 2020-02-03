extern crate evo_main;
extern crate evo_model;

use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::biology::layers::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_cell(Cell::new(
            Position::new(200.0, -50.0),
            Velocity::new(2.0, 0.0),
            vec![CellLayer::new(
                Area::new(200.0 * PI),
                Density::new(0.001),
                Color::Green,
                Box::new(NullCellLayerSpecialty::new()),
            )
            .with_health_parameters(LayerHealthParameters {
                overlap_damage_health_delta: -0.05,
                ..LayerHealthParameters::DEFAULT
            })],
        ))
}
