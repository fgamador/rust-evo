use biology::control::*;
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

    fn execute_control_request(&mut self, request: ControlRequest);

    fn control_input(&mut self, _index: usize, _value: f64) {}

    fn after_influences(&mut self, _forces: &mut Forces) {}

    fn resize(&mut self, new_area: Area) {
        self.execute_control_request(ControlRequest::new(0, 0, new_area.value()));
    }
}

#[derive(Debug)]
pub struct Annulus {
    area: Area,
    density: Density,
    mass: Mass,
    outer_radius: Length,
    color: Color,
}

impl Annulus {
    pub fn new(area: Area, density: Density, color: Color) -> Self {
        Annulus {
            area,
            density,
            mass: area * density,
            outer_radius: (area / PI).sqrt(),
            color,
        }
    }

    pub fn update_outer_radius(&mut self, inner_radius: Length) {
        self.outer_radius = (inner_radius.sqr() + self.area / PI).sqrt();
    }

    pub fn resize(&mut self, new_area: Area) {
        self.area = new_area;
        self.mass = self.area * self.density;
    }
}

#[derive(Debug)]
pub struct SimpleCellLayer {
    annulus: Annulus
}

impl SimpleCellLayer {
    pub fn new(area: Area, density: Density, color: Color) -> Self {
        SimpleCellLayer {
            annulus: Annulus::new(area, density, color)
        }
    }
}

impl OnionLayer for SimpleCellLayer {
    fn outer_radius(&self) -> Length {
        self.annulus.outer_radius
    }

    fn color(&self) -> Color {
        self.annulus.color
    }
}

impl CellLayer for SimpleCellLayer {
    fn area(&self) -> Area {
        self.annulus.area
    }

    fn mass(&self) -> Mass {
        self.annulus.mass
    }

    fn update_outer_radius(&mut self, inner_radius: Length) {
        self.annulus.update_outer_radius(inner_radius);
    }

    fn execute_control_request(&mut self, request: ControlRequest) {
        match request.control_index {
            0 => self.annulus.resize(Area::new(request.control_value)),
            1 => (), // TODO maintenance/repair
            _ => panic!("Invalid control input index: {}", request.control_index)
        }
    }
}

#[derive(Debug)]
pub struct ThrusterLayer {
    annulus: Annulus,
    force_x: f64,
    force_y: f64,
}

impl ThrusterLayer {
    pub fn new(area: Area) -> Self {
        let density = Density::new(1.0); // TODO
        ThrusterLayer {
            annulus: Annulus::new(area, density, Color::Green), // TODO color
            force_x: 0.0,
            force_y: 0.0,
        }
    }
}

impl OnionLayer for ThrusterLayer {
    fn outer_radius(&self) -> Length {
        self.annulus.outer_radius
    }

    fn color(&self) -> Color {
        self.annulus.color
    }
}

impl CellLayer for ThrusterLayer {
    fn area(&self) -> Area {
        self.annulus.area
    }

    fn mass(&self) -> Mass {
        self.annulus.mass
    }

    fn update_outer_radius(&mut self, inner_radius: Length) {
        self.annulus.update_outer_radius(inner_radius);
    }

    fn execute_control_request(&mut self, request: ControlRequest) {
        match request.control_index {
            0 => self.annulus.resize(Area::new(request.control_value)),
            1 => (), // TODO maintenance/repair
            2 => self.force_x = request.control_value,
            3 => self.force_y = request.control_value,
            _ => panic!("Invalid control input index: {}", request.control_index)
        }
    }

    fn control_input(&mut self, index: usize, value: f64) {
        self.execute_control_request(ControlRequest::new(0, index + 2, value));
    }

    fn after_influences(&mut self, forces: &mut Forces) {
        forces.add_force(Force::new(self.force_x, self.force_y));
    }
}

#[derive(Debug)]
pub struct PhotoLayer {
    annulus: Annulus,
    efficiency: f64,
}

impl PhotoLayer {
    pub fn new(area: Area, efficiency: f64) -> Self {
        let density = Density::new(1.0); // TODO
        PhotoLayer {
            annulus: Annulus::new(area, density, Color::Green),
            efficiency,
        }
    }
}

impl OnionLayer for PhotoLayer {
    fn outer_radius(&self) -> Length {
        self.annulus.outer_radius
    }

    fn color(&self) -> Color {
        self.annulus.color
    }
}

impl CellLayer for PhotoLayer {
    fn area(&self) -> Area {
        self.annulus.area
    }

    fn mass(&self) -> Mass {
        self.annulus.mass
    }

    fn update_outer_radius(&mut self, inner_radius: Length) {
        self.annulus.update_outer_radius(inner_radius);
    }

    fn execute_control_request(&mut self, request: ControlRequest) {
        match request.control_index {
            0 => self.annulus.resize(Area::new(request.control_value)),
            1 => (), // TODO maintenance/repair
            _ => panic!("Invalid control input index: {}", request.control_index)
        }
    }

    fn after_influences(&mut self, _forces: &mut Forces) {
        // TODO convert light into energy
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
        layer.execute_control_request(ControlRequest::new(0, 2, 1.0));
        layer.execute_control_request(ControlRequest::new(0, 3, -1.0));

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
