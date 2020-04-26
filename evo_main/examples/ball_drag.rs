use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::environment::influences::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_influence(Box::new(SimpleForceInfluence::new(Box::new(
            DragForce::new(0.0005),
        ))))
        .with_cell(Cell::ball(
            Length::new(20.0),
            Mass::new(1.0),
            Position::new(50.0, -200.0),
            Velocity::new(10.0, 1.0),
        ))
}
