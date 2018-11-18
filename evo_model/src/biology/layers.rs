use evo_view_model::Color;
use physics::quantities::*;
use physics::shapes::Circle;
use physics::util::sqr;
use std::f64::consts::PI;
use std::fmt::Debug;

pub trait Onion: Circle {
    fn layers(&self) -> &[Box<OnionLayer>];
}

pub trait OnionLayer: Debug {
    fn outer_radius(&self) -> Length;

    fn color(&self) -> Color;
}

#[derive(Debug)]
pub struct SimpleOnionLayer {
    outer_radius: Length,
    color: Color,
}

impl SimpleOnionLayer {
    pub fn new(outer_radius: Length, color: Color) -> Self {
        SimpleOnionLayer {
            outer_radius,
            color,
        }
    }
}

impl OnionLayer for SimpleOnionLayer {
    fn outer_radius(&self) -> Length {
        self.outer_radius
    }

    fn color(&self) -> Color {
        self.color
    }
}

pub trait CellLayer: OnionLayer {
    fn mass(&self) -> Mass;

    fn update_outer_radius(&mut self, inner_radius: Length);
}

#[derive(Debug)]
pub struct SimpleCellLayer {
    area: Area,
    density: Density,
    mass: Mass,
    outer_radius: Length,
    color: Color,
}

impl SimpleCellLayer {
    pub fn new(area: Area, density: Density) -> Self {
        SimpleCellLayer {
            area,
            density,
            mass: area * density,
            outer_radius: Length::new((area.value() / PI).sqrt()),
            color: Color::Green, // TODO
        }
    }
}

impl OnionLayer for SimpleCellLayer {
    fn outer_radius(&self) -> Length {
        self.outer_radius
    }

    fn color(&self) -> Color {
        self.color
    }
}

impl CellLayer for SimpleCellLayer {
    fn mass(&self) -> Mass {
        self.mass
    }

    fn update_outer_radius(&mut self, inner_radius: Length) {
        self.outer_radius = Length::new((sqr(inner_radius.value()) + self.area.value() / PI).sqrt());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_calculates_mass() {
        let layer = SimpleCellLayer::new(Area::new(2.0 * PI), Density::new(3.0));
        assert_eq!(Mass::new(6.0 * PI), layer.mass());
    }

    #[test]
    fn single_layer_calculates_outer_radius() {
        let layer = SimpleCellLayer::new(Area::new(4.0 * PI), Density::new(1.0));
        assert_eq!(Length::new(2.0), layer.outer_radius());
    }

    #[test]
    fn layer_updates_outer_radius_based_on_inner_radius() {
        let mut layer = SimpleCellLayer::new(Area::new(3.0 * PI), Density::new(1.0));
        layer.update_outer_radius(Length::new(1.0));
        assert_eq!(Length::new(2.0), layer.outer_radius());
    }
}
