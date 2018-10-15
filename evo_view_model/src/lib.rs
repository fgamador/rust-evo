pub mod events;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Event {
    Rendered,
    Updated,
}

#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

pub struct ViewModel {
    pub circles: Vec<Circle>,
}

impl ViewModel {
    pub fn new() -> Self {
        ViewModel {
            circles: vec![],
        }
    }
}

pub struct CoordinateTransform {}

impl CoordinateTransform {
    pub fn new() -> Self {
        CoordinateTransform {}
    }

    pub fn transform_x(&self, input_x: f64) -> f64 {
        input_x
    }

    pub fn transform_y(&self, input_y: f64) -> f64 {
        input_y
    }

    pub fn transform_length(&self, input_length: f64) -> f64 {
        input_length
    }
}
