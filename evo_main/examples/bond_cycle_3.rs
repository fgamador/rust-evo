use evo_domain::biology::cell::Cell;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::init_and_run;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    let offset = 40.0_f64 / (2.0_f64).sqrt();
    World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_cross_cell_influence(Box::new(BondForces::new()))
        .with_cells(vec![
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-offset, offset),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(0.0, 0.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-offset, -offset),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(5.0),
                Mass::new(10.0),
                Position::new(-100.0, -100.0),
                Velocity::new(10.0, 2.0),
            ),
        ])
        .with_bonds(vec![(0, 1), (1, 2), (2, 0)])
}
