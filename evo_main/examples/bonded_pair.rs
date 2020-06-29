use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::environment::influences::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_influence(Box::new(BondForces::new()))
        .with_cells(vec![
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-25.0, 0.0),
                Velocity::new(0.0, 1.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(25.0, 0.0),
                Velocity::new(0.0, -1.0),
            ),
        ])
        .with_bonds(vec![(0, 1)])
}
