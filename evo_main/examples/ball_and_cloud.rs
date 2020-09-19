use evo_domain::biology::cell::Cell;
use evo_domain::biology::cloud::Cloud;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;

fn main() {
    let args = parse_command_line();
    init_and_run(create_world(), args);
}

fn create_world() -> World {
    World::new(Position::new(-100.0, -100.0), Position::new(100.0, 100.0))
        .with_standard_influences()
        .with_cells(vec![Cell::ball(
            Length::new(5.0),
            Mass::new(0.5),
            Position::new(-95.0, 75.0),
            Velocity::new(1.21, -1.0),
        )])
        .with_clouds(vec![Cloud::new(Position::new(0.0, 0.0), Length::new(5.0))])
}
