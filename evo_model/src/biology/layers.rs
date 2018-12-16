use biology::control::*;
use environment::environment::LocalEnvironment;
use evo_view_model::Color;
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

    fn after_influences(&mut self, _env: &LocalEnvironment) -> (f64, Force) {
        (0.0, Force::new(0.0, 0.0))
    }

    fn cost_control_request(&mut self, request: ControlRequest) -> CostedControlRequest {
        CostedControlRequest::new(request, 0.0)
    }

    fn execute_control_request(&mut self, request: ControlRequest);
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
    annulus: Annulus,
    growth_cost: f64,
}

impl SimpleCellLayer {
    pub fn new(area: Area, density: Density, color: Color) -> Self {
        SimpleCellLayer {
            annulus: Annulus::new(area, density, color),
            growth_cost: 0.0,
        }
    }

    pub fn with_growth_cost(mut self, cost: f64) -> Self {
        self.growth_cost = cost;
        self
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

    fn cost_control_request(&mut self, request: ControlRequest) -> CostedControlRequest {
        CostedControlRequest::new(request, request.control_value * self.growth_cost)
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

    fn after_influences(&mut self, _env: &LocalEnvironment) -> (f64, Force) {
        (0.0, Force::new(self.force_x, self.force_y))
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

    fn after_influences(&mut self, env: &LocalEnvironment) -> (f64, Force) {
        (env.light_intensity() * self.efficiency * self.area().value(), Force::new(0.0, 0.0))
    }

    fn execute_control_request(&mut self, request: ControlRequest) {
        match request.control_index {
            0 => self.annulus.resize(Area::new(request.control_value)),
            1 => (), // TODO maintenance/repair
            _ => panic!("Invalid control input index: {}", request.control_index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use environment::environment::LocalEnvironment;

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
        layer.execute_control_request(ControlRequest::new(0, 0, 2.0));
        assert_eq!(Area::new(2.0), layer.area());
        assert_eq!(Mass::new(4.0), layer.mass());
    }

    #[test]
    fn layer_costs_resize_request() {
        let mut layer = SimpleCellLayer::new(Area::new(1.0), Density::new(1.0), Color::Green)
            .with_growth_cost(0.5);
        let costed_request = layer.cost_control_request(ControlRequest::new(0, 0, 3.0));
        assert_eq!(costed_request, CostedControlRequest::new(ControlRequest::new(0, 0, 3.0), 1.5));
    }

    #[test]
    fn thruster_layer_adds_force() {
        let mut layer = ThrusterLayer::new(Area::new(1.0));
        layer.execute_control_request(ControlRequest::new(0, 2, 1.0));
        layer.execute_control_request(ControlRequest::new(0, 3, -1.0));

        let env = LocalEnvironment::new();
        let (_, force) = layer.after_influences(&env);

        assert_eq!(Force::new(1.0, -1.0), force);
    }

    #[test]
    fn photo_layer_adds_energy_based_on_area_and_efficiency() {
        let mut layer = PhotoLayer::new(Area::new(4.0), 0.5);

        let mut env = LocalEnvironment::new();
        env.add_light_intensity(10.0);

        let (energy, _) = layer.after_influences(&env);

        assert_eq!(20.0, energy);
    }
}
