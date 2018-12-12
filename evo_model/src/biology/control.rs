use physics::quantities::*;
use std::fmt::Debug;

pub trait CellControl: Debug {
    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> { vec![] }

    fn get_resize_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ResizeRequest> { vec![] }
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

#[derive(Debug, PartialEq)]
pub struct ControlRequest {
    pub layer_index: usize,
    pub input_index: usize,
    pub input_value: f64,
}

impl ControlRequest {
    pub fn new(layer_index: usize, input_index: usize, input_value: f64) -> Self {
        ControlRequest {
            layer_index,
            input_index,
            input_value,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ResizeRequest {
    pub layer_index: usize,
    pub desired_area: Area,
}

impl ResizeRequest {
    pub fn new(layer_index: usize, desired_area: Area) -> Self {
        ResizeRequest {
            layer_index,
            desired_area,
        }
    }
}

#[derive(Debug)]
pub struct NullControl {}

impl NullControl {
    pub fn new() -> Self {
        NullControl {}
    }
}

impl CellControl for NullControl {}

#[derive(Debug)]
pub struct SimpleGrowthControl {
    layer_index: usize,
    growth_amount: Area,
}

impl SimpleGrowthControl {
    pub fn new(layer_index: usize, growth_amount: Area) -> Self {
        SimpleGrowthControl {
            layer_index,
            growth_amount,
        }
    }
}

impl CellControl for SimpleGrowthControl {
    fn get_resize_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ResizeRequest> {
        let desired_area = cell_state.layers[self.layer_index].area + self.growth_amount;
        vec![ResizeRequest::new(self.layer_index, desired_area)]
    }
}

#[derive(Debug)]
pub struct CyclicResizeControl {
    layer_index: usize,
    growth_ticks: u32,
    growth_amount: Area,
    tick_count: u32,
}

impl CyclicResizeControl {
    pub fn new(layer_index: usize, growth_ticks: u32, growth_amount: Area) -> Self {
        CyclicResizeControl {
            layer_index,
            growth_ticks,
            growth_amount,
            tick_count: 0,
        }
    }
}

impl CellControl for CyclicResizeControl {
    fn get_resize_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ResizeRequest> {
        self.tick_count += 1;
        if self.tick_count > self.growth_ticks * 2 {
            self.tick_count = 1;
        }
        let desired_area =
            if self.tick_count <= self.growth_ticks {
                cell_state.layers[self.layer_index].area + self.growth_amount
            } else {
                cell_state.layers[self.layer_index].area - self.growth_amount
            };
        vec![ResizeRequest::new(self.layer_index, desired_area)]
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
    fn get_resize_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ResizeRequest> {
        let y_offset = cell_state.center.y() - self.target_y;
        let target_velocity_y = -y_offset / 100.0;
        let target_delta_vy = target_velocity_y - cell_state.velocity.y();
        let desired_delta_area = target_delta_vy * 10.0;
        let current_area = cell_state.layers[self.float_layer_index].area;
        let desired_area = Area::new((current_area.value() + desired_delta_area).max(0.0));
        vec![ResizeRequest::new(self.float_layer_index, desired_area)]
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
            ControlRequest::new(self.thruster_layer_index, 0, self.force.x()),
            ControlRequest::new(self.thruster_layer_index, 1, self.force.y()),
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
    ticks_before_turn: u32,
    direction: Direction,
    ticks: u32,
}

impl ThrustInSquareControl {
    pub fn new(thruster_layer_index: usize, force: f64, initial_direction: Direction, ticks_before_turn: u32) -> Self {
        ThrustInSquareControl {
            thruster_layer_index,
            force,
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
        self.ticks += 1;
        if self.ticks >= self.ticks_before_turn {
            self.ticks = 0;
            self.direction = Self::turn(self.direction);
        }
        let force = Self::calc_force(self.force, self.direction);
        vec![
            ControlRequest::new(self.thruster_layer_index, 0, force.x()),
            ControlRequest::new(self.thruster_layer_index, 1, force.y()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_thruster_control_returns_control_requests_for_force() {
        let cell_state = CellStateSnapshot {
            center: Position::new(0.0, 0.0),
            velocity: Velocity::new(0.0, 0.0),
            layers: vec![
                CellLayerStateSnapshot { area: Area::new(10.0) },
                CellLayerStateSnapshot { area: Area::new(10.0) }
            ],
        };
        let mut control = SimpleThrusterControl::new(2, Force::new(1.0, -1.0));
        let reqs = control.get_control_requests(&cell_state);
        assert_eq!(reqs,
                   vec![
                       ControlRequest::new(2, 0, 1.0),
                       ControlRequest::new(2, 1, -1.0)
                   ]);
    }

    #[test]
    fn cyclic_resize_control_returns_request_for_specified_layer_index() {
        let cell_state = CellStateSnapshot {
            center: Position::new(0.0, 0.0),
            velocity: Velocity::new(0.0, 0.0),
            layers: vec![
                CellLayerStateSnapshot { area: Area::new(10.0) },
                CellLayerStateSnapshot { area: Area::new(10.0) }
            ],
        };
        let mut control = CyclicResizeControl::new(1, 1, Area::new(0.5));
        let reqs = control.get_resize_requests(&cell_state);
        assert_eq!(1, reqs.len());
        assert_eq!(1, reqs[0].layer_index);
    }

    #[test]
    fn cyclic_resize_control_returns_alternating_growth_and_shrink_requests() {
        let cell_state = CellStateSnapshot {
            center: Position::new(0.0, 0.0),
            velocity: Velocity::new(0.0, 0.0),
            layers: vec![
                CellLayerStateSnapshot { area: Area::new(1.0) },
                CellLayerStateSnapshot { area: Area::new(10.0) }
            ],
        };
        let mut control = CyclicResizeControl::new(1, 2, Area::new(0.5));
        assert_eq!(Area::new(10.5), control.get_resize_requests(&cell_state)[0].desired_area);
        assert_eq!(Area::new(10.5), control.get_resize_requests(&cell_state)[0].desired_area);
        assert_eq!(Area::new(9.5), control.get_resize_requests(&cell_state)[0].desired_area);
        assert_eq!(Area::new(9.5), control.get_resize_requests(&cell_state)[0].desired_area);
        assert_eq!(Area::new(10.5), control.get_resize_requests(&cell_state)[0].desired_area);
    }
}
