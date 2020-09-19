use evo_domain::biology::cell::Cell;
use evo_domain::biology::cloud::{Cloud, CloudParameters};
use evo_domain::physics::quantities::*;
use evo_domain::world::{Parameters, World};
use evo_main::main_support::*;

fn main() {
    let args = parse_command_line();
    init_and_run(create_world(), args);
}

fn create_world() -> World {
    let parameters = Parameters {
        cloud_params: CloudParameters {
            resize_factor: 1.01,
        },
    };
    World::new(Position::new(-100.0, -100.0), Position::new(100.0, 100.0))
        .with_parameters(parameters)
        .with_standard_influences()
        .with_cells(vec![Cell::ball(
            Length::new(5.0),
            Mass::new(1.0),
            Position::new(-5.0, 5.0),
            Velocity::ZERO,
        )])
        .with_clouds(vec![Cloud::new(Position::new(0.0, 0.0), Length::new(5.0))])
}
