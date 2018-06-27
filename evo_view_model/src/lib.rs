pub mod events;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Event {
    Rendered,
    Updated,
}

pub struct Circle {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

pub struct ViewModel {
    pub circle: Circle,
}

impl ViewModel {
    pub fn new() -> Self {
        ViewModel {
            circle: Circle {
                x: 0.0,
                y: 0.0,
                radius: 10.0,
            }
        }
    }
}
