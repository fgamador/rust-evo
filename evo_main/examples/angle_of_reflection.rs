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
                Length::new(5.0),
                Mass::new(1.0),
                Position::new(-120.0, 160.0),
                Velocity::new(1.5, -2.0),
            ),
            Cell::ball(
                Length::new(14.2),
                Mass::new(100.0),
                Position::new(10.0, -10.0),
                Velocity::ZERO,
            ),
        ])
}
