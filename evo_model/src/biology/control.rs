use physics::quantities::*;
use std::fmt::Debug;

pub trait CellControl: Debug {
    fn get_resize_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ResizeRequest>;
}

#[derive(Debug)]
pub struct CellStateSnapshot {
    pub layers: Vec<CellLayerStateSnapshot>,
}

#[derive(Debug)]
pub struct CellLayerStateSnapshot {
    pub area: Area,
}

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

impl CellControl for NullControl {
    fn get_resize_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ResizeRequest> { vec![] }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyclic_resize_control_returns_request_for_specified_layer_index() {
        let cell_state = CellStateSnapshot {
            layers: vec![
                CellLayerStateSnapshot { area: Area::new(10.0) },
                CellLayerStateSnapshot { area: Area::new(10.0) }
            ]
        };
        let mut control = CyclicResizeControl::new(1, 1, Area::new(0.5));
        let reqs = control.get_resize_requests(&cell_state);
        assert_eq!(1, reqs.len());
        assert_eq!(1, reqs[0].layer_index);
    }

    #[test]
    fn cyclic_resize_control_returns_alternating_growth_and_shrink_requests() {
        let cell_state = CellStateSnapshot {
            layers: vec![
                CellLayerStateSnapshot { area: Area::new(1.0) },
                CellLayerStateSnapshot { area: Area::new(10.0) }
            ]
        };
        let mut control = CyclicResizeControl::new(1, 2, Area::new(0.5));
        assert_eq!(Area::new(10.5), control.get_resize_requests(&cell_state)[0].desired_area);
        assert_eq!(Area::new(10.5), control.get_resize_requests(&cell_state)[0].desired_area);
        assert_eq!(Area::new(9.5), control.get_resize_requests(&cell_state)[0].desired_area);
        assert_eq!(Area::new(9.5), control.get_resize_requests(&cell_state)[0].desired_area);
        assert_eq!(Area::new(10.5), control.get_resize_requests(&cell_state)[0].desired_area);
    }
}
