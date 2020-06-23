use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::physics::quantities::*;
use evo_model::world::World;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(-100.0, -100.0), Position::new(100.0, 100.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_cells(vec![
            Cell::ball(
                Length::new(40.0),
                Mass::new(10.0),
                Position::new(50.0, -50.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(5.0),
                Mass::new(0.5),
                Position::new(-95.0, 75.0),
                Velocity::new(1.21, -1.0),
            ),
        ])
}
