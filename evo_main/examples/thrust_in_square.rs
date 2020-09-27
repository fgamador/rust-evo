use evo_domain::biology::cell::Cell;
use evo_domain::biology::control::*;
use evo_domain::biology::control_requests::*;
use evo_domain::biology::layers::*;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;
use std::f64::consts::PI;

fn main() {
    let args = parse_command_line();
    init_and_run_old(create_world(), args);
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_standard_influences()
        .with_per_cell_influences(vec![Box::new(SimpleForceInfluence::new(Box::new(
            DragForce::new(2.0),
        )))])
        .with_cells(vec![Cell::new(
            Position::new(300.0, -300.0),
            Velocity::new(0.0, 0.0),
            vec![CellLayer::new(
                Area::new(200.0 * PI),
                Density::new(1.0),
                Tissue::Photosynthetic,
                Box::new(ThrusterCellLayerSpecialty::new()),
            )],
        )
        .with_control(Box::new(ThrustInSquareControl::new(
            0,
            70.0,
            Direction::Left,
            100,
            200,
        )))])
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

#[derive(Clone, Debug)]
pub struct ThrustInSquareControl {
    thruster_layer_index: usize,
    force: f64,
    accel_ticks: u32,
    ticks_before_turn: u32,
    direction: Direction,
    ticks: u32,
}

impl ThrustInSquareControl {
    pub fn new(
        thruster_layer_index: usize,
        force: f64,
        initial_direction: Direction,
        accel_ticks: u32,
        ticks_before_turn: u32,
    ) -> Self {
        ThrustInSquareControl {
            thruster_layer_index,
            force,
            accel_ticks,
            ticks_before_turn,
            direction: initial_direction,
            ticks: 0,
        }
    }

    fn turn(direction: Direction) -> Direction {
        match direction {
            Direction::Left => Direction::Up,
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
        }
    }

    fn calc_force(magnitude: f64, direction: Direction) -> Force {
        match direction {
            Direction::Left => Force::new(-magnitude, 0.0),
            Direction::Up => Force::new(0.0, magnitude),
            Direction::Right => Force::new(magnitude, 0.0),
            Direction::Down => Force::new(0.0, -magnitude),
        }
    }
}

impl CellControl for ThrustInSquareControl {
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let force = if self.ticks < self.accel_ticks {
            Self::calc_force(self.force, self.direction)
        } else {
            Force::new(0.0, 0.0)
        };

        self.ticks += 1;
        if self.ticks >= self.ticks_before_turn {
            self.ticks = 0;
            self.direction = Self::turn(self.direction);
        }

        vec![
            ThrusterCellLayerSpecialty::force_x_request(self.thruster_layer_index, force.x()),
            ThrusterCellLayerSpecialty::force_y_request(self.thruster_layer_index, force.y()),
        ]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}
