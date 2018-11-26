use std::fmt::Debug;

pub trait CellControl: Debug {
    fn get_resize_requests(&mut self) -> Vec<(usize, f64)>;
}

#[derive(Debug)]
pub struct NullControl {}

impl NullControl {
    pub fn new() -> Self {
        NullControl {}
    }
}

impl CellControl for NullControl {
    fn get_resize_requests(&mut self) -> Vec<(usize, f64)> { vec![] }
}

#[derive(Debug)]
pub struct CyclicResizeControl {
    layer_index: usize,
    growth_ticks: u32,
    growth_fraction: f64,
    tick_count: u32,
}

impl CyclicResizeControl {
    pub fn new(layer_index: usize, growth_ticks: u32, growth_fraction: f64) -> Self {
        CyclicResizeControl {
            layer_index,
            growth_ticks,
            growth_fraction,
            tick_count: 0,
        }
    }
}

impl CellControl for CyclicResizeControl {
    fn get_resize_requests(&mut self) -> Vec<(usize, f64)> {
        self.tick_count += 1;
        if self.tick_count > self.growth_ticks * 2 {
            self.tick_count = 1;
        }
        if self.tick_count <= self.growth_ticks {
            vec![(self.layer_index, self.growth_fraction)]
        } else {
            vec![(self.layer_index, -self.growth_fraction)]
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
        assert_eq!(3, reqs[0].0);
    }

    #[test]
    fn cyclic_resize_control_returns_alternating_growth_and_shrink_requests() {
        let mut control = CyclicResizeControl::new(0, 2, 0.5);
        assert_eq!(0.5, control.get_resize_requests()[0].1);
        assert_eq!(0.5, control.get_resize_requests()[0].1);
        assert_eq!(-0.5, control.get_resize_requests()[0].1);
        assert_eq!(-0.5, control.get_resize_requests()[0].1);
        assert_eq!(0.5, control.get_resize_requests()[0].1);
    }
}
