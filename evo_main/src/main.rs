extern crate evo_conrod;
extern crate evo_model;
extern crate evo_view_model;

pub mod main_support;
pub mod mvvm;

use evo_model::physics::ball::Ball;
use evo_model::physics::bond::Bond;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use main_support::init_and_run;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World<Ball> {
    let mut world = World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0));

    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(-100.0, -90.0), Velocity::new(3.0, 2.5)));
    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(-60.0, -90.0), Velocity::new(0.0, 0.0)));
    let bond = Bond::new(&world.balls()[0], &world.balls()[1]);
    world.add_bond(bond);

    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(100.0, 90.0), Velocity::new(-3.0, -2.5)));
    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(60.0, 90.0), Velocity::new(0.0, 0.0)));
    let bond = Bond::new(&world.balls()[2], &world.balls()[3]);
    world.add_bond(bond);

    world
}
