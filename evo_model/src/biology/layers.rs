use physics::quantities::*;
use std::f64::consts::PI;

#[derive(Clone, Debug)]
pub struct SimpleCellLayer {
    outer_radius: Length,
    density: Density,
}

impl SimpleCellLayer {
    pub fn new(outer_radius: Length, density: Density) -> Self {
        SimpleCellLayer {
            outer_radius,
            density,
        }
    }

    pub fn outer_radius(&self) -> Length {
        self.outer_radius
    }

    pub fn mass(&self) -> Mass {
        self.outer_radius * self.outer_radius * PI * self.density
    }
}
