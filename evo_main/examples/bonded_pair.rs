use evo_domain::biology::cell::Cell;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;

fn main() {
    init_and_run(|_seed| create_world());
}

fn create_world() -> World {
    World::new(Position::new(-200.0, -200.0), Position::new(200.0, 200.0))
        .with_standard_influences()
        .with_cells(vec![
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-25.0, 140.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(25.0, 140.0),
                Velocity::new(0.0, 0.0),
            ),
        ])
        .with_bonds(vec![(0, 1)])
        .with_cells(vec![
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-160.0, 45.0),
                Velocity::new(10.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(0.0, 45.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(40.0, 45.0),
                Velocity::new(0.0, 0.0),
            ),
        ])
        .with_bonds(vec![(3, 4)])
        .with_cells(vec![
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-160.0, -45.0),
                Velocity::new(4.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-20.0, -45.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(20.0, -45.0),
                Velocity::new(0.0, 0.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(60.0, -45.0),
                Velocity::new(0.0, 0.0),
            ),
        ])
        .with_bonds(vec![(6, 7)])
        .with_cells(vec![
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(-20.0, -140.0),
                Velocity::new(0.0, 1.0),
            ),
            Cell::ball(
                Length::new(20.0),
                Mass::new(1.0),
                Position::new(20.0, -140.0),
                Velocity::new(0.0, -1.0),
            ),
        ])
        .with_bonds(vec![(9, 10)])
}
