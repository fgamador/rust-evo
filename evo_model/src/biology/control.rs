use biology::control_requests::*;
use biology::layers::CellLayer;
use physics::quantities::*;
use std::fmt::Debug;

pub trait CellControl: Debug {
    fn box_clone(&self) -> Box<CellControl>;

    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest>;
}

impl Clone for Box<CellControl>
{
    fn clone(&self) -> Box<CellControl> {
        self.box_clone()
    }
}

#[derive(Debug)]
pub struct CellStateSnapshot {
    pub area: Area,
    pub center: Position,
    pub velocity: Velocity,
}

#[derive(Clone, Debug)]
pub struct NullControl {}

impl NullControl {
    pub fn new() -> Self {
        NullControl {}
    }
}

impl CellControl for NullControl {
    fn box_clone(&self) -> Box<CellControl> {
        Box::new(self.clone())
    }

    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![]
    }
}

#[derive(Clone, Debug)]
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
    fn box_clone(&self) -> Box<CellControl> {
        Box::new(self.clone())
    }

    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        self.controls.iter_mut()
            .flat_map(|control| control.get_control_requests(cell_state))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct ContinuousRequestsControl {
    requests: Vec<ControlRequest>
}

impl ContinuousRequestsControl {
    pub fn new(requests: Vec<ControlRequest>) -> Self {
        ContinuousRequestsControl {
            requests
        }
    }
}

impl CellControl for ContinuousRequestsControl {
    fn box_clone(&self) -> Box<CellControl> {
        Box::new(self.clone())
    }

    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        self.requests.clone()
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
    fn box_clone(&self) -> Box<CellControl> {
        Box::new(self.clone())
    }

    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![CellLayer::resize_request(self.layer_index, self.resize_amount)]
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
    fn box_clone(&self) -> Box<CellControl> {
        Box::new(self.clone())
    }

    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![
            ControlRequest::new(self.thruster_layer_index, 2, self.force.x()),
            ControlRequest::new(self.thruster_layer_index, 3, self.force.y()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_clone_control() {
        let control = NullControl::new();
        let _clone = control.clone();
    }

    #[test]
    fn continuous_resize_control_returns_request_to_grow_specified_layer() {
        let cell_state = CellStateSnapshot {
            area: Area::new(0.0),
            center: Position::new(0.0, 0.0),
            velocity: Velocity::new(0.0, 0.0),
        };
        let mut control = ContinuousResizeControl::new(1, AreaDelta::new(0.5));
        let requests = control.get_control_requests(&cell_state);
        assert_eq!(requests, vec![CellLayer::resize_request(1, AreaDelta::new(0.5))]);
    }

    #[test]
    fn simple_thruster_control_returns_requests_for_force() {
        let cell_state = CellStateSnapshot {
            area: Area::new(0.0),
            center: Position::new(0.0, 0.0),
            velocity: Velocity::new(0.0, 0.0),
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
