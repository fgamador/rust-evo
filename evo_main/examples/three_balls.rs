use evo_domain::biology::cell::Cell;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;

fn main() {
    let args = parse_command_line();
    init_and_run_old(create_world(), args);
}

fn create_world() -> World {
    World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0))
        .with_standard_influences()
        .with_cells(vec![
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-20.0, 0.0),
                Velocity::ZERO,
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(20.0, 0.0),
                Velocity::ZERO,
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(0.0, 150.0),
                Velocity::new(0.0, -1.0),
            ),
        ])
}
