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

fn create_world() -> World {
    World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0))
        .with_perimeter_walls()
        .with_influences(vec![
            Box::new(PairCollisions::new()),
            Box::new(BondForces::new())
        ])
        .with_cells(vec![
            Cell::ball(Length::new(20.0), Mass::new(1.0),
                       Position::new(0.0, 100.0), Velocity::new(0.0, 0.0)),
            Cell::ball(Length::new(20.0), Mass::new(1.0),
                       Position::new(0.0, 60.0), Velocity::new(0.0, 0.0)),
            Cell::ball(Length::new(20.0), Mass::new(1.0),
                       Position::new(0.0, 20.0), Velocity::new(0.0, 0.0)),
            Cell::ball(Length::new(20.0), Mass::new(1.0),
                       Position::new(0.0, -20.0), Velocity::new(0.0, 0.0)),
            Cell::ball(Length::new(20.0), Mass::new(1.0),
                       Position::new(0.0, -60.0), Velocity::new(0.0, 0.0)),
            Cell::ball(Length::new(20.0), Mass::new(1.0),
                       Position::new(0.0, -100.0), Velocity::new(0.0, 0.0)),
        ])
        .with_bonds(vec![
            (0, 1), (1, 2), (2, 3), (3, 4), (4, 5)
        ])
        .with_cell(
            Cell::ball(Length::new(20.0), Mass::new(1.0),
                       Position::new(-40.0, 100.0), Velocity::new(-3.0, 0.0))
        )
}
