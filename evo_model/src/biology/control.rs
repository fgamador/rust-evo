use std::fmt::Debug;

pub trait CellControl: Debug {
    fn get_resize_requests(&mut self) -> Vec<ResizeRequest>;
}

pub struct ResizeRequest {
    pub layer_index: usize,
    pub desired_area: f64,
}

impl ResizeRequest {
    pub fn new(layer_index: usize, desired_area: f64) -> Self {
        ResizeRequest {
            layer_index,
            desired_area: desired_area,
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
    fn get_resize_requests(&mut self) -> Vec<ResizeRequest> { vec![] }
}

#[derive(Debug)]
pub struct CyclicResizeControl {
    layer_index: usize,
    growth_ticks: u32,
    growth_amount: f64,
    tick_count: u32,
}

impl CyclicResizeControl {
    pub fn new(layer_index: usize, growth_ticks: u32, growth_amount: f64) -> Self {
        CyclicResizeControl {
            layer_index,
            growth_ticks,
            growth_amount,
            tick_count: 0,
        }
    }
}

impl CellControl for CyclicResizeControl {
    fn get_resize_requests(&mut self) -> Vec<ResizeRequest> {
        self.tick_count += 1;
        if self.tick_count > self.growth_ticks * 2 {
            self.tick_count = 1;
        }
        if self.tick_count <= self.growth_ticks {
            vec![ResizeRequest::new(self.layer_index, self.growth_amount)]
        } else {
            vec![ResizeRequest::new(self.layer_index, -self.growth_amount)]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyclic_resize_control_returns_request_for_specified_layer_index() {
        let mut control = CyclicResizeControl::new(3, 1, 0.5);
        let reqs = control.get_resize_requests();
        assert_eq!(1, reqs.len());
        assert_eq!(3, reqs[0].layer_index);
    }

    #[test]
    fn cyclic_resize_control_returns_alternating_growth_and_shrink_requests() {
        let mut control = CyclicResizeControl::new(0, 2, 0.5);
        assert_eq!(0.5, control.get_resize_requests()[0].desired_area);
        assert_eq!(0.5, control.get_resize_requests()[0].desired_area);
        assert_eq!(-0.5, control.get_resize_requests()[0].desired_area);
        assert_eq!(-0.5, control.get_resize_requests()[0].desired_area);
        assert_eq!(0.5, control.get_resize_requests()[0].desired_area);
    }
}
