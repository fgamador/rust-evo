use biology::control_requests::*;
use environment::environment::LocalEnvironment;
use evo_view_model::Color;
use physics::quantities::*;
use physics::shapes::Circle;
use std::f64;
use std::f64::consts::PI;
use std::fmt::Debug;

pub trait Onion: Circle {
    type Layer: OnionLayer + ?Sized;

    fn layers(&self) -> &[Box<Self::Layer>];
}

pub trait OnionLayer: Debug {
    fn outer_radius(&self) -> Length;

    fn color(&self) -> Color;

    fn health(&self) -> f64;
}

#[derive(Debug)]
pub struct SimpleOnionLayer {
    outer_radius: Length,
    color: Color,
    health: f64,
}

impl SimpleOnionLayer {
    pub fn new(outer_radius: Length, color: Color) -> Self {
        SimpleOnionLayer {
            outer_radius,
            color,
            health: 1.0,
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

    fn health(&self) -> f64 {
        self.health
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LayerHealthParameters {
    pub healing_energy_delta: BioEnergyDelta,
    pub entropic_damage_health_delta: f64,
}

impl LayerHealthParameters {
    pub const DEFAULT: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::ZERO,
        entropic_damage_health_delta: 0.0,
    };
}

#[derive(Debug, Clone, Copy)]
pub struct LayerResizeParameters {
    pub growth_energy_delta: BioEnergyDelta,
    pub max_growth_rate: f64,
    pub shrinkage_energy_delta: BioEnergyDelta,
    pub max_shrinkage_rate: f64,
}

impl LayerResizeParameters {
    pub const DEFAULT: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::ZERO,
        max_growth_rate: f64::INFINITY,
        shrinkage_energy_delta: BioEnergyDelta::ZERO,
        max_shrinkage_rate: f64::INFINITY,
    };
}

#[derive(Debug)]
pub struct CellLayer {
    specialty: Box<CellLayerSpecialty>,
    area: Area,
    density: Density,
    mass: Mass,
    outer_radius: Length,
    color: Color,
    health: f64,
    health_parameters: LayerHealthParameters,
    resize_parameters: LayerResizeParameters,
}

impl CellLayer {
    pub fn new(area: Area, density: Density, color: Color, specialty: Box<CellLayerSpecialty>) -> Self {
        CellLayer {
            specialty,
            area,
            density,
            mass: area * density,
            outer_radius: (area / PI).sqrt(),
            color,
            health: 1.0,
            // TODO pull these out and share them
            health_parameters: LayerHealthParameters::DEFAULT,
            resize_parameters: LayerResizeParameters::DEFAULT,
        }
    }

    pub fn with_health_parameters(mut self, health_parameters: LayerHealthParameters) -> Self {
        self.health_parameters = health_parameters;
        self
    }

    pub fn with_resize_parameters(mut self, resize_parameters: LayerResizeParameters) -> Self {
        self.resize_parameters = resize_parameters;
        self
    }

    pub fn area(&self) -> Area {
        self.area
    }

    pub fn mass(&self) -> Mass {
        self.mass
    }

    pub fn damage(&mut self, health_loss: f64) {
        assert!(health_loss >= 0.0);
        self.health = (self.health - health_loss).max(0.0);
    }

    pub fn update_outer_radius(&mut self, inner_radius: Length) {
        self.outer_radius = (inner_radius.sqr() + self.area / PI).sqrt();
    }

    pub fn after_influences(&mut self, env: &LocalEnvironment, subtick_duration: Duration) -> (BioEnergy, Force) {
        if self.health == 0.0 {
            return (BioEnergy::ZERO, Force::ZERO);
        }

        self.entropic_damage(subtick_duration);
        let health = self.health();
        let area = self.area();
        self.specialty.after_influences(env, subtick_duration, health, area)
    }

    fn entropic_damage(&mut self, subtick_duration: Duration) {
        let subtick_decay = self.health_parameters.entropic_damage_health_delta * subtick_duration.value();
        self.damage(-subtick_decay);
    }

    pub fn cost_control_request(&self, request: ControlRequest) -> CostedControlRequest {
        if self.health == 0.0 {
            return CostedControlRequest::new(request, BioEnergyDelta::new(0.0));
        }

        match request.channel_index {
            0 => self.cost_restore_health(request),
            1 => self.cost_resize(request),
            _ => self.specialty.cost_control_request(request),
        }
    }

    fn cost_restore_health(&self, request: ControlRequest) -> CostedControlRequest {
        CostedControlRequest::new(request,
                                  self.health_parameters.healing_energy_delta * self.area.value() * request.value)
    }

    fn cost_resize(&self, request: ControlRequest) -> CostedControlRequest {
        let delta_area = self.bound_resize_delta_area(request.value);
        let energy_delta = if request.value >= 0.0 {
            self.resize_parameters.growth_energy_delta
        } else {
            -self.resize_parameters.shrinkage_energy_delta
        };
        CostedControlRequest::new(request, delta_area * energy_delta)
    }

    pub fn execute_control_request(&mut self, request: BudgetedControlRequest) {
        if self.health == 0.0 {
            return;
        }

        match request.channel_index {
            0 => self.restore_health(request.value, request.budgeted_fraction),
            1 => self.resize(request.value, request.budgeted_fraction),
            _ => {
                let health = self.health();
                self.specialty.execute_control_request(request, health)
            }
        }
    }

    fn restore_health(&mut self, requested_delta_health: f64, budgeted_fraction: f64) {
        assert!(requested_delta_health >= 0.0);
        self.health = (self.health + budgeted_fraction * requested_delta_health).min(1.0);
    }

    fn resize(&mut self, requested_delta_area: f64, budgeted_fraction: f64) {
        let delta_area = self.health * budgeted_fraction * self.bound_resize_delta_area(requested_delta_area);
        self.area = Area::new((self.area.value() + delta_area).max(0.0));
        self.mass = self.area * self.density;
    }

    fn bound_resize_delta_area(&self, requested_delta_area: f64) -> f64 {
        if requested_delta_area >= 0.0 {
            // TODO a layer that starts with area 0.0 cannot grow
            let max_delta_area = self.resize_parameters.max_growth_rate * self.area.value();
            requested_delta_area.min(max_delta_area)
        } else {
            let min_delta_area = -self.resize_parameters.max_shrinkage_rate * self.area.value();
            requested_delta_area.max(min_delta_area)
        }
    }
}

impl OnionLayer for CellLayer {
    fn outer_radius(&self) -> Length {
        self.outer_radius
    }

    fn color(&self) -> Color {
        self.color
    }

    fn health(&self) -> f64 {
        self.health
    }
}

pub trait CellLayerSpecialty: Debug {
    fn after_influences(&mut self, _env: &LocalEnvironment, _subtick_duration: Duration, _health: f64, _area: Area) -> (BioEnergy, Force) {
        (BioEnergy::ZERO, Force::ZERO)
    }

    fn cost_control_request(&self, request: ControlRequest) -> CostedControlRequest {
        CostedControlRequest::new(request, BioEnergyDelta::ZERO)
    }

    fn execute_control_request(&mut self, _request: BudgetedControlRequest, _health: f64) {}
}

#[derive(Debug)]
pub struct NullCellLayerSpecialty {}

impl NullCellLayerSpecialty {
    pub fn new() -> Self {
        NullCellLayerSpecialty {}
    }
}

impl CellLayerSpecialty for NullCellLayerSpecialty {}

#[derive(Debug)]
pub struct ThrusterCellLayerSpecialty {
    force_x: f64,
    force_y: f64,
}

impl ThrusterCellLayerSpecialty {
    pub fn new() -> Self {
        ThrusterCellLayerSpecialty {
            force_x: 0.0,
            force_y: 0.0,
        }
    }
}

impl CellLayerSpecialty for ThrusterCellLayerSpecialty {
    fn after_influences(&mut self, _env: &LocalEnvironment, _subtick_duration: Duration, _health: f64, _area: Area) -> (BioEnergy, Force) {
        (BioEnergy::ZERO, Force::new(self.force_x, self.force_y))
    }

    fn cost_control_request(&self, request: ControlRequest) -> CostedControlRequest {
        match request.channel_index {
            2 | 3 => CostedControlRequest::new(request, BioEnergyDelta::ZERO), // TODO
            _ => panic!("Invalid control input index: {}", request.channel_index)
        }
    }

    fn execute_control_request(&mut self, request: BudgetedControlRequest, health: f64) {
        match request.channel_index {
            2 => self.force_x = health * request.value,
            3 => self.force_y = health * request.value,
            _ => panic!("Invalid control input index: {}", request.channel_index)
        }
    }
}

#[derive(Debug)]
pub struct PhotoCellLayerSpecialty {
    efficiency: f64,
}

impl PhotoCellLayerSpecialty {
    pub fn new(efficiency: f64) -> Self {
        PhotoCellLayerSpecialty {
            efficiency
        }
    }
}

impl CellLayerSpecialty for PhotoCellLayerSpecialty {
    fn after_influences(&mut self, env: &LocalEnvironment, subtick_duration: Duration, health: f64, area: Area) -> (BioEnergy, Force) {
        (BioEnergy::new(env.light_intensity() * self.efficiency * health * area.value() * subtick_duration.value()),
         Force::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use environment::environment::LocalEnvironment;
    use biology::control_requests::BudgetedControlRequest;

    #[test]
    fn layer_calculates_mass() {
        let layer = simple_cell_layer(Area::new(2.0 * PI), Density::new(3.0));
        assert_eq!(Mass::new(6.0 * PI), layer.mass());
    }

    #[test]
    fn single_layer_calculates_outer_radius() {
        let layer = simple_cell_layer(Area::new(4.0 * PI), Density::new(1.0));
        assert_eq!(Length::new(2.0), layer.outer_radius());
    }

    #[test]
    fn layer_updates_outer_radius_based_on_inner_radius() {
        let mut layer = simple_cell_layer(Area::new(3.0 * PI), Density::new(1.0));
        layer.update_outer_radius(Length::new(1.0));
        assert_eq!(Length::new(2.0), layer.outer_radius());
    }

    #[test]
    fn layer_resize_updates_area_and_mass() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(2.0));
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::for_resize(0, 2.0), BioEnergyDelta::ZERO), 1.0));
        assert_eq!(Area::new(3.0), layer.area());
        assert_eq!(Mass::new(6.0), layer.mass());
    }

    #[test]
    fn layer_damage_reduces_health() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(0.25);
        assert_eq!(0.75, layer.health());
    }

    #[test]
    fn layer_damage_cannot_reduce_health_below_zero() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(2.0);
        assert_eq!(0.0, layer.health());
    }

    #[test]
    fn layer_costs_resize_request() {
        let layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::new(-0.5),
                max_growth_rate: f64::INFINITY,
                shrinkage_energy_delta: BioEnergyDelta::ZERO,
                max_shrinkage_rate: f64::INFINITY,
            });
        let costed_request = layer.cost_control_request(ControlRequest::for_resize(0, 3.0));
        assert_eq!(costed_request, CostedControlRequest::new(
            ControlRequest::for_resize(0, 3.0), BioEnergyDelta::new(-1.5)));
    }

    #[test]
    fn layer_growth_is_limited_by_budgeted_fraction() {
        let mut layer = simple_cell_layer(Area::new(2.0), Density::new(1.0));
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::for_resize(0, 2.0), BioEnergyDelta::ZERO), 0.5));
        assert_eq!(Area::new(3.0), layer.area());
    }

    #[test]
    fn layer_growth_is_limited_by_max_growth_rate() {
        let mut layer = simple_cell_layer(Area::new(2.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::ZERO,
                max_growth_rate: 0.5,
                shrinkage_energy_delta: BioEnergyDelta::ZERO,
                max_shrinkage_rate: f64::INFINITY,
            });
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::for_resize(0, 10.0), BioEnergyDelta::ZERO), 1.0));
        assert_eq!(Area::new(3.0), layer.area());
    }

    #[test]
    fn layer_growth_cost_is_limited_by_max_growth_rate() {
        let layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::new(-3.0),
                max_growth_rate: 0.5,
                shrinkage_energy_delta: BioEnergyDelta::ZERO,
                max_shrinkage_rate: f64::INFINITY,
            });
        let control_request = ControlRequest::for_resize(0, 2.0);
        let costed_request = layer.cost_control_request(control_request);
        assert_eq!(costed_request, CostedControlRequest::new(control_request, BioEnergyDelta::new(-1.5)));
    }

    #[test]
    fn layer_shrinkage_is_limited_by_max_shrinkage_rate() {
        let mut layer = simple_cell_layer(Area::new(2.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::ZERO,
                max_growth_rate: f64::INFINITY,
                shrinkage_energy_delta: BioEnergyDelta::ZERO,
                max_shrinkage_rate: 0.25,
            });
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::for_resize(0, -10.0), BioEnergyDelta::ZERO), 1.0));
        assert_eq!(Area::new(1.5), layer.area());
    }

    #[test]
    fn layer_shrinkage_yield_is_limited_by_max_shrinkage_rate() {
        let layer = simple_cell_layer(Area::new(4.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::ZERO,
                max_growth_rate: f64::INFINITY,
                shrinkage_energy_delta: BioEnergyDelta::new(3.0),
                max_shrinkage_rate: 0.5,
            });
        let control_request = ControlRequest::for_resize(0, -10.0);
        let costed_request = layer.cost_control_request(control_request);
        assert_eq!(costed_request, CostedControlRequest::new(control_request, BioEnergyDelta::new(6.0)));
    }

    #[test]
    fn layer_resize_is_reduced_by_reduced_health() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(0.5);
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::for_resize(0, 10.0), BioEnergyDelta::ZERO), 1.0));
        assert_eq!(Area::new(6.0), layer.area());
    }

    #[test]
    fn layer_resize_cost_is_not_reduced_by_reduced_health() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::new(-1.0),
                max_growth_rate: f64::INFINITY,
                shrinkage_energy_delta: BioEnergyDelta::ZERO,
                max_shrinkage_rate: f64::INFINITY,
            });
        layer.damage(0.5);
        let control_request = ControlRequest::for_resize(0, 1.0);
        let costed_request = layer.cost_control_request(control_request);
        assert_eq!(costed_request, CostedControlRequest::new(control_request, BioEnergyDelta::new(-1.0)));
    }

    #[test]
    fn layer_health_can_be_restored() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(0.5);
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::for_healing(0, 0.25), BioEnergyDelta::ZERO), 1.0));
        assert_eq!(0.75, layer.health());
    }

    #[test]
    fn layer_health_cannot_be_restored_above_one() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(0.5);
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::for_healing(0, 1.0), BioEnergyDelta::ZERO), 1.0));
        assert_eq!(1.0, layer.health());
    }

    #[test]
    fn layer_health_restoration_is_limited_by_budgeted_fraction() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(0.5);
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::for_healing(0, 0.5), BioEnergyDelta::ZERO), 0.5));
        assert_eq!(0.75, layer.health());
    }

    #[test]
    fn layer_costs_health_restoration() {
        let mut layer = simple_cell_layer(Area::new(2.0), Density::new(1.0))
            .with_health_parameters(LayerHealthParameters {
                healing_energy_delta: BioEnergyDelta::new(-3.0),
                entropic_damage_health_delta: 0.0,
            });
        layer.damage(0.5);
        let control_request = ControlRequest::for_healing(0, 0.25);
        let costed_request = layer.cost_control_request(control_request);
        assert_eq!(costed_request, CostedControlRequest::new(control_request, BioEnergyDelta::new(-1.5)));
    }

    #[test]
    fn layer_undergoes_entropic_damage() {
        let mut layer = simple_cell_layer(Area::new(2.0), Density::new(1.0))
            .with_health_parameters(LayerHealthParameters {
                healing_energy_delta: BioEnergyDelta::ZERO,
                entropic_damage_health_delta: -0.1,
            });

        let env = LocalEnvironment::new();
        let (_, _) = layer.after_influences(&env, Duration::new(0.5));

        assert_eq!(0.95, layer.health());
    }

    #[test]
    fn dead_layer_costs_control_requests_at_zero() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_health_parameters(LayerHealthParameters {
                healing_energy_delta: BioEnergyDelta::new(-1.0),
                entropic_damage_health_delta: 0.0,
            });
        layer.damage(1.0);
        let control_request = ControlRequest::for_healing(0, 1.0);
        let costed_request = layer.cost_control_request(control_request);
        assert_eq!(costed_request, CostedControlRequest::new(control_request, BioEnergyDelta::new(0.0)));
    }

    #[test]
    fn dead_layer_ignores_control_requests() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(1.0);
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::for_healing(0, 1.0), BioEnergyDelta::ZERO), 1.0));
        assert_eq!(0.0, layer.health());
    }

    #[test]
    fn thruster_layer_adds_force() {
        let mut layer = CellLayer::new(Area::new(1.0), Density::new(1.0), Color::Green,
                                       Box::new(ThrusterCellLayerSpecialty::new()));
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::new(0, 2, 1.0), BioEnergyDelta::ZERO), 1.0));
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::new(0, 3, -1.0), BioEnergyDelta::ZERO), 1.0));

        let env = LocalEnvironment::new();
        let (_, force) = layer.after_influences(&env, Duration::new(0.5));

        assert_eq!(Force::new(1.0, -1.0), force);
    }

    #[test]
    fn thruster_layer_force_is_reduced_by_reduced_health() {
        let mut layer = CellLayer::new(Area::new(1.0), Density::new(1.0), Color::Green,
                                       Box::new(ThrusterCellLayerSpecialty::new()));
        layer.damage(0.5);
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::new(0, 2, 1.0), BioEnergyDelta::ZERO), 1.0));
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::new(0, 3, -1.0), BioEnergyDelta::ZERO), 1.0));

        let env = LocalEnvironment::new();
        let (_, force) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(Force::new(0.5, -0.5), force);
    }

    #[test]
    fn dead_thruster_layer_adds_no_force() {
        let mut layer = CellLayer::new(Area::new(1.0), Density::new(1.0), Color::Green,
                                       Box::new(ThrusterCellLayerSpecialty::new()));
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::new(0, 2, 1.0), BioEnergyDelta::ZERO), 1.0));
        layer.execute_control_request(
            BudgetedControlRequest::new(
                CostedControlRequest::new(
                    ControlRequest::new(0, 3, -1.0), BioEnergyDelta::ZERO), 1.0));
        layer.damage(1.0);

        let env = LocalEnvironment::new();
        let (_, force) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(Force::new(0.0, 0.0), force);
    }

    #[test]
    fn photo_layer_adds_energy_based_on_area_and_efficiency_and_duration() {
        let mut layer = CellLayer::new(Area::new(4.0), Density::new(1.0), Color::Green,
                                       Box::new(PhotoCellLayerSpecialty::new(0.5)));

        let mut env = LocalEnvironment::new();
        env.add_light_intensity(10.0);

        let (energy, _) = layer.after_influences(&env, Duration::new(0.5));

        assert_eq!(BioEnergy::new(10.0), energy);
    }

    #[test]
    fn photo_layer_energy_is_reduced_by_reduced_health() {
        let mut layer = CellLayer::new(Area::new(1.0), Density::new(1.0), Color::Green,
                                       Box::new(PhotoCellLayerSpecialty::new(1.0)));
        layer.damage(0.25);

        let mut env = LocalEnvironment::new();
        env.add_light_intensity(1.0);

        let (energy, _) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(BioEnergy::new(0.75), energy);
    }

    #[test]
    fn dead_photo_layer_adds_no_energy() {
        let mut layer = CellLayer::new(Area::new(1.0), Density::new(1.0), Color::Green,
                                       Box::new(PhotoCellLayerSpecialty::new(1.0)));
        layer.damage(1.0);

        let mut env = LocalEnvironment::new();
        env.add_light_intensity(1.0);

        let (energy, _) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(BioEnergy::new(0.0), energy);
    }

    fn simple_cell_layer(area: Area, density: Density) -> CellLayer {
        CellLayer::new(area, density, Color::Green, Box::new(NullCellLayerSpecialty::new()))
    }
}
