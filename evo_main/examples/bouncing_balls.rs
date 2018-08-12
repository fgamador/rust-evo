extern crate evo_main;
extern crate evo_model;

use evo_model::physics::ball::Ball;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_main::main_support::init_and_run;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    let mut world = World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0));
    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(-100.0, -90.0), Velocity::new(3.0, 2.5)));
    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(-90.0, 100.0), Velocity::new(2.5, -3.0)));
    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(100.0, 90.0), Velocity::new(-3.0, -2.5)));
    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(90.0, -100.0), Velocity::new(-2.5, 3.0)));
    world
}
