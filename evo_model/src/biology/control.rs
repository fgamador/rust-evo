use biology::control_requests::*;
use physics::quantities::*;
use std::fmt::Debug;

pub trait CellControl: Debug {
    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest>;
}

#[derive(Debug)]
pub struct CellStateSnapshot {
    pub center: Position,
    pub velocity: Velocity,
    pub layers: Vec<CellLayerStateSnapshot>,
}

#[derive(Debug)]
pub struct CellLayerStateSnapshot {
    pub area: Area,
}

#[derive(Debug)]
pub struct NullControl {}

impl NullControl {
    pub fn new() -> Self {
        NullControl {}
    }
}

impl CellControl for NullControl {
    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![]
    }
}

#[derive(Debug)]
pub struct CompositeControl {
    controls: Vec<Box<CellControl>>
}

impl CompositeControl {
    pub fn new(controls: Vec<Box<CellControl>>) -> Self {
        CompositeControl {
            controls
        }
    }
}

impl CellControl for CompositeControl {
    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        self.controls.iter_mut()
            .flat_map(|control| control.get_control_requests(cell_state))
            .collect()
    }
}

#[derive(Debug)]
pub struct ContinuousResizeControl {
    layer_index: usize,
    resize_amount: AreaDelta,
}

impl ContinuousResizeControl {
    pub fn new(layer_index: usize, resize_amount: AreaDelta) -> Self {
        ContinuousResizeControl {
            layer_index,
            resize_amount,
        }
    }
}

impl CellControl for ContinuousResizeControl {
    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![ControlRequest::new(self.layer_index, 1, self.resize_amount.value())]
    }
}

#[derive(Debug)]
pub struct FixedDepthSeekingControl {
    float_layer_index: usize,
    target_y: f64,
}

impl FixedDepthSeekingControl {
    pub fn new(float_layer_index: usize, target_y: f64) -> Self {
        FixedDepthSeekingControl {
            float_layer_index,
            target_y,
        }
    }
}

impl CellControl for FixedDepthSeekingControl {
    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let y_offset = cell_state.center.y() - self.target_y;
        let target_velocity_y = -y_offset / 100.0;
        let target_delta_vy = target_velocity_y - cell_state.velocity.y();
        let desired_delta_area = target_delta_vy * 10.0;
        vec![ControlRequest::new(self.float_layer_index, 1, desired_delta_area)]
    }
}

#[derive(Debug)]
pub struct SimpleThrusterControl {
    thruster_layer_index: usize,
    force: Force,
}

impl SimpleThrusterControl {
    pub fn new(thruster_layer_index: usize, force: Force) -> Self {
        SimpleThrusterControl {
            thruster_layer_index,
            force,
        }
    }
}

impl CellControl for SimpleThrusterControl {
    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![
            ControlRequest::new(self.thruster_layer_index, 2, self.force.x()),
            ControlRequest::new(self.thruster_layer_index, 3, self.force.y()),
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

#[derive(Debug)]
pub struct ThrustInSquareControl {
    thruster_layer_index: usize,
    force: f64,
    accel_ticks: u32,
    ticks_before_turn: u32,
    direction: Direction,
    ticks: u32,
}

impl ThrustInSquareControl {
    pub fn new(thruster_layer_index: usize, force: f64, initial_direction: Direction, accel_ticks: u32, ticks_before_turn: u32) -> Self {
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
    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
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
            ControlRequest::new(self.thruster_layer_index, 2, force.x()),
            ControlRequest::new(self.thruster_layer_index, 3, force.y()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuous_resize_control_returns_request_to_grow_specified_layer() {
        let cell_state = CellStateSnapshot {
            center: Position::new(0.0, 0.0),
            velocity: Velocity::new(0.0, 0.0),
            layers: vec![
                CellLayerStateSnapshot { area: Area::new(1.0) },
                CellLayerStateSnapshot { area: Area::new(2.0) }
            ],
        };
        let mut control = ContinuousResizeControl::new(1, AreaDelta::new(0.5));
        let requests = control.get_control_requests(&cell_state);
        assert_eq!(requests, vec![ControlRequest::new(1, 1, 0.5)]);
    }

    #[test]
    fn simple_thruster_control_returns_requests_for_force() {
        let cell_state = CellStateSnapshot {
            center: Position::new(0.0, 0.0),
            velocity: Velocity::new(0.0, 0.0),
            layers: vec![
                CellLayerStateSnapshot { area: Area::new(10.0) },
                CellLayerStateSnapshot { area: Area::new(10.0) }
            ],
        };
        let mut control = SimpleThrusterControl::new(2, Force::new(1.0, -1.0));
        let requests = control.get_control_requests(&cell_state);
        assert_eq!(requests,
                   vec![
                       ControlRequest::new(2, 2, 1.0),
                       ControlRequest::new(2, 3, -1.0)
                   ]);
    }
}
