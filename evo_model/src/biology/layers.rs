use crate::biology::cell::Cell;
use crate::biology::control::CellStateSnapshot;
use crate::biology::control_requests::*;
use crate::environment::local_environment::LocalEnvironment;
use crate::physics::overlap::Overlap;
use crate::physics::quantities::*;
use crate::physics::shapes::Circle;
use std::f64;
use std::f64::consts::PI;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    Green,
    White,
    Yellow,
}

// TODO Do we still need Onion and OnionLayer? Can we merge them into Cell and Layer?
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
    pub overlap_damage_health_delta: f64,
}

impl LayerHealthParameters {
    pub const DEFAULT: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::ZERO,
        entropic_damage_health_delta: 0.0,
        overlap_damage_health_delta: 0.0,
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
    pub const UNLIMITED: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::ZERO,
        max_growth_rate: f64::INFINITY,
        shrinkage_energy_delta: BioEnergyDelta::ZERO,
        max_shrinkage_rate: 1.0,
    };
}

#[derive(Debug)]
pub struct CellLayer {
    body: CellLayerBody,
    specialty: Box<dyn CellLayerSpecialty>,
}

impl CellLayer {
    const LIVING_BRAIN: LivingCellLayerBrain = LivingCellLayerBrain {};
    const DEAD_BRAIN: DeadCellLayerBrain = DeadCellLayerBrain {};

    pub fn new(
        area: Area,
        density: Density,
        color: Color,
        specialty: Box<dyn CellLayerSpecialty>,
    ) -> Self {
        CellLayer {
            body: CellLayerBody::new(area, density, color),
            specialty,
        }
    }

    pub fn with_health_parameters(mut self, health_parameters: LayerHealthParameters) -> Self {
        self.body.health_parameters = health_parameters;
        self
    }

    pub fn with_resize_parameters(mut self, resize_parameters: LayerResizeParameters) -> Self {
        self.body.resize_parameters = resize_parameters;
        self
    }

    pub fn with_health(mut self, health: f64) -> Self {
        self.body.health = health;
        self
    }

    pub fn dead(mut self) -> Self {
        self.damage(1.0);
        self
    }

    pub fn mark_as_surface(&mut self) {
        self.body.is_surface = true;
    }

    pub fn is_alive(&self) -> bool {
        self.health() > 0.0
    }

    pub fn area(&self) -> Area {
        self.body.area
    }

    pub fn mass(&self) -> Mass {
        self.body.mass
    }

    pub fn damage(&mut self, health_loss: f64) {
        self.body.brain.damage(&mut self.body, health_loss);
    }

    pub fn update_outer_radius(&mut self, inner_radius: Length) {
        self.body.update_outer_radius(inner_radius);
    }

    pub fn after_influences(
        &mut self,
        env: &LocalEnvironment,
        subtick_duration: Duration,
    ) -> (BioEnergy, Force) {
        self.body.brain.after_influences(
            &mut *self.specialty,
            &mut self.body,
            env,
            subtick_duration,
        )
    }

    pub fn cost_control_request(&mut self, request: ControlRequest) -> CostedControlRequest {
        self.body
            .brain
            .cost_control_request(&mut *self.specialty, &self.body, request)
    }

    pub fn execute_control_request(&mut self, request: BudgetedControlRequest) {
        self.body
            .brain
            .execute_control_request(&mut *self.specialty, &mut self.body, request);
    }

    pub fn after_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Option<Cell> {
        let child = self
            .body
            .brain
            .after_control_requests(&mut *self.specialty, cell_state);
        self.specialty.reset();
        child
    }

    pub fn healing_request(layer_index: usize, delta_health: f64) -> ControlRequest {
        ControlRequest::new(layer_index, 0, delta_health)
    }

    pub fn resize_request(layer_index: usize, delta_area: AreaDelta) -> ControlRequest {
        ControlRequest::new(layer_index, 1, delta_area.value())
    }
}

impl OnionLayer for CellLayer {
    fn outer_radius(&self) -> Length {
        self.body.outer_radius
    }

    fn color(&self) -> Color {
        self.body.color
    }

    fn health(&self) -> f64 {
        self.body.health
    }
}

// CellLayerBody is separate from CellLayer so it can be mutably passed to CellLayerSpecialty.
// CellLayerBrain is in CellLayerBody so the brain can change its body to use a new brain.
#[derive(Debug)]
pub struct CellLayerBody {
    area: Area,
    density: Density,
    mass: Mass,
    outer_radius: Length,
    is_surface: bool,
    health: f64,
    color: Color,
    brain: &'static dyn CellLayerBrain,
    // TODO move to CellLayerParameters struct?
    health_parameters: LayerHealthParameters,
    resize_parameters: LayerResizeParameters,
}

impl CellLayerBody {
    fn new(area: Area, density: Density, color: Color) -> Self {
        CellLayerBody {
            area,
            density,
            mass: area * density,
            outer_radius: (area / PI).sqrt(),
            is_surface: false,
            health: 1.0,
            color,
            brain: &CellLayer::LIVING_BRAIN,
            // TODO pull these out and share them
            health_parameters: LayerHealthParameters::DEFAULT,
            resize_parameters: LayerResizeParameters::UNLIMITED,
        }
    }

    fn damage(&mut self, health_loss: f64) {
        assert!(health_loss >= 0.0);
        self.health = (self.health - health_loss).max(0.0);
    }

    fn update_outer_radius(&mut self, inner_radius: Length) {
        self.outer_radius = (inner_radius.sqr() + self.area / PI).sqrt();
    }

    fn cost_restore_health(&self, request: ControlRequest) -> CostedControlRequest {
        CostedControlRequest::new(
            request,
            self.health_parameters.healing_energy_delta * self.area.value() * request.value,
        )
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

    fn restore_health(&mut self, requested_delta_health: f64, budgeted_fraction: f64) {
        assert!(requested_delta_health >= 0.0);
        self.health = (self.health + budgeted_fraction * requested_delta_health).min(1.0);
    }

    fn resize(&mut self, requested_delta_area: f64, budgeted_fraction: f64) {
        let delta_area =
            self.health * budgeted_fraction * self.bound_resize_delta_area(requested_delta_area);
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

trait CellLayerBrain: Debug {
    fn damage(&self, body: &mut CellLayerBody, health_loss: f64);

    fn after_influences(
        &self,
        specialty: &mut dyn CellLayerSpecialty,
        body: &mut CellLayerBody,
        env: &LocalEnvironment,
        subtick_duration: Duration,
    ) -> (BioEnergy, Force);

    fn cost_control_request(
        &self,
        specialty: &mut dyn CellLayerSpecialty,
        body: &CellLayerBody,
        request: ControlRequest,
    ) -> CostedControlRequest;

    fn execute_control_request(
        &self,
        specialty: &mut dyn CellLayerSpecialty,
        body: &mut CellLayerBody,
        request: BudgetedControlRequest,
    );

    fn after_control_requests(
        &self,
        specialty: &mut dyn CellLayerSpecialty,
        cell_state: &CellStateSnapshot,
    ) -> Option<Cell>;
}

#[derive(Debug)]
struct LivingCellLayerBrain {}

impl LivingCellLayerBrain {
    fn entropic_damage(&self, body: &mut CellLayerBody, subtick_duration: Duration) {
        let subtick_damage =
            body.health_parameters.entropic_damage_health_delta * subtick_duration.value();
        self.damage(body, -subtick_damage);
    }

    fn overlap_damage(&self, body: &mut CellLayerBody, overlaps: &[Overlap]) {
        let overlap_damage = overlaps.iter().fold(0.0, |total_damage, overlap| {
            total_damage + body.health_parameters.overlap_damage_health_delta * overlap.magnitude()
        });
        self.damage(body, -overlap_damage);
    }
}

impl CellLayerBrain for LivingCellLayerBrain {
    fn damage(&self, body: &mut CellLayerBody, health_loss: f64) {
        body.damage(health_loss);
        if body.health == 0.0 {
            body.brain = &CellLayer::DEAD_BRAIN;
        }
    }

    fn after_influences(
        &self,
        specialty: &mut dyn CellLayerSpecialty,
        body: &mut CellLayerBody,
        env: &LocalEnvironment,
        subtick_duration: Duration,
    ) -> (BioEnergy, Force) {
        self.entropic_damage(body, subtick_duration);
        if body.is_surface {
            self.overlap_damage(body, env.overlaps());
        }
        specialty.after_influences(body, env, subtick_duration)
    }

    fn cost_control_request(
        &self,
        specialty: &mut dyn CellLayerSpecialty,
        body: &CellLayerBody,
        request: ControlRequest,
    ) -> CostedControlRequest {
        match request.channel_index {
            0 => body.cost_restore_health(request),
            1 => body.cost_resize(request),
            _ => specialty.cost_control_request(request),
        }
    }

    fn execute_control_request(
        &self,
        specialty: &mut dyn CellLayerSpecialty,
        body: &mut CellLayerBody,
        request: BudgetedControlRequest,
    ) {
        match request.channel_index {
            0 => body.restore_health(request.value, request.budgeted_fraction),
            1 => body.resize(request.value, request.budgeted_fraction),
            _ => specialty.execute_control_request(body, request),
        }
    }

    fn after_control_requests(
        &self,
        specialty: &mut dyn CellLayerSpecialty,
        cell_state: &CellStateSnapshot,
    ) -> Option<Cell> {
        specialty.after_control_requests(cell_state)
    }
}

#[derive(Debug)]
struct DeadCellLayerBrain {}

impl DeadCellLayerBrain {}

impl CellLayerBrain for DeadCellLayerBrain {
    fn damage(&self, _body: &mut CellLayerBody, _health_loss: f64) {}

    fn after_influences(
        &self,
        _specialty: &mut dyn CellLayerSpecialty,
        _body: &mut CellLayerBody,
        _env: &LocalEnvironment,
        _subtick_duration: Duration,
    ) -> (BioEnergy, Force) {
        (BioEnergy::ZERO, Force::ZERO)
    }

    fn cost_control_request(
        &self,
        _specialty: &mut dyn CellLayerSpecialty,
        _body: &CellLayerBody,
        request: ControlRequest,
    ) -> CostedControlRequest {
        CostedControlRequest::new(request, BioEnergyDelta::ZERO)
    }

    fn execute_control_request(
        &self,
        _specialty: &mut dyn CellLayerSpecialty,
        _body: &mut CellLayerBody,
        _request: BudgetedControlRequest,
    ) {
    }

    fn after_control_requests(
        &self,
        _specialty: &mut dyn CellLayerSpecialty,
        _cell_state: &CellStateSnapshot,
    ) -> Option<Cell> {
        None
    }
}

pub trait CellLayerSpecialty: Debug {
    fn after_influences(
        &mut self,
        _body: &CellLayerBody,
        _env: &LocalEnvironment,
        _subtick_duration: Duration,
    ) -> (BioEnergy, Force) {
        (BioEnergy::ZERO, Force::ZERO)
    }

    // TODO spread and use this, e.g. for the invalid-index panic
    fn max_control_channel_index(&self) -> usize {
        1
    }

    fn cost_control_request(&self, request: ControlRequest) -> CostedControlRequest {
        panic!("Invalid control channel index: {}", request.channel_index);
    }

    fn execute_control_request(&mut self, _body: &CellLayerBody, request: BudgetedControlRequest) {
        panic!("Invalid control channel index: {}", request.channel_index);
    }

    fn after_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Option<Cell> {
        None
    }

    fn reset(&mut self) {}
}

#[derive(Debug)]
pub struct NullCellLayerSpecialty {}

impl NullCellLayerSpecialty {
    #[allow(clippy::new_without_default)]
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ThrusterCellLayerSpecialty {
            force_x: 0.0,
            force_y: 0.0,
        }
    }

    pub fn force_x_request(layer_index: usize, value: f64) -> ControlRequest {
        ControlRequest::new(layer_index, 2, value)
    }

    pub fn force_y_request(layer_index: usize, value: f64) -> ControlRequest {
        ControlRequest::new(layer_index, 3, value)
    }
}

impl CellLayerSpecialty for ThrusterCellLayerSpecialty {
    fn after_influences(
        &mut self,
        _body: &CellLayerBody,
        _env: &LocalEnvironment,
        _subtick_duration: Duration,
    ) -> (BioEnergy, Force) {
        (BioEnergy::ZERO, Force::new(self.force_x, self.force_y))
    }

    fn cost_control_request(&self, request: ControlRequest) -> CostedControlRequest {
        match request.channel_index {
            // TODO cost forces based on a parameter struct(?)
            2 | 3 => CostedControlRequest::new(request, BioEnergyDelta::ZERO),
            _ => panic!("Invalid control channel index: {}", request.channel_index),
        }
    }

    fn execute_control_request(&mut self, body: &CellLayerBody, request: BudgetedControlRequest) {
        match request.channel_index {
            2 => self.force_x = body.health * request.budgeted_fraction * request.value,
            3 => self.force_y = body.health * request.budgeted_fraction * request.value,
            _ => panic!("Invalid control channel index: {}", request.channel_index),
        }
    }
}

#[derive(Debug)]
pub struct PhotoCellLayerSpecialty {
    efficiency: f64,
}

impl PhotoCellLayerSpecialty {
    pub fn new(efficiency: f64) -> Self {
        PhotoCellLayerSpecialty { efficiency }
    }
}

impl CellLayerSpecialty for PhotoCellLayerSpecialty {
    fn after_influences(
        &mut self,
        body: &CellLayerBody,
        env: &LocalEnvironment,
        subtick_duration: Duration,
    ) -> (BioEnergy, Force) {
        (
            BioEnergy::new(
                env.light_intensity()
                    * self.efficiency
                    * body.health
                    * body.area.value()
                    * subtick_duration.value(),
            ),
            Force::ZERO,
        )
    }
}

#[derive(Debug)]
pub struct EnergyGeneratingCellLayerSpecialty {
    energy: BioEnergy,
}

impl EnergyGeneratingCellLayerSpecialty {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EnergyGeneratingCellLayerSpecialty {
            energy: BioEnergy::ZERO,
        }
    }

    pub fn energy_request(layer_index: usize, energy: BioEnergy) -> ControlRequest {
        ControlRequest::new(layer_index, 2, energy.value())
    }
}

impl CellLayerSpecialty for EnergyGeneratingCellLayerSpecialty {
    fn after_influences(
        &mut self,
        body: &CellLayerBody,
        _env: &LocalEnvironment,
        subtick_duration: Duration,
    ) -> (BioEnergy, Force) {
        (
            self.energy * body.health * subtick_duration.value(),
            Force::ZERO,
        )
    }

    fn cost_control_request(&self, request: ControlRequest) -> CostedControlRequest {
        match request.channel_index {
            2 => CostedControlRequest::new(request, BioEnergyDelta::ZERO),
            _ => panic!("Invalid control channel index: {}", request.channel_index),
        }
    }

    fn execute_control_request(&mut self, _body: &CellLayerBody, request: BudgetedControlRequest) {
        match request.channel_index {
            2 => self.energy = request.budgeted_fraction * BioEnergy::new(request.value.max(0.0)),
            _ => panic!("Invalid control channel index: {}", request.channel_index),
        }
    }

    fn reset(&mut self) {
        self.energy = BioEnergy::ZERO;
    }
}

#[derive(Debug)]
pub struct BuddingCellLayerSpecialty {
    create_child: fn() -> Cell,
    budding_angle: Angle,
    donation_energy: BioEnergy,
}

impl BuddingCellLayerSpecialty {
    pub fn new(create_child: fn() -> Cell) -> Self {
        BuddingCellLayerSpecialty {
            create_child,
            budding_angle: Angle::ZERO,
            donation_energy: BioEnergy::ZERO,
        }
    }

    fn create_and_init_child(&mut self, cell_state: &CellStateSnapshot) -> Cell {
        let mut child = (self.create_child)();
        let offset =
            Displacement::from_polar(cell_state.radius + child.radius(), self.budding_angle);
        child.set_initial_position(cell_state.center + offset);
        child.set_initial_velocity(cell_state.velocity);
        child.set_initial_energy(self.donation_energy);
        child
    }

    pub fn budding_angle_request(layer_index: usize, angle: Angle) -> ControlRequest {
        ControlRequest::new(layer_index, 2, angle.radians())
    }

    pub fn donation_energy_request(layer_index: usize, energy: BioEnergy) -> ControlRequest {
        ControlRequest::new(layer_index, 3, energy.value())
    }
}

impl CellLayerSpecialty for BuddingCellLayerSpecialty {
    fn cost_control_request(&self, request: ControlRequest) -> CostedControlRequest {
        match request.channel_index {
            2 => CostedControlRequest::new(request, BioEnergyDelta::ZERO),
            3 => CostedControlRequest::new(request, BioEnergyDelta::new(request.value)),
            _ => panic!("Invalid control channel index: {}", request.channel_index),
        }
    }

    fn execute_control_request(&mut self, body: &CellLayerBody, request: BudgetedControlRequest) {
        match request.channel_index {
            2 => self.budding_angle = Angle::from_radians(request.value),
            3 => {
                self.donation_energy =
                    body.health * request.budgeted_fraction * BioEnergy::new(request.value)
            }
            _ => panic!("Invalid control channel index: {}", request.channel_index),
        }
    }

    fn after_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Option<Cell> {
        if self.donation_energy.value() == 0.0 {
            return None;
        }

        Some(self.create_and_init_child(cell_state))
    }

    fn reset(&mut self) {
        self.donation_energy = BioEnergy::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::control_requests::BudgetedControlRequest;
    use crate::environment::local_environment::LocalEnvironment;
    use crate::physics::newtonian::NewtonianBody;
    use crate::physics::overlap::Overlap;

    #[test]
    fn layer_calculates_mass() {
        let layer = simple_cell_layer(Area::new(2.0 * PI), Density::new(3.0));
        assert_eq!(layer.mass(), Mass::new(6.0 * PI));
    }

    #[test]
    fn single_layer_calculates_outer_radius() {
        let layer = simple_cell_layer(Area::new(4.0 * PI), Density::new(1.0));
        assert_eq!(layer.outer_radius(), Length::new(2.0));
    }

    #[test]
    fn layer_updates_outer_radius_based_on_inner_radius() {
        let mut layer = simple_cell_layer(Area::new(3.0 * PI), Density::new(1.0));
        layer.update_outer_radius(Length::new(1.0));
        assert_eq!(layer.outer_radius(), Length::new(2.0));
    }

    #[test]
    fn layer_resize_updates_area_and_mass() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(2.0));
        layer.execute_control_request(fully_budgeted_resize_request(0, 2.0));
        assert_eq!(layer.area(), Area::new(3.0));
        assert_eq!(layer.mass(), Mass::new(6.0));
    }

    #[test]
    fn layer_damage_reduces_health() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(0.25);
        assert_eq!(layer.health(), 0.75);
    }

    #[test]
    fn layer_damage_cannot_reduce_health_below_zero() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(2.0);
        assert_eq!(layer.health(), 0.0);
    }

    #[test]
    fn layer_with_some_health_is_not_dead() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(0.99);
        assert!(layer.is_alive());
    }

    #[test]
    fn layer_with_zero_health_is_dead() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        layer.damage(1.0);
        assert!(!layer.is_alive());
    }

    #[test]
    fn layer_costs_resize_request() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::new(-0.5),
                ..LayerResizeParameters::UNLIMITED
            });
        let costed_request =
            layer.cost_control_request(CellLayer::resize_request(0, AreaDelta::new(3.0)));
        assert_eq!(
            costed_request,
            CostedControlRequest::new(
                CellLayer::resize_request(0, AreaDelta::new(3.0)),
                BioEnergyDelta::new(-1.5),
            )
        );
    }

    #[test]
    fn layer_growth_is_limited_by_budgeted_fraction() {
        let mut layer = simple_cell_layer(Area::new(2.0), Density::new(1.0));
        layer.execute_control_request(budgeted(
            CellLayer::resize_request(0, AreaDelta::new(2.0)),
            BioEnergyDelta::ZERO,
            0.5,
        ));
        assert_eq!(layer.area(), Area::new(3.0));
    }

    #[test]
    fn layer_growth_is_limited_by_max_growth_rate() {
        let mut layer = simple_cell_layer(Area::new(2.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                max_growth_rate: 0.5,
                ..LayerResizeParameters::UNLIMITED
            });
        layer.execute_control_request(fully_budgeted_resize_request(0, 10.0));
        assert_eq!(layer.area(), Area::new(3.0));
    }

    #[test]
    fn layer_growth_cost_is_limited_by_max_growth_rate() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::new(-3.0),
                max_growth_rate: 0.5,
                ..LayerResizeParameters::UNLIMITED
            });
        let control_request = CellLayer::resize_request(0, AreaDelta::new(2.0));
        let costed_request = layer.cost_control_request(control_request);
        assert_eq!(
            costed_request,
            CostedControlRequest::new(control_request, BioEnergyDelta::new(-1.5))
        );
    }

    #[test]
    fn layer_shrinkage_is_limited_by_max_shrinkage_rate() {
        let mut layer = simple_cell_layer(Area::new(2.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                max_shrinkage_rate: 0.25,
                ..LayerResizeParameters::UNLIMITED
            });
        layer.execute_control_request(fully_budgeted_resize_request(0, -10.0));
        assert_eq!(layer.area(), Area::new(1.5));
    }

    #[test]
    fn layer_shrinkage_yield_is_limited_by_max_shrinkage_rate() {
        let mut layer = simple_cell_layer(Area::new(4.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                shrinkage_energy_delta: BioEnergyDelta::new(3.0),
                max_shrinkage_rate: 0.5,
                ..LayerResizeParameters::UNLIMITED
            });
        let control_request = CellLayer::resize_request(0, AreaDelta::new(-10.0));
        let costed_request = layer.cost_control_request(control_request);
        assert_eq!(
            costed_request,
            CostedControlRequest::new(control_request, BioEnergyDelta::new(6.0))
        );
    }

    #[test]
    fn layer_resize_is_reduced_by_reduced_health() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0)).with_health(0.5);
        layer.execute_control_request(fully_budgeted_resize_request(0, 10.0));
        assert_eq!(layer.area(), Area::new(6.0));
    }

    #[test]
    fn layer_resize_cost_is_not_reduced_by_reduced_health() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_resize_parameters(LayerResizeParameters {
                growth_energy_delta: BioEnergyDelta::new(-1.0),
                ..LayerResizeParameters::UNLIMITED
            })
            .with_health(0.5);
        let control_request = CellLayer::resize_request(0, AreaDelta::new(1.0));
        let costed_request = layer.cost_control_request(control_request);
        assert_eq!(
            costed_request,
            CostedControlRequest::new(control_request, BioEnergyDelta::new(-1.0))
        );
    }

    #[test]
    fn layer_health_can_be_restored() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0)).with_health(0.5);
        layer.execute_control_request(fully_budgeted_healing_request(0, 0.25));
        assert_eq!(layer.health(), 0.75);
    }

    #[test]
    fn layer_health_cannot_be_restored_above_one() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0)).with_health(0.5);
        layer.execute_control_request(fully_budgeted_healing_request(0, 1.0));
        assert_eq!(layer.health(), 1.0);
    }

    #[test]
    fn layer_health_restoration_is_limited_by_budgeted_fraction() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0)).with_health(0.5);
        layer.execute_control_request(budgeted(
            CellLayer::healing_request(0, 0.5),
            BioEnergyDelta::ZERO,
            0.5,
        ));
        assert_eq!(layer.health(), 0.75);
    }

    #[test]
    fn layer_costs_health_restoration() {
        let mut layer = simple_cell_layer(Area::new(2.0), Density::new(1.0))
            .with_health_parameters(LayerHealthParameters {
                healing_energy_delta: BioEnergyDelta::new(-3.0),
                ..LayerHealthParameters::DEFAULT
            })
            .with_health(0.5);
        let control_request = CellLayer::healing_request(0, 0.25);
        let costed_request = layer.cost_control_request(control_request);
        assert_eq!(
            costed_request,
            CostedControlRequest::new(control_request, BioEnergyDelta::new(-1.5))
        );
    }

    #[test]
    fn layer_undergoes_entropic_damage() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_health_parameters(LayerHealthParameters {
                entropic_damage_health_delta: -0.25,
                ..LayerHealthParameters::DEFAULT
            });

        let env = LocalEnvironment::new();
        layer.after_influences(&env, Duration::new(0.5));

        assert_eq!(layer.health(), 0.875);
    }

    #[test]
    fn overlap_damages_surface_layer() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_health_parameters(LayerHealthParameters {
                overlap_damage_health_delta: -0.25,
                ..LayerHealthParameters::DEFAULT
            });
        layer.mark_as_surface();

        let mut env = LocalEnvironment::new();
        env.add_overlap(Overlap::new(Displacement::new(0.5, 0.0)));
        layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(layer.health(), 0.875);
    }

    #[test]
    fn overlap_does_not_damage_interior_layer() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_health_parameters(LayerHealthParameters {
                overlap_damage_health_delta: -0.25,
                ..LayerHealthParameters::DEFAULT
            });

        let mut env = LocalEnvironment::new();
        env.add_overlap(Overlap::new(Displacement::new(0.5, 0.0)));
        layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(layer.health(), 1.0);
    }

    #[test]
    fn dead_layer_costs_control_requests_at_zero() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_health_parameters(LayerHealthParameters {
                healing_energy_delta: BioEnergyDelta::new(-1.0),
                ..LayerHealthParameters::DEFAULT
            })
            .dead();
        let control_request = CellLayer::healing_request(0, 1.0);
        let costed_request = layer.cost_control_request(control_request);
        assert_eq!(
            costed_request,
            CostedControlRequest::new(control_request, BioEnergyDelta::new(0.0))
        );
    }

    #[test]
    fn dead_layer_ignores_control_requests() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0)).dead();
        layer.execute_control_request(fully_budgeted_healing_request(0, 1.0));
        assert_eq!(layer.health(), 0.0);
    }

    #[test]
    fn thruster_layer_adds_force() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(ThrusterCellLayerSpecialty::new()),
        );
        layer.execute_control_request(fully_budgeted(ThrusterCellLayerSpecialty::force_x_request(
            0, 1.0,
        )));
        layer.execute_control_request(fully_budgeted(ThrusterCellLayerSpecialty::force_y_request(
            0, -1.0,
        )));

        let env = LocalEnvironment::new();
        let (_, force) = layer.after_influences(&env, Duration::new(0.5));

        assert_eq!(force, Force::new(1.0, -1.0));
    }

    #[test]
    fn thruster_layer_force_is_limited_by_budget() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(ThrusterCellLayerSpecialty::new()),
        );
        layer.execute_control_request(budgeted(
            ThrusterCellLayerSpecialty::force_x_request(0, 1.0),
            BioEnergyDelta::new(1.0),
            0.5,
        ));
        layer.execute_control_request(budgeted(
            ThrusterCellLayerSpecialty::force_y_request(0, -1.0),
            BioEnergyDelta::new(1.0),
            0.25,
        ));

        let env = LocalEnvironment::new();
        let (_, force) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(force, Force::new(0.5, -0.25));
    }

    #[test]
    fn thruster_layer_force_is_limited_by_health() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(ThrusterCellLayerSpecialty::new()),
        )
        .with_health(0.5);
        layer.execute_control_request(fully_budgeted(ThrusterCellLayerSpecialty::force_x_request(
            0, 1.0,
        )));
        layer.execute_control_request(fully_budgeted(ThrusterCellLayerSpecialty::force_y_request(
            0, -1.0,
        )));

        let env = LocalEnvironment::new();
        let (_, force) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(force, Force::new(0.5, -0.5));
    }

    #[test]
    fn dead_thruster_layer_adds_no_force() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(ThrusterCellLayerSpecialty::new()),
        );
        layer.execute_control_request(fully_budgeted(ThrusterCellLayerSpecialty::force_x_request(
            0, 1.0,
        )));
        layer.execute_control_request(fully_budgeted(ThrusterCellLayerSpecialty::force_y_request(
            0, -1.0,
        )));
        layer.damage(1.0);

        let env = LocalEnvironment::new();
        let (_, force) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(force, Force::new(0.0, 0.0));
    }

    #[test]
    fn photo_layer_adds_energy_based_on_area_and_efficiency_and_duration() {
        let mut layer = CellLayer::new(
            Area::new(4.0),
            Density::new(1.0),
            Color::Green,
            Box::new(PhotoCellLayerSpecialty::new(0.5)),
        );

        let mut env = LocalEnvironment::new();
        env.add_light_intensity(10.0);

        let (energy, _) = layer.after_influences(&env, Duration::new(0.5));

        assert_eq!(energy, BioEnergy::new(10.0));
    }

    #[test]
    fn photo_layer_energy_is_limited_by_health() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(PhotoCellLayerSpecialty::new(1.0)),
        )
        .with_health(0.75);

        let mut env = LocalEnvironment::new();
        env.add_light_intensity(1.0);

        let (energy, _) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(energy, BioEnergy::new(0.75));
    }

    #[test]
    fn dead_photo_layer_adds_no_energy() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(PhotoCellLayerSpecialty::new(1.0)),
        )
        .dead();

        let mut env = LocalEnvironment::new();
        env.add_light_intensity(1.0);

        let (energy, _) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(energy, BioEnergy::new(0.0));
    }

    #[test]
    fn energy_generating_layer_adds_energy() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(EnergyGeneratingCellLayerSpecialty::new()),
        );

        layer.execute_control_request(fully_budgeted(
            EnergyGeneratingCellLayerSpecialty::energy_request(0, BioEnergy::new(2.0)),
        ));

        let env = LocalEnvironment::new();
        let (energy, _) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(energy, BioEnergy::new(2.0));
    }

    #[test]
    fn energy_generating_layer_does_not_remember_previous_requested_energy() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(EnergyGeneratingCellLayerSpecialty::new()),
        );

        layer.execute_control_request(fully_budgeted(
            EnergyGeneratingCellLayerSpecialty::energy_request(0, BioEnergy::new(2.0)),
        ));
        layer.after_control_requests(&CellStateSnapshot::ZEROS);

        let env = LocalEnvironment::new();
        let (energy, _) = layer.after_influences(&env, Duration::new(1.0));

        assert_eq!(energy, BioEnergy::ZERO);
    }

    #[test]
    fn budding_layer_creates_child_with_right_state() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(BuddingCellLayerSpecialty::new(create_child)),
        );
        layer.execute_control_request(fully_budgeted(
            BuddingCellLayerSpecialty::budding_angle_request(0, Angle::from_radians(0.0)),
        ));
        layer.execute_control_request(fully_budgeted(
            BuddingCellLayerSpecialty::donation_energy_request(0, BioEnergy::new(1.0)),
        ));
        let cell_state = CellStateSnapshot {
            radius: Length::new(2.0),
            area: Area::new(3.0),
            center: Position::new(1.0, -1.0),
            velocity: Velocity::new(2.0, -2.0),
        };
        match layer.after_control_requests(&cell_state) {
            None => panic!(),
            Some(child) => {
                assert_eq!(2, child.layers().len());
                assert_eq!(
                    Position::new(
                        cell_state.center.x() + cell_state.radius.value() + child.radius().value(),
                        cell_state.center.y(),
                    ),
                    child.center()
                );
                assert_eq!(child.velocity(), cell_state.velocity);
                assert_eq!(child.energy(), BioEnergy::new(1.0));
            }
        }
    }

    #[test]
    fn budding_layer_does_not_create_child_if_given_zero_energy() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(BuddingCellLayerSpecialty::new(create_child)),
        );
        layer.execute_control_request(fully_budgeted(
            BuddingCellLayerSpecialty::donation_energy_request(0, BioEnergy::new(0.0)),
        ));
        assert_eq!(
            layer.after_control_requests(&CellStateSnapshot::ZEROS),
            None
        );
    }

    #[test]
    fn budding_layer_does_not_remember_previous_donation_energy() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(BuddingCellLayerSpecialty::new(create_child)),
        );
        layer.execute_control_request(fully_budgeted(
            BuddingCellLayerSpecialty::donation_energy_request(0, BioEnergy::new(1.0)),
        ));
        layer.after_control_requests(&CellStateSnapshot::ZEROS);
        assert_eq!(
            layer.after_control_requests(&CellStateSnapshot::ZEROS),
            None
        );
    }

    #[test]
    fn budding_energy_is_limited_by_budget() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(BuddingCellLayerSpecialty::new(create_child)),
        );
        layer.execute_control_request(budgeted(
            BuddingCellLayerSpecialty::donation_energy_request(0, BioEnergy::new(1.0)),
            BioEnergyDelta::new(1.0),
            0.5,
        ));
        match layer.after_control_requests(&CellStateSnapshot::ZEROS) {
            None => panic!(),
            Some(child) => assert_eq!(BioEnergy::new(0.5), child.energy()),
        }
    }

    #[test]
    fn budding_energy_is_limited_by_health() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(BuddingCellLayerSpecialty::new(create_child)),
        )
        .with_health(0.5);
        layer.execute_control_request(fully_budgeted(
            BuddingCellLayerSpecialty::donation_energy_request(0, BioEnergy::new(1.0)),
        ));
        match layer.after_control_requests(&CellStateSnapshot::ZEROS) {
            None => panic!(),
            Some(child) => assert_eq!(BioEnergy::new(0.5), child.energy()),
        }
    }

    fn create_child() -> Cell {
        Cell::new(
            Position::ORIGIN,
            Velocity::ZERO,
            vec![
                Box::new(simple_cell_layer(Area::new(PI), Density::new(1.0))),
                Box::new(simple_cell_layer(Area::new(PI), Density::new(1.0))),
            ],
        )
    }

    fn simple_cell_layer(area: Area, density: Density) -> CellLayer {
        CellLayer::new(
            area,
            density,
            Color::Green,
            Box::new(NullCellLayerSpecialty::new()),
        )
    }

    fn fully_budgeted_healing_request(layer_index: usize, value: f64) -> BudgetedControlRequest {
        fully_budgeted(CellLayer::healing_request(layer_index, value))
    }

    fn fully_budgeted_resize_request(layer_index: usize, value: f64) -> BudgetedControlRequest {
        fully_budgeted(CellLayer::resize_request(
            layer_index,
            AreaDelta::new(value),
        ))
    }

    fn fully_budgeted(control_request: ControlRequest) -> BudgetedControlRequest {
        budgeted(control_request, BioEnergyDelta::ZERO, 1.0)
    }

    fn budgeted(
        control_request: ControlRequest,
        cost: BioEnergyDelta,
        budgeted_fraction: f64,
    ) -> BudgetedControlRequest {
        BudgetedControlRequest::new(
            CostedControlRequest::new(control_request, cost),
            budgeted_fraction,
        )
    }
}
