use evo_domain::biology::cell::Cell;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;

fn main() {
    init_and_run(|_seed| create_world());
}

fn create_world() -> World {
    World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0))
        .with_standard_influences()
        .with_cells(vec![
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-100.0, -90.0),
                Velocity::new(3.0, 2.5),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-90.0, 100.0),
                Velocity::new(2.5, -3.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(100.0, 90.0),
                Velocity::new(-3.0, -2.5),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(90.0, -100.0),
                Velocity::new(-2.5, 3.0),
            ),
        ])
}
