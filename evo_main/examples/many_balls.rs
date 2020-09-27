use evo_domain::biology::cell::Cell;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;

fn main() {
    let args = parse_command_line();
    init_and_run_old(create_world(), args);
}

fn create_world() -> World {
    let mut world = World::new(Position::new(-750.0, -350.0), Position::new(750.0, 350.0))
        .with_standard_influences();
    for i in 0..48 {
        for j in 0..21 {
            world = world.with_cell(Cell::ball(
                Length::new(10.0),
                Mass::new(1.0),
                Position::new(-700.0 + (i * 30) as f64, -300.0 + (j * 30) as f64),
                Velocity::new(2.0, 2.0),
            ));
        }
    }
    world
}
