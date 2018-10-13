extern crate evo_main;
extern crate evo_model;

use evo_model::environment::influences::*;
use evo_model::physics::ball::Ball;
use evo_model::physics::bond::AngleGusset;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_main::main_support::init_and_run;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World<Ball> {
    let mut world = World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0))
        .with_perimeter_walls()
        .with_influences(vec![
            Box::new(PairCollisions::new()),
            Box::new(BondForces::new()),
            Box::new(BondAngleForces::new()),
        ])
        .with_balls(vec![
            Ball::new(Length::new(20.0), Mass::new(1.0),
                      Position::new(0.0, 100.0), Velocity::new(0.0, 0.0)),
            Ball::new(Length::new(20.0), Mass::new(1.0),
                      Position::new(0.0, 60.0), Velocity::new(0.0, 0.0)),
            Ball::new(Length::new(20.0), Mass::new(1.0),
                      Position::new(0.0, 20.0), Velocity::new(0.0, 0.0)),
            Ball::new(Length::new(20.0), Mass::new(1.0),
                      Position::new(0.0, -20.0), Velocity::new(0.0, 0.0)),
            Ball::new(Length::new(20.0), Mass::new(1.0),
                      Position::new(0.0, -60.0), Velocity::new(0.0, 0.0)),
            Ball::new(Length::new(20.0), Mass::new(1.0),
                      Position::new(0.0, -100.0), Velocity::new(0.0, 0.0)),
        ])
        .with_bonds(vec![
            (0, 1), (1, 2), (2, 3), (3, 4), (4, 5)
        ]);

    let gusset = AngleGusset::new(&world.bonds()[1], &world.bonds()[2], Angle::from_radians(PI));
    world.add_angle_gusset(gusset);
    let gusset = AngleGusset::new(&world.bonds()[2], &world.bonds()[3], Angle::from_radians(PI));
    world.add_angle_gusset(gusset);

    world.add_ball(Ball::new(Length::new(20.0), Mass::new(1.0),
                             Position::new(-40.0, 100.0), Velocity::new(-3.0, 0.0)));

    world
}
