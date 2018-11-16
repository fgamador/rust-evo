extern crate evo_main;
extern crate evo_model;

use evo_model::biology::cell::Cell;
use evo_model::environment::influences::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_main::main_support::init_and_run;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World<Cell> {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(-0.05)))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(-0.05, 0.001)))),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.0005))))
        ])
        .with_cell(Cell::new_old(Length::new(20.0), Mass::new(1.0),
                                 Position::new(200.0, -200.0), Velocity::new(0.0, 0.0)))
}
