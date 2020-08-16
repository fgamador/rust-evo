use evo_domain::biology::cell::Cell;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;

fn main() {
    let args = parse_command_line();
    init_and_run(create_world(), args);
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
                Position::new(-60.0, -90.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(100.0, -90.0),
                Velocity::new(-3.0, 2.5),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(60.0, -90.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-100.0, 90.0),
                Velocity::new(3.0, -2.5),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-60.0, 90.0),
                Velocity::new(0.0, 0.0),
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
                Position::new(60.0, 90.0),
                Velocity::new(0.0, 0.0),
            ),
        ])
        .with_bonds(vec![(0, 1), (2, 3), (4, 5), (6, 7)])
}
