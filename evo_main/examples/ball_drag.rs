extern crate evo_main;
extern crate evo_model;

use evo_model::environment::influences::*;
use evo_model::physics::ball::Ball;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_main::main_support::init_and_run;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World<Ball> {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_influence(Box::new(Drag::new(0.0005)))
        .with_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(50.0, -200.0), Velocity::new(10.0, 1.0)))
}
