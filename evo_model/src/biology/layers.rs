use physics::quantities::*;

#[derive(Clone, Debug)]
pub struct SimpleCellLayer {
    outer_radius: Length,
}

impl SimpleCellLayer {
    pub fn new(outer_radius: Length) -> Self {
        SimpleCellLayer {
            outer_radius,
        }
    }

    pub fn outer_radius(&self) -> Length {
        self.outer_radius
    }
}
