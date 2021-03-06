use evo_domain::biology::cell::Cell;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;
use std::f64::consts::PI;

// TODO doesn't work yet

fn main() {
    init_and_run(|_seed| create_world());
}

fn create_world() -> World {
    World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0))
        .with_standard_influences()
        .with_cross_cell_influence(Box::new(BondAngleForces::new()))
        .with_cells(vec![
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(0.0, 100.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(0.0, 60.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(0.0, 20.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(0.0, -20.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(0.0, -60.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(0.0, -100.0),
                Velocity::new(0.0, 0.0),
            ),
        ])
        .with_bonds(vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)])
        .with_angle_gussets(vec![(1, 2, PI), (2, 3, PI)])
        .with_cell(Cell::ball(
            Length::new(20.0),
            Mass::new(1.0),
            Position::new(-40.0, 100.0),
            Velocity::new(-3.0, 0.0),
        ))
}
