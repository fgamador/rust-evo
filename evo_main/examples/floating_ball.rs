use evo_domain::biology::cell::Cell;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::init_and_run;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_standard_influences()
        .with_per_cell_influence(Box::new(SimpleForceInfluence::new(Box::new(
            BuoyancyForce::new(-0.05, 0.001),
        ))))
        .with_cell(Cell::ball(
            Length::new(20.0),
            Mass::new(1.0),
            Position::new(50.0, -300.0),
            Velocity::new(1.0, 0.0),
        ))
}
