extern crate evo_main;
extern crate evo_model;
extern crate evo_view_model;

use evo_model::biology::control::*;
use evo_model::biology::layers::*;
use evo_model::biology::cell::Cell;
use evo_model::environment::influences::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_main::main_support::init_and_run;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World<Cell> {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(2.0))))
        ])
        .with_cells(vec![
            Cell::new(
                Position::new(300.0, -300.0), Velocity::new(0.0, 0.0),
                vec![
                    Box::new(ThrusterLayer::new(Area::new(200.0 * PI), Density::new(1.0))),
                ])
                .with_control(Box::new(
                    ThrustInSquareControl::new(0, 70.0, Direction::Left, 100, 200))),
        ])
}
