use evo_view_model::Color;
use physics::newtonian::Forces;
use physics::quantities::*;
use physics::shapes::Circle;
use std::f64::consts::PI;
use std::fmt::Debug;

pub trait Onion: Circle {
    type Layer: OnionLayer + ?Sized;

    fn layers(&self) -> &[Box<Self::Layer>];
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
    fn area(&self) -> Area;

    fn mass(&self) -> Mass;

    fn update_outer_radius(&mut self, inner_radius: Length);

    fn control_input(&mut self, _index: usize, _value: f64) {}

    fn after_influences(&mut self, _forces: &mut Forces) {}

    fn resize(&mut self, new_area: Area);
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
    pub fn new(area: Area, density: Density, color: Color) -> Self {
        SimpleCellLayer {
            area,
            density,
            mass: area * density,
            outer_radius: (area / PI).sqrt(),
            color,
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
    fn area(&self) -> Area {
        self.area
    }

    fn mass(&self) -> Mass {
        self.mass
    }

    fn update_outer_radius(&mut self, inner_radius: Length) {
        self.outer_radius = (inner_radius.sqr() + self.area / PI).sqrt();
    }

    fn resize(&mut self, new_area: Area) {
        self.area = new_area;
        self.mass = self.area * self.density;
    }
}

#[derive(Debug)]
pub struct ThrusterLayer {
    area: Area,
    density: Density,
    mass: Mass,
    outer_radius: Length,
    color: Color,
    force_x: f64,
    force_y: f64,
}

impl ThrusterLayer {
    pub fn new(area: Area) -> Self {
        let density = Density::new(1.0); // TODO
        ThrusterLayer {
            area,
            density,
            mass: area * density,
            outer_radius: (area / PI).sqrt(),
            color: Color::Green, // TODO
            force_x: 0.0,
            force_y: 0.0,
        }
    }
}

impl OnionLayer for ThrusterLayer {
    fn outer_radius(&self) -> Length {
        self.outer_radius
    }

    fn color(&self) -> Color {
        self.color
    }
}

impl CellLayer for ThrusterLayer {
    fn area(&self) -> Area {
        self.area
    }

    fn mass(&self) -> Mass {
        self.mass
    }

    fn update_outer_radius(&mut self, inner_radius: Length) {
        self.outer_radius = (inner_radius.sqr() + self.area / PI).sqrt();
    }

    fn control_input(&mut self, index: usize, value: f64) {
        match index {
            0 => self.force_x = value,
            1 => self.force_y = value,
            _ => panic!("Invalid control input index: {}", index)
        }
    }

    fn after_influences(&mut self, forces: &mut Forces) {
        forces.add_force(Force::new(self.force_x, self.force_y));
    }

    fn resize(&mut self, new_area: Area) {
        self.area = new_area;
        self.mass = self.area * self.density;
    }
}

pub trait LayerCellAPI {
    fn forces(&self) -> &Forces;

    fn forces_mut(&mut self) -> &mut Forces;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_calculates_mass() {
        let layer = SimpleCellLayer::new(
            Area::new(2.0 * PI), Density::new(3.0), Color::Green);
        assert_eq!(Mass::new(6.0 * PI), layer.mass());
    }

    #[test]
    fn single_layer_calculates_outer_radius() {
        let layer = SimpleCellLayer::new(
            Area::new(4.0 * PI), Density::new(1.0), Color::Green);
        assert_eq!(Length::new(2.0), layer.outer_radius());
    }

    #[test]
    fn layer_updates_outer_radius_based_on_inner_radius() {
        let mut layer = SimpleCellLayer::new(
            Area::new(3.0 * PI), Density::new(1.0), Color::Green);
        layer.update_outer_radius(Length::new(1.0));
        assert_eq!(Length::new(2.0), layer.outer_radius());
    }

    #[test]
    fn layer_resize_updates_area_and_mass() {
        let mut layer = SimpleCellLayer::new(
            Area::new(1.0), Density::new(2.0), Color::Green);
        layer.resize(Area::new(2.0));
        assert_eq!(Area::new(2.0), layer.area());
        assert_eq!(Mass::new(4.0), layer.mass());
    }

    #[test]
    fn thruster_layer_adds_force_to_cell() {
        let mut layer = ThrusterLayer::new(Area::new(1.0));
        layer.control_input(0, 1.0);
        layer.control_input(1, -1.0);

        let mut cell = SimpleLayerCellAPI::new();
        layer.after_influences(cell.forces_mut());

        assert_eq!(Force::new(1.0, -1.0), cell.forces().net_force());
    }

    struct SimpleLayerCellAPI {
        forces: Forces
    }

    impl SimpleLayerCellAPI {
        fn new() -> Self {
            SimpleLayerCellAPI {
                forces: Forces::new(0.0, 0.0)
            }
        }
    }

    impl LayerCellAPI for SimpleLayerCellAPI {
        fn forces(&self) -> &Forces {
            &self.forces
        }

        fn forces_mut(&mut self) -> &mut Forces {
            &mut self.forces
        }
    }
}
