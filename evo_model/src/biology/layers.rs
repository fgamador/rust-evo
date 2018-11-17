use physics::quantities::*;
use std::f64::consts::PI;

#[derive(Clone, Debug)]
pub struct SimpleCellLayer {
    outer_radius: Length,
    density: Density,
}

impl SimpleCellLayer {
    pub fn new_old(outer_radius: Length, density: Density) -> Self {
        SimpleCellLayer {
            outer_radius,
            density,
        }
    }

    pub fn outer_radius(&self) -> Length {
        self.outer_radius
    }

    pub fn mass(&self) -> Mass {
        PI * self.outer_radius * self.outer_radius * self.density
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_calculates_mass() {
        let layer = SimpleCellLayer::new_old(Length::new(2.0), Density::new(3.0));
        assert_eq!(Mass::new(12.0 * PI), layer.mass());
    }
}
