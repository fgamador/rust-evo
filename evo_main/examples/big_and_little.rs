use evo_domain::biology::cell::Cell;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::init_and_run;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(-100.0, -100.0), Position::new(100.0, 100.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_influence(Box::new(BondForces::new()))
        .with_cells(vec![
            Cell::ball(
                Length::new(5.0),
                Mass::new(0.5),
                Position::new(-95.0, 75.0),
                Velocity::new(1.21, -1.0),
            ),
            Cell::ball(
                Length::new(40.0),
                Mass::new(10.0),
                Position::new(50.0, -50.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(5.0),
                Mass::new(0.5),
                Position::new(50.0, -5.0),
                Velocity::new(0.0, 0.0),
            ),
        ])
        .with_bonds(vec![(1, 2)])
}
