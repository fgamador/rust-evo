use crate::biology::control_requests::*;
use crate::biology::layers::CellLayer;
use crate::physics::quantities::*;
use std::fmt::Debug;

pub trait CellControl: Debug {
    fn run(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest>;

    fn spawn(&mut self) -> Box<dyn CellControl>;
}

#[derive(Debug)]
pub struct CellStateSnapshot {
    pub radius: Length,
    pub area: Area,
    pub mass: Mass,
    pub center: Position,
    pub velocity: Velocity,
    pub energy: BioEnergy,
    pub layers: Vec<CellLayerStateSnapshot>,
}

impl CellStateSnapshot {
    pub const ZEROS: CellStateSnapshot = CellStateSnapshot {
        radius: Length::ZERO,
        area: Area::ZERO,
        mass: Mass::ZERO,
        center: Position::ORIGIN,
        velocity: Velocity::ZERO,
        energy: BioEnergy::ZERO,
        layers: Vec::new(),
    };
}

#[derive(Debug)]
pub struct CellLayerStateSnapshot {
    pub area: Area,
    pub mass: Mass,
    pub health: Health,
}

#[derive(Debug)]
pub struct NullControl {}

impl NullControl {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        NullControl {}
    }
}

impl CellControl for NullControl {
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(Self::new())
    }
}

#[derive(Clone, Debug)]
pub struct ContinuousRequestsControl {
    requests: Vec<ControlRequest>,
}

impl ContinuousRequestsControl {
    pub fn new(requests: Vec<ControlRequest>) -> Self {
        ContinuousRequestsControl { requests }
    }
}

impl CellControl for ContinuousRequestsControl {
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        self.requests.clone()
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
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
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![CellLayer::resize_request(
            self.layer_index,
            self.resize_amount,
        )]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
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
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![
            ControlRequest::new(self.thruster_layer_index, 2, 0, self.force.x()),
            ControlRequest::new(self.thruster_layer_index, 3, 0, self.force.y()),
        ]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuous_resize_control_returns_request_to_grow_specified_layer() {
        let mut control = ContinuousResizeControl::new(1, AreaDelta::new(0.5));
        let requests = control.run(&CellStateSnapshot::ZEROS);
        assert_eq!(
            requests,
            vec![CellLayer::resize_request(1, AreaDelta::new(0.5))]
        );
    }

    #[test]
    fn simple_thruster_control_returns_requests_for_force() {
        let mut control = SimpleThrusterControl::new(2, Force::new(1.0, -1.0));
        let requests = control.run(&CellStateSnapshot::ZEROS);
        assert_eq!(
            requests,
            vec![
                ControlRequest::new(2, 2, 0, 1.0),
                ControlRequest::new(2, 3, 0, -1.0)
            ]
        );
    }
}
