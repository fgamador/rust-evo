use physics::quantities::*;
use physics::shapes::*;
use std::fmt::Debug;

pub trait CellControl: Debug {
    fn get_resize_requests(&mut self) -> Vec<ResizeRequest>;
}

pub trait ControllableCell: Circle + Debug {}

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
    fn get_resize_requests(&mut self) -> Vec<ResizeRequest> { vec![] }
}

#[derive(Debug)]
pub struct CyclicResizeControl {
    layer_index: usize,
    current_area: Area,
    growth_ticks: u32,
    growth_amount: Area,
    tick_count: u32,
}

impl CyclicResizeControl {
    pub fn new(layer_index: usize, initial_area: Area, growth_ticks: u32, growth_amount: Area) -> Self {
        CyclicResizeControl {
            layer_index,
            current_area: initial_area,
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
            self.current_area += self.growth_amount;
        } else {
            self.current_area -= self.growth_amount;
        }
        vec![ResizeRequest::new(self.layer_index, self.current_area)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyclic_resize_control_returns_request_for_specified_layer_index() {
        let cell = SimpleControllableCell::new(Position::new(0.0, 0.0), Length::new(10.0));
        let mut control = CyclicResizeControl::new(3, Area::new(10.0), 1, Area::new(0.5));
        let reqs = control.get_resize_requests();
        assert_eq!(1, reqs.len());
        assert_eq!(3, reqs[0].layer_index);
    }

    #[test]
    fn cyclic_resize_control_returns_alternating_growth_and_shrink_requests() {
        let mut cell = SimpleControllableCell::new(Position::new(0.0, 0.0), Length::new(10.0));
        let mut control = CyclicResizeControl::new(0, Area::new(10.0), 2, Area::new(0.5));
        assert_eq!(Area::new(10.5), control.get_resize_requests()[0].desired_area);
        assert_eq!(Area::new(11.0), control.get_resize_requests()[0].desired_area);
        assert_eq!(Area::new(10.5), control.get_resize_requests()[0].desired_area);
        assert_eq!(Area::new(10.0), control.get_resize_requests()[0].desired_area);
        assert_eq!(Area::new(10.5), control.get_resize_requests()[0].desired_area);
    }

    #[derive(Debug)]
    struct SimpleControllableCell {
        pub center: Position,
        pub radius: Length,
    }

    impl SimpleControllableCell {
        pub fn new(center: Position, radius: Length) -> Self {
            SimpleControllableCell {
                center,
                radius,
            }
        }
    }

    impl ControllableCell for SimpleControllableCell {}

    impl Circle for SimpleControllableCell {
        fn radius(&self) -> Length {
            return self.radius;
        }

        fn center(&self) -> Position {
            return self.center;
        }
    }
}
