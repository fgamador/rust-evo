extern crate evo_glium;
extern crate evo_model;
extern crate evo_view_model;

pub mod main_support;
pub mod mvvm;

use evo_model::biology::cell::Cell;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use main_support::init_and_run;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0))
        .with_standard_influences()
        .with_cells(vec![
            Cell::ball(Length::new(20.0), Mass::new(1.0),
                       Position::new(-100.0, -90.0), Velocity::new(3.0, 2.5))
        ])
}
