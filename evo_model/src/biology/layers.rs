use physics::quantities::*;

#[derive(Clone, Debug)]
pub struct SimpleCellLayer {
    radius: Length,
}

impl SimpleCellLayer {
    pub fn new(radius: Length) -> Self {
        SimpleCellLayer {
            radius,
        }
    }

    pub fn radius(&self) -> Length {
        self.radius
    }
}
