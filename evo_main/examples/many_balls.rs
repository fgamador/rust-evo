use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::physics::quantities::*;
use evo_model::world::World;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    let mut world = World::new(Position::new(-750.0, -350.0), Position::new(750.0, 350.0))
        .with_perimeter_walls()
        .with_pair_collisions();
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
