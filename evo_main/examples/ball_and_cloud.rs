use evo_domain::biology::cell::Cell;
use evo_domain::biology::cloud::{Cloud, CloudParameters};
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_domain::Parameters;
use evo_main::main_support::*;

fn main() {
    init_and_run(|_seed| create_world());
}

fn create_world() -> World {
    let parameters = Parameters {
        cloud_params: CloudParameters {
            resize_factor: Positive::new(1.01),
            minimum_concentration: Fraction::new(0.1),
        },
        ..Parameters::DEFAULT
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
