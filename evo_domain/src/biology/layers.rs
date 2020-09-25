use crate::biology::changes::*;
use crate::biology::control::*;
use crate::biology::control_requests::*;
use crate::environment::local_environment::LocalEnvironment;
use crate::physics::overlap::Overlap;
use crate::physics::quantities::*;
use std::f64;
use std::f64::consts::PI;
use std::fmt::Debug;
use std::usize;

// TODO rename as TissueType?
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    Green,
    White,
    Yellow,
}

#[derive(Debug, Clone, Copy)]
pub struct LayerParameters {
    pub minimum_intact_thickness: Fraction,
}

impl LayerParameters {
    pub const DEFAULT: LayerParameters = LayerParameters {
        minimum_intact_thickness: Fraction::ZERO,
    };

    fn validate(&self) {}
}

#[derive(Debug, Clone, Copy)]
pub struct LayerHealthParameters {
    pub healing_energy_delta: BioEnergyDelta,
    pub entropic_damage_health_delta: HealthDelta,
    pub overlap_damage_health_delta: HealthDelta,
}

impl LayerHealthParameters {
    pub const DEFAULT: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::ZERO,
        entropic_damage_health_delta: HealthDelta::ZERO,
        overlap_damage_health_delta: HealthDelta::ZERO,
    };

    fn validate(&self) {
        assert!(self.healing_energy_delta <= BioEnergyDelta::ZERO);
        assert!(self.entropic_damage_health_delta <= HealthDelta::ZERO);
        assert!(self.overlap_damage_health_delta <= HealthDelta::ZERO);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LayerResizeParameters {
    pub growth_energy_delta: BioEnergyDelta,
    pub max_growth_rate: Value1D,
    pub shrinkage_energy_delta: BioEnergyDelta,
    pub max_shrinkage_rate: Value1D,
}

impl LayerResizeParameters {
    pub const UNLIMITED: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::ZERO,
        max_growth_rate: f64::INFINITY,
        shrinkage_energy_delta: BioEnergyDelta::ZERO,
        max_shrinkage_rate: 1.0,
    };

    fn validate(&self) {
        assert!(self.growth_energy_delta.value() <= 0.0);
        assert!(self.max_growth_rate >= 0.0);
        // self.shrinkage_energy_delta can be negative or positive
        assert!(self.max_shrinkage_rate >= 0.0);
    }
}

#[derive(Debug)]
pub struct CellLayer {
    brain: &'static dyn CellLayerBrain,
    body: CellLayerBody,
    specialty: Box<dyn CellLayerSpecialty>,
}

impl CellLayer {
    const HEALING_CHANNEL_INDEX: usize = 0;
    const RESIZE_CHANNEL_INDEX: usize = 1;
    const LIVING_BRAIN: LivingCellLayerBrain = LivingCellLayerBrain {};
    const DEAD_BRAIN: DeadCellLayerBrain = DeadCellLayerBrain {};

    pub fn new(
        area: Area,
        density: Density,
        color: Color,
        specialty: Box<dyn CellLayerSpecialty>,
    ) -> Self {
        CellLayer {
            brain: &CellLayer::LIVING_BRAIN,
            body: CellLayerBody::new(area, density, color),
            specialty,
        }
    }

    fn new_with_radii(
        outer_radius: Length,
        inner_radius: Length,
        density: Density,
        color: Color,
        specialty: Box<dyn CellLayerSpecialty>,
    ) -> Self {
        CellLayer {
            brain: &CellLayer::LIVING_BRAIN,
            body: CellLayerBody::new_with_radii(outer_radius, inner_radius, density, color),
            specialty,
        }
    }

    pub fn with_parameters(mut self, parameters: &'static LayerParameters) -> Self {
        parameters.validate();
        self.body.parameters = parameters;
        self
    }

    pub fn with_health_parameters(
        mut self,
        health_parameters: &'static LayerHealthParameters,
    ) -> Self {
        health_parameters.validate();
        self.body.health_parameters = health_parameters;
        self
    }

    pub fn with_resize_parameters(
        mut self,
        resize_parameters: &'static LayerResizeParameters,
    ) -> Self {
        resize_parameters.validate();
        self.body.resize_parameters = resize_parameters;
        self
    }

    pub fn with_health(mut self, health: Health) -> Self {
        self.body.health = health;
        self
    }

    pub fn dead(mut self) -> Self {
        self.die();
        self
    }

    pub fn die(&mut self) {
        self.update_health(HealthDelta::new(-1.0));
    }

    pub fn spawn(&self, area: Area) -> Self {
        Self {
            brain: &CellLayer::LIVING_BRAIN,
            body: self.body.spawn(area),
            specialty: self.specialty.spawn(),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.brain.is_alive()
    }

    pub fn is_intact(&self) -> bool {
        self.body.is_intact()
    }

    pub fn outer_radius(&self) -> Length {
        self.body.outer_radius
    }

    pub fn inner_radius(&self) -> Length {
        self.body.inner_radius()
    }

    pub fn color(&self) -> Color {
        self.body.color
    }

    pub fn health(&self) -> Health {
        self.body.health
    }

    pub fn area(&self) -> Area {
        self.body.area
    }

    pub fn mass(&self) -> Mass {
        self.body.mass
    }

    pub fn update_outer_radius(&mut self, inner_radius: Length) {
        self.body.update_outer_radius(inner_radius);
    }

    pub fn calculate_automatic_changes(
        &mut self,
        env: &LocalEnvironment,
        changes: &mut CellChanges,
        layer_index: usize,
    ) {
        self.brain.calculate_automatic_changes(
            &*self.specialty,
            &self.body,
            env,
            changes,
            layer_index,
        );
    }

    pub fn cost_control_request(&self, request: &ControlRequest) -> CostedControlRequest {
        self.brain
            .cost_control_request(&*self.specialty, &self.body, request)
    }

    pub fn execute_control_request(
        &self,
        request: &BudgetedControlRequest,
        changes: &mut CellChanges,
    ) {
        self.brain
            .execute_control_request(&*self.specialty, &self.body, request, changes);
    }

    pub fn apply_changes(&mut self, changes: &CellLayerChanges) {
        self.update_health(changes.health);
        self.body.resize(changes.area);
    }

    fn update_health(&mut self, delta_health: HealthDelta) {
        self.brain = self.body.update_health(delta_health);
    }

    pub fn healing_request(layer_index: usize, delta_health: HealthDelta) -> ControlRequest {
        if delta_health < HealthDelta::ZERO {
            panic!("Negative healing request");
        }

        ControlRequest::new(
            layer_index,
            Self::HEALING_CHANNEL_INDEX,
            0,
            delta_health.value(),
        )
    }

    pub fn resize_request(layer_index: usize, delta_area: AreaDelta) -> ControlRequest {
        ControlRequest::new(
            layer_index,
            Self::RESIZE_CHANNEL_INDEX,
            0,
            delta_area.value(),
        )
    }

    fn record_request_energy_change(
        request: &BudgetedControlRequest,
        label: &'static str,
        changes: &mut CellChanges,
    ) {
        changes.add_energy_change(
            request.budgeted_energy_delta(),
            label,
            request.layer_index(),
        );
    }

    pub fn print_tick_info(
        &self,
        index: usize,
        layer_start_snapshot: &CellLayerStateSnapshot,
        layer_changes: &CellLayerChanges,
    ) {
        println!(
            "  layer {}{}:",
            index,
            if self.is_alive() { "" } else { " (DEAD)" }
        );
        print_value1d_change_info(
            "    area",
            layer_start_snapshot.area.value(),
            self.area().value(),
        );
        println!(
            " [requested {:+.4}, allowed {:+.4}]",
            layer_changes.requested_area.value(),
            layer_changes.allowed_area.value()
        );
        println_value1d_change_info(
            "    mass",
            layer_start_snapshot.mass.value(),
            self.mass().value(),
        );
        print_value1d_change_info(
            "    health",
            layer_start_snapshot.health.value(),
            self.health().value(),
        );
        println!(
            " [requested {:+.4}, allowed {:+.4}]",
            layer_changes.requested_health.value(),
            layer_changes.allowed_health.value()
        );
        if let Some(health_changes) = &layer_changes.health_changes {
            for health_change in health_changes {
                println!(
                    "      {} {:+.4}",
                    health_change.label,
                    health_change.health_delta.value()
                );
            }
        }
    }
}

trait CellLayerBrain: Debug + Send + Sync {
    fn is_alive(&self) -> bool;

    fn calculate_automatic_changes(
        &self,
        specialty: &dyn CellLayerSpecialty,
        body: &CellLayerBody,
        env: &LocalEnvironment,
        changes: &mut CellChanges,
        layer_index: usize,
    );

    fn cost_control_request(
        &self,
        specialty: &dyn CellLayerSpecialty,
        body: &CellLayerBody,
        request: &ControlRequest,
    ) -> CostedControlRequest;

    fn execute_control_request(
        &self,
        specialty: &dyn CellLayerSpecialty,
        body: &CellLayerBody,
        request: &BudgetedControlRequest,
        changes: &mut CellChanges,
    );
}

#[derive(Debug)]
struct LivingCellLayerBrain {}

impl LivingCellLayerBrain {
    fn entropic_damage(&self, body: &CellLayerBody) -> HealthDelta {
        body.health_parameters.entropic_damage_health_delta
    }

    fn overlap_damage(&self, body: &CellLayerBody, overlaps: &[Overlap]) -> HealthDelta {
        overlaps.iter().fold(HealthDelta::ZERO, |damage, overlap| {
            damage + body.health_parameters.overlap_damage_health_delta * overlap.magnitude()
        })
    }
}

impl CellLayerBrain for LivingCellLayerBrain {
    fn is_alive(&self) -> bool {
        true
    }

    fn calculate_automatic_changes(
        &self,
        specialty: &dyn CellLayerSpecialty,
        body: &CellLayerBody,
        env: &LocalEnvironment,
        changes: &mut CellChanges,
        layer_index: usize,
    ) {
        changes.layers[layer_index].add_health_change(self.entropic_damage(body), "entropy");
        changes.layers[layer_index]
            .add_health_change(self.overlap_damage(body, env.overlaps()), "overlap");
        specialty.calculate_automatic_changes(body, env, changes)
    }

    fn cost_control_request(
        &self,
        specialty: &dyn CellLayerSpecialty,
        body: &CellLayerBody,
        request: &ControlRequest,
    ) -> CostedControlRequest {
        match request.channel_index() {
            CellLayer::HEALING_CHANNEL_INDEX => body.cost_restore_health(request),
            CellLayer::RESIZE_CHANNEL_INDEX => body.cost_resize(request),
            _ => specialty.cost_control_request(request, body),
        }
    }

    fn execute_control_request(
        &self,
        specialty: &dyn CellLayerSpecialty,
        body: &CellLayerBody,
        request: &BudgetedControlRequest,
        changes: &mut CellChanges,
    ) {
        match request.channel_index() {
            CellLayer::HEALING_CHANNEL_INDEX => {
                let delta_health = body.actual_delta_health(request);
                changes.layers[request.layer_index()].add_healing(delta_health, request);
                CellLayer::record_request_energy_change(&request, "healing", changes);
            }
            CellLayer::RESIZE_CHANNEL_INDEX => {
                let delta_area = body.actual_delta_area(request);
                changes.layers[request.layer_index()].add_resize(delta_area, request);
                CellLayer::record_request_energy_change(&request, "resize", changes);
            }
            _ => specialty.execute_control_request(body, request, changes),
        }
    }
}

#[derive(Debug)]
struct DeadCellLayerBrain {}

impl CellLayerBrain for DeadCellLayerBrain {
    fn is_alive(&self) -> bool {
        false
    }

    fn calculate_automatic_changes(
        &self,
        _specialty: &dyn CellLayerSpecialty,
        _body: &CellLayerBody,
        _env: &LocalEnvironment,
        _changes: &mut CellChanges,
        _layer_index: usize,
    ) {
    }

    fn cost_control_request(
        &self,
        _specialty: &dyn CellLayerSpecialty,
        _body: &CellLayerBody,
        request: &ControlRequest,
    ) -> CostedControlRequest {
        CostedControlRequest::free(request)
    }

    fn execute_control_request(
        &self,
        _specialty: &dyn CellLayerSpecialty,
        _body: &CellLayerBody,
        _request: &BudgetedControlRequest,
        _changes: &mut CellChanges,
    ) {
    }
}

// CellLayerBody is separate from CellLayer so it can be passed to CellLayerBrain.
#[derive(Debug)]
pub struct CellLayerBody {
    area: Area,
    density: Density,
    mass: Mass,
    outer_radius: Length,
    health: Health,
    color: Color,
    // TODO move to CellLayerParameters struct?
    parameters: &'static LayerParameters,
    health_parameters: &'static LayerHealthParameters,
    resize_parameters: &'static LayerResizeParameters,
}

impl CellLayerBody {
    fn new(area: Area, density: Density, color: Color) -> Self {
        let mut body = CellLayerBody {
            area,
            density,
            mass: Mass::ZERO,
            outer_radius: Length::ZERO,
            health: Health::FULL,
            color,
            parameters: &LayerParameters::DEFAULT,
            health_parameters: &LayerHealthParameters::DEFAULT,
            resize_parameters: &LayerResizeParameters::UNLIMITED,
        };
        body.init_from_area();
        body
    }

    pub fn new_with_radii(
        outer_radius: Length,
        inner_radius: Length,
        density: Density,
        color: Color,
    ) -> Self {
        let mut body = CellLayerBody {
            area: Area::ZERO,
            density,
            mass: Mass::ZERO,
            outer_radius,
            health: Health::FULL,
            color,
            parameters: &LayerParameters::DEFAULT,
            health_parameters: &LayerHealthParameters::DEFAULT,
            resize_parameters: &LayerResizeParameters::UNLIMITED,
        };
        body.init_from_radii(inner_radius);
        body
    }

    fn area(&self) -> Area {
        self.area
    }

    fn inner_radius(&self) -> Length {
        (self.outer_radius.sqr() - self.area / PI).sqrt()
    }

    fn health(&self) -> Health {
        self.health
    }

    fn is_intact(&self) -> bool {
        let thickness = self.outer_radius.value() - self.inner_radius().value();
        thickness / self.outer_radius.value() >= self.parameters.minimum_intact_thickness.value()
    }

    fn spawn(&self, area: Area) -> Self {
        let mut copy = Self {
            area,
            health: Health::FULL,
            ..*self
        };
        copy.init_from_area();
        copy
    }

    fn init_from_area(&mut self) {
        self.outer_radius = (self.area / PI).sqrt();
        self.mass = self.area * self.density;
    }

    fn init_from_radii(&mut self, inner_radius: Length) {
        self.area = PI * (self.outer_radius.sqr() - inner_radius.sqr());
        self.mass = self.area * self.density;
    }

    fn update_outer_radius(&mut self, inner_radius: Length) {
        self.outer_radius = (inner_radius.sqr() + self.area / PI).sqrt();
    }

    fn cost_restore_health(&self, request: &ControlRequest) -> CostedControlRequest {
        CostedControlRequest::unlimited(
            request,
            self.health_parameters.healing_energy_delta
                * self.area.value()
                * request.requested_value(),
        )
    }

    fn actual_delta_health(&self, request: &BudgetedControlRequest) -> HealthDelta {
        assert!(request.budgeted_value() >= 0.0);
        HealthDelta::new(request.budgeted_value())
    }

    fn update_health(&mut self, delta_health: HealthDelta) -> &'static dyn CellLayerBrain {
        self.health += delta_health;
        if self.health > Health::ZERO {
            &CellLayer::LIVING_BRAIN
        } else {
            &CellLayer::DEAD_BRAIN
        }
    }

    fn cost_resize(&self, request: &ControlRequest) -> CostedControlRequest {
        let allowed_delta_area = self.allowed_resize_delta_area(request.requested_value());
        let energy_delta_per_area = if request.requested_value() >= 0.0 {
            self.resize_parameters.growth_energy_delta
        } else {
            -self.resize_parameters.shrinkage_energy_delta
        };
        CostedControlRequest::limited(
            request,
            allowed_delta_area,
            allowed_delta_area * energy_delta_per_area,
        )
    }

    fn allowed_resize_delta_area(&self, requested_delta_area: Value1D) -> Value1D {
        if requested_delta_area >= 0.0 {
            self.allowed_growth_delta_area(requested_delta_area)
        } else {
            self.allowed_shrinkage_delta_area(requested_delta_area)
        }
    }

    fn allowed_growth_delta_area(&self, requested_delta_area: Value1D) -> Value1D {
        // TODO a layer that starts with area 0.0 cannot grow; add min-area param?
        let max_rate_limited_delta_area =
            self.resize_parameters.max_growth_rate * self.area.value();
        self.health.value() * requested_delta_area.min(max_rate_limited_delta_area)
    }

    fn allowed_shrinkage_delta_area(&self, requested_delta_area: Value1D) -> Value1D {
        let min_rate_limited_delta_area =
            -self.resize_parameters.max_shrinkage_rate * self.area.value();
        self.health.value() * requested_delta_area.max(min_rate_limited_delta_area)
    }

    fn actual_delta_area(&self, request: &BudgetedControlRequest) -> AreaDelta {
        AreaDelta::new(request.budgeted_value().max(-self.area.value()))
    }

    fn resize(&mut self, delta_area: AreaDelta) {
        self.area += delta_area;
        self.mass = self.area * self.density;
    }
}

trait CellLayerSpecialtySpawn {
    fn spawn(&self) -> Box<dyn CellLayerSpecialty>;
}

impl CellLayerSpecialtySpawn for Box<dyn CellLayerSpecialty> {
    fn spawn(&self) -> Box<dyn CellLayerSpecialty> {
        self.box_spawn()
    }
}

pub trait CellLayerSpecialty: Debug + Send + Sync {
    fn box_spawn(&self) -> Box<dyn CellLayerSpecialty>;

    fn calculate_automatic_changes(
        &self,
        _body: &CellLayerBody,
        _env: &LocalEnvironment,
        _changes: &mut CellChanges,
    ) {
    }

    // TODO implement and use this, e.g. for the invalid-index panic
    //    fn max_control_channel_index(&self) -> usize {
    //        CellLayer::RESIZE_CHANNEL_INDEX
    //    }

    fn cost_control_request(
        &self,
        request: &ControlRequest,
        _body: &CellLayerBody,
    ) -> CostedControlRequest {
        panic!("Invalid control channel index: {}", request.channel_index());
    }

    fn execute_control_request(
        &self,
        _body: &CellLayerBody,
        request: &BudgetedControlRequest,
        _changes: &mut CellChanges,
    ) {
        panic!("Invalid control channel index: {}", request.channel_index());
    }
}

#[derive(Debug)]
pub struct NullCellLayerSpecialty {}

impl NullCellLayerSpecialty {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        NullCellLayerSpecialty {}
    }
}

impl CellLayerSpecialty for NullCellLayerSpecialty {
    fn box_spawn(&self) -> Box<dyn CellLayerSpecialty> {
        Box::new(NullCellLayerSpecialty::new())
    }
}

#[derive(Clone, Debug)]
pub struct PhotoCellLayerSpecialty {
    efficiency: Fraction,
}

impl PhotoCellLayerSpecialty {
    pub fn new(efficiency: Fraction) -> Self {
        PhotoCellLayerSpecialty { efficiency }
    }
}

impl CellLayerSpecialty for PhotoCellLayerSpecialty {
    fn box_spawn(&self) -> Box<dyn CellLayerSpecialty> {
        Box::new(self.clone())
    }

    fn calculate_automatic_changes(
        &self,
        body: &CellLayerBody,
        env: &LocalEnvironment,
        changes: &mut CellChanges,
    ) {
        let energy = BioEnergy::new(
            env.light_intensity()
                * self.efficiency.value()
                * body.health.value()
                * body.area.value(),
        );
        changes.add_energy_change(energy.into(), "photo", usize::MAX);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BondingLayerParameters {
    pub max_donation_energy_per_unit_area: BioEnergy,
    pub donation_energy_tax_rate: Fraction,
}

impl BondingLayerParameters {
    pub const DEFAULT: BondingLayerParameters = BondingLayerParameters {
        max_donation_energy_per_unit_area: BioEnergy::MAX,
        donation_energy_tax_rate: Fraction::ZERO,
    };
}

#[derive(Debug)]
pub struct BondingCellLayerSpecialty {
    parameters: &'static BondingLayerParameters,
}

impl BondingCellLayerSpecialty {
    const RETAIN_BOND_CHANNEL_INDEX: usize = 2;
    const BUDDING_ANGLE_CHANNEL_INDEX: usize = 3;
    const DONATION_ENERGY_CHANNEL_INDEX: usize = 4;

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        BondingCellLayerSpecialty {
            parameters: &BondingLayerParameters::DEFAULT,
        }
    }

    pub fn with_parameters(mut self, parameters: &'static BondingLayerParameters) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn retain_bond_request(
        layer_index: usize,
        bond_index: usize,
        flag: bool,
    ) -> ControlRequest {
        ControlRequest::new(
            layer_index,
            Self::RETAIN_BOND_CHANNEL_INDEX,
            bond_index,
            if flag { 1.0 } else { 0.0 },
        )
    }

    pub fn budding_angle_request(
        layer_index: usize,
        bond_index: usize,
        angle: Angle,
    ) -> ControlRequest {
        ControlRequest::new(
            layer_index,
            Self::BUDDING_ANGLE_CHANNEL_INDEX,
            bond_index,
            angle.radians(),
        )
    }

    pub fn donation_energy_request(
        layer_index: usize,
        bond_index: usize,
        energy: BioEnergy,
    ) -> ControlRequest {
        ControlRequest::new(
            layer_index,
            Self::DONATION_ENERGY_CHANNEL_INDEX,
            bond_index,
            energy.value(),
        )
    }

    fn cost_donation_request(
        &self,
        request: &ControlRequest,
        body: &CellLayerBody,
    ) -> CostedControlRequest {
        let allowed_value = self.allowed_donation_energy(request.requested_value(), body);
        CostedControlRequest::limited(
            request,
            allowed_value,
            BioEnergyDelta::new(
                allowed_value * -(1.0 + self.parameters.donation_energy_tax_rate.value()),
            ),
        )
    }

    fn allowed_donation_energy(&self, requested_value: Value1D, body: &CellLayerBody) -> Value1D {
        let max_area_limited_donation =
            self.parameters.max_donation_energy_per_unit_area.value() * body.area().value();
        body.health().value() * requested_value.min(max_area_limited_donation)
    }
}

impl CellLayerSpecialty for BondingCellLayerSpecialty {
    fn box_spawn(&self) -> Box<dyn CellLayerSpecialty> {
        Box::new(BondingCellLayerSpecialty::new())
    }

    fn cost_control_request(
        &self,
        request: &ControlRequest,
        body: &CellLayerBody,
    ) -> CostedControlRequest {
        match request.channel_index() {
            Self::RETAIN_BOND_CHANNEL_INDEX => CostedControlRequest::free(request),
            Self::BUDDING_ANGLE_CHANNEL_INDEX => CostedControlRequest::free(request),
            Self::DONATION_ENERGY_CHANNEL_INDEX => self.cost_donation_request(request, body),
            _ => panic!("Invalid control channel index: {}", request.channel_index()),
        }
    }

    fn execute_control_request(
        &self,
        _body: &CellLayerBody,
        request: &BudgetedControlRequest,
        changes: &mut CellChanges,
    ) {
        let bond_request = &mut changes.bond_requests[request.value_index()];
        match request.channel_index() {
            Self::RETAIN_BOND_CHANNEL_INDEX => {
                bond_request.retain_bond = request.requested_value() > 0.0;
            }
            Self::BUDDING_ANGLE_CHANNEL_INDEX => {
                bond_request.budding_angle = Angle::from_radians(request.requested_value());
            }
            Self::DONATION_ENERGY_CHANNEL_INDEX => {
                bond_request.donation_energy =
                    request.budgeted_fraction().value() * BioEnergy::new(request.allowed_value());
                CellLayer::record_request_energy_change(&request, "donated", changes);
            }
            _ => panic!("Invalid control channel index: {}", request.channel_index()),
        }
    }
}

#[derive(Debug)]
pub struct ThrusterCellLayerSpecialty {}

impl ThrusterCellLayerSpecialty {
    const FORCE_X_CHANNEL_INDEX: usize = 2;
    const FORCE_Y_CHANNEL_INDEX: usize = 3;

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ThrusterCellLayerSpecialty {}
    }

    pub fn force_x_request(layer_index: usize, value: Value1D) -> ControlRequest {
        ControlRequest::new(layer_index, Self::FORCE_X_CHANNEL_INDEX, 0, value)
    }

    pub fn force_y_request(layer_index: usize, value: Value1D) -> ControlRequest {
        ControlRequest::new(layer_index, Self::FORCE_Y_CHANNEL_INDEX, 0, value)
    }
}

impl CellLayerSpecialty for ThrusterCellLayerSpecialty {
    fn box_spawn(&self) -> Box<dyn CellLayerSpecialty> {
        Box::new(ThrusterCellLayerSpecialty::new())
    }

    fn cost_control_request(
        &self,
        request: &ControlRequest,
        _body: &CellLayerBody,
    ) -> CostedControlRequest {
        match request.channel_index() {
            // TODO cost forces based on a parameter struct(?)
            Self::FORCE_X_CHANNEL_INDEX | Self::FORCE_Y_CHANNEL_INDEX => {
                CostedControlRequest::free(request)
            }
            _ => panic!("Invalid control channel index: {}", request.channel_index()),
        }
    }

    fn execute_control_request(
        &self,
        body: &CellLayerBody,
        request: &BudgetedControlRequest,
        changes: &mut CellChanges,
    ) {
        match request.channel_index() {
            Self::FORCE_X_CHANNEL_INDEX => {
                let force_x =
                    body.health.value() * request.budgeted_fraction() * request.requested_value();
                changes.thrust += Force::new(force_x, 0.0);
            }
            Self::FORCE_Y_CHANNEL_INDEX => {
                let force_y =
                    body.health.value() * request.budgeted_fraction() * request.requested_value();
                changes.thrust += Force::new(0.0, force_y);
            }
            _ => panic!("Invalid control channel index: {}", request.channel_index()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::control_requests::BudgetedControlRequest;
    use crate::environment::local_environment::LocalEnvironment;
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
    fn can_create_layer_from_radii() {
        let layer =
            simple_cell_layer_with_radii(Length::new(3.0), Length::new(2.0), Density::new(1.0));
        assert_eq!(layer.outer_radius(), Length::new(3.0));
        assert_eq!(layer.inner_radius(), Length::new(2.0));
    }

    #[test]
    fn layer_updates_outer_radius_based_on_inner_radius() {
        let mut layer = simple_cell_layer(Area::new(3.0 * PI), Density::new(1.0));
        layer.update_outer_radius(Length::new(1.0));
        assert_eq!(layer.outer_radius(), Length::new(2.0));
    }

    #[test]
    #[should_panic]
    fn healing_request_cannot_be_negative() {
        CellLayer::healing_request(0, HealthDelta::new(-0.25));
    }

    #[test]
    fn layer_costs_health_restoration() {
        const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
            healing_energy_delta: BioEnergyDelta::new(-3.0),
            ..LayerHealthParameters::DEFAULT
        };
        let layer = simple_cell_layer(Area::new(2.0), Density::new(1.0))
            .with_health_parameters(&LAYER_HEALTH_PARAMS)
            .with_health(Health::new(0.5));

        let control_request = CellLayer::healing_request(0, HealthDelta::new(0.25));
        let costed_request = layer.cost_control_request(&control_request);

        assert_eq!(
            costed_request,
            CostedControlRequest::unlimited(&control_request, BioEnergyDelta::new(-1.5))
        );
    }

    #[test]
    fn layer_health_restoration_is_limited_by_budgeted_fraction() {
        let layer =
            simple_cell_layer(Area::new(1.0), Density::new(1.0)).with_health(Health::new(0.5));
        let mut changes = CellChanges::new(1, false);
        layer.execute_control_request(
            &budgeted(
                &CellLayer::healing_request(0, HealthDelta::new(0.5)),
                BioEnergyDelta::ZERO,
                Fraction::new(0.5),
            ),
            &mut changes,
        );
        assert_eq!(changes.layers[0].health, HealthDelta::new(0.25));
    }

    #[test]
    fn layer_healing_records_energy_change() {
        let layer =
            simple_cell_layer(Area::new(1.0), Density::new(1.0)).with_health(Health::new(0.5));
        let mut changes = CellChanges::new(1, false);
        layer.execute_control_request(
            &budgeted(
                &CellLayer::healing_request(0, HealthDelta::new(0.5)),
                BioEnergyDelta::new(-10.0),
                Fraction::new(0.5),
            ),
            &mut changes,
        );
        assert_eq!(changes.energy, BioEnergyDelta::new(-5.0));
    }

    #[test]
    fn layer_bounds_and_costs_growth_request() {
        const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
            growth_energy_delta: BioEnergyDelta::new(-0.5),
            max_growth_rate: 2.0,
            ..LayerResizeParameters::UNLIMITED
        };
        let layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_resize_parameters(&LAYER_RESIZE_PARAMS)
            .with_health(Health::new(0.75));

        let resize_request = CellLayer::resize_request(0, AreaDelta::new(10.0));
        let costed_request = layer.cost_control_request(&resize_request);

        assert_eq!(
            costed_request,
            CostedControlRequest::limited(&resize_request, 1.5, BioEnergyDelta::new(-0.75))
        );
    }

    #[test]
    fn layer_growth_with_unlimited_rate_is_still_limited_by_health() {
        let layer =
            simple_cell_layer(Area::new(1.0), Density::new(1.0)).with_health(Health::new(0.5));

        let resize_request = CellLayer::resize_request(0, AreaDelta::new(10.0));
        let costed_request = layer.cost_control_request(&resize_request);

        assert_eq!(costed_request.allowed_value(), 5.0);
    }

    #[test]
    fn layer_bounds_and_costs_shrinkage_request() {
        const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
            shrinkage_energy_delta: BioEnergyDelta::new(0.5),
            max_shrinkage_rate: 0.5,
            ..LayerResizeParameters::UNLIMITED
        };
        let layer = simple_cell_layer(Area::new(10.0), Density::new(1.0))
            .with_resize_parameters(&LAYER_RESIZE_PARAMS)
            .with_health(Health::new(0.5));

        let resize_request = CellLayer::resize_request(0, AreaDelta::new(-9.0));
        let costed_request = layer.cost_control_request(&resize_request);

        assert_eq!(
            costed_request,
            CostedControlRequest::limited(&resize_request, -2.5, BioEnergyDelta::new(1.25))
        );
    }

    #[test]
    fn layer_shrinkage_with_unlimited_rate_is_still_limited_by_health() {
        let layer =
            simple_cell_layer(Area::new(10.0), Density::new(1.0)).with_health(Health::new(0.5));

        let resize_request = CellLayer::resize_request(0, AreaDelta::new(-10.0));
        let costed_request = layer.cost_control_request(&resize_request);

        assert_eq!(costed_request.allowed_value(), -5.0);
    }

    #[test]
    fn layer_growth_is_limited_to_budgeted_fraction_of_allowed_value() {
        let layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        let mut changes = CellChanges::new(1, false);

        layer.execute_control_request(
            &costed_and_budgeted(
                &CellLayer::resize_request(0, AreaDelta::new(10.0)),
                4.0,
                BioEnergyDelta::new(-3.0),
                Fraction::new(0.5),
            ),
            &mut changes,
        );

        assert_eq!(changes.layers[0].area, AreaDelta::new(2.0));
        assert_eq!(changes.energy, BioEnergyDelta::new(-1.5));
    }

    #[test]
    fn layer_undergoes_entropic_damage() {
        const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
            entropic_damage_health_delta: HealthDelta::new(-0.25),
            ..LayerHealthParameters::DEFAULT
        };

        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_health_parameters(&LAYER_HEALTH_PARAMS);

        let env = LocalEnvironment::new();
        let mut changes = CellChanges::new(1, false);
        layer.calculate_automatic_changes(&env, &mut changes, 0);

        assert_eq!(changes.layers[0].health, HealthDelta::new(-0.25));

        layer.apply_changes(&changes.layers[0]);
        assert_eq!(layer.health(), Health::new(0.75));
    }

    #[test]
    fn overlap_damages_layer() {
        const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
            overlap_damage_health_delta: HealthDelta::new(-0.25),
            ..LayerHealthParameters::DEFAULT
        };

        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_health_parameters(&LAYER_HEALTH_PARAMS);

        let mut env = LocalEnvironment::new();
        env.add_overlap(Overlap::new(Displacement::new(0.5, 0.0), 1.0));
        let mut changes = CellChanges::new(1, false);
        layer.calculate_automatic_changes(&env, &mut changes, 0);

        assert_eq!(changes.layers[0].health, HealthDelta::new(-0.125));

        layer.apply_changes(&changes.layers[0]);
        assert_eq!(layer.health(), Health::new(0.875));
    }

    #[test]
    fn applying_layer_changes_changes_health() {
        let mut layer =
            simple_cell_layer(Area::new(1.0), Density::new(1.0)).with_health(Health::new(0.5));
        let mut changes = CellLayerChanges::new(false);
        changes.health = HealthDelta::new(0.25);
        layer.apply_changes(&changes);
        assert_eq!(layer.health(), Health::new(0.75));
    }

    #[test]
    fn applying_layer_changes_can_kill_layer() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(1.0));
        let mut changes = CellLayerChanges::new(false);
        changes.health = HealthDelta::new(-1.0);
        layer.apply_changes(&changes);
        assert!(!layer.is_alive());
    }

    #[test]
    fn applying_layer_changes_resizes() {
        let mut layer = simple_cell_layer(Area::new(1.0), Density::new(2.0));
        let mut changes = CellLayerChanges::new(false);
        changes.area = AreaDelta::new(2.0);
        layer.apply_changes(&changes);
        assert_eq!(layer.area(), Area::new(3.0));
        assert_eq!(layer.mass(), Mass::new(6.0));
    }

    #[test]
    fn dead_layer_costs_control_requests_at_zero() {
        const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
            healing_energy_delta: BioEnergyDelta::new(-1.0),
            ..LayerHealthParameters::DEFAULT
        };

        let layer = simple_cell_layer(Area::new(1.0), Density::new(1.0))
            .with_health_parameters(&LAYER_HEALTH_PARAMS)
            .dead();
        let control_request = CellLayer::healing_request(0, HealthDelta::new(1.0));
        let costed_request = layer.cost_control_request(&control_request);
        assert_eq!(costed_request, CostedControlRequest::free(&control_request));
    }

    #[test]
    fn dead_layer_ignores_control_requests() {
        let layer = simple_cell_layer(Area::new(1.0), Density::new(1.0)).dead();
        let mut changes = CellChanges::new(1, false);
        layer.execute_control_request(&fully_budgeted_healing_request(0, 1.0), &mut changes);
        assert_eq!(layer.health(), Health::ZERO);
        assert_eq!(changes.layers[0].health, HealthDelta::ZERO);
    }

    #[test]
    fn layer_above_minimum_thickness_is_intact() {
        const LAYER_PARAMS: LayerParameters = LayerParameters {
            minimum_intact_thickness: Fraction::unchecked(0.1),
            ..LayerParameters::DEFAULT
        };

        let layer =
            simple_cell_layer_with_radii(Length::new(1.0), Length::new(0.8), Density::new(1.0))
                .with_parameters(&LAYER_PARAMS);

        assert!(layer.is_intact());
    }

    #[test]
    fn layer_below_minimum_thickness_is_not_intact() {
        const LAYER_PARAMS: LayerParameters = LayerParameters {
            minimum_intact_thickness: Fraction::unchecked(0.1),
            ..LayerParameters::DEFAULT
        };

        let layer =
            simple_cell_layer_with_radii(Length::new(1.0), Length::new(0.99), Density::new(1.0))
                .with_parameters(&LAYER_PARAMS);

        assert!(!layer.is_intact());
    }

    #[test]
    fn photo_layer_adds_energy_based_on_area_and_efficiency_and_duration() {
        let mut layer = CellLayer::new(
            Area::new(4.0),
            Density::new(1.0),
            Color::Green,
            Box::new(PhotoCellLayerSpecialty::new(Fraction::new(0.5))),
        );

        let mut env = LocalEnvironment::new();
        env.add_light_intensity(10.0);

        let mut changes = CellChanges::new(1, false);
        layer.calculate_automatic_changes(&env, &mut changes, 0);

        assert_eq!(changes.energy, BioEnergyDelta::new(20.0));
    }

    #[test]
    fn photo_layer_energy_is_limited_by_health() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(PhotoCellLayerSpecialty::new(Fraction::ONE)),
        )
        .with_health(Health::new(0.75));

        let mut env = LocalEnvironment::new();
        env.add_light_intensity(1.0);

        let mut changes = CellChanges::new(1, false);
        layer.calculate_automatic_changes(&env, &mut changes, 0);

        assert_eq!(changes.energy, BioEnergyDelta::new(0.75));
    }

    #[test]
    fn dead_photo_layer_adds_no_energy() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(PhotoCellLayerSpecialty::new(Fraction::ONE)),
        )
        .dead();

        let mut env = LocalEnvironment::new();
        env.add_light_intensity(1.0);

        let mut changes = CellChanges::new(1, false);
        layer.calculate_automatic_changes(&env, &mut changes, 0);

        assert_eq!(changes.energy, BioEnergyDelta::new(0.0));
    }

    #[test]
    fn bonding_layer_bounds_and_costs_donation_request() {
        const LAYER_PARAMS: BondingLayerParameters = BondingLayerParameters {
            max_donation_energy_per_unit_area: BioEnergy::unchecked(0.5),
            donation_energy_tax_rate: Fraction::unchecked(0.25),
        };
        let layer = CellLayer::new(
            Area::new(10.0),
            Density::new(1.0),
            Color::Yellow,
            Box::new(BondingCellLayerSpecialty::new().with_parameters(&LAYER_PARAMS)),
        )
        .with_health(Health::new(0.5));

        let donation_request =
            BondingCellLayerSpecialty::donation_energy_request(0, 1, BioEnergy::new(10.0));
        let costed_request = layer.cost_control_request(&donation_request);

        assert_eq!(
            costed_request,
            CostedControlRequest::limited(&donation_request, 2.5, BioEnergyDelta::new(-3.125))
        );
    }

    #[test]
    fn unlimited_donation_energy_is_still_limited_by_health() {
        let layer = CellLayer::new(
            Area::new(10.0),
            Density::new(1.0),
            Color::Yellow,
            Box::new(BondingCellLayerSpecialty::new()),
        )
        .with_health(Health::new(0.5));

        let donation_request =
            BondingCellLayerSpecialty::donation_energy_request(0, 1, BioEnergy::new(10.0));
        let costed_request = layer.cost_control_request(&donation_request);

        assert_eq!(
            costed_request,
            CostedControlRequest::limited(&donation_request, 5.0, BioEnergyDelta::new(-5.0))
        );
    }

    #[test]
    fn bonding_layer_collects_tax() {
        const LAYER_PARAMS: BondingLayerParameters = BondingLayerParameters {
            max_donation_energy_per_unit_area: BioEnergy::unchecked(0.5),
            donation_energy_tax_rate: Fraction::unchecked(0.25),
        };
        let layer = CellLayer::new(
            Area::new(10.0),
            Density::new(1.0),
            Color::Yellow,
            Box::new(BondingCellLayerSpecialty::new().with_parameters(&LAYER_PARAMS)),
        );

        let mut changes = CellChanges::new(1, false);
        layer.execute_control_request(
            &costed_and_budgeted(
                &BondingCellLayerSpecialty::donation_energy_request(0, 0, BioEnergy::new(10.0)),
                2.5,
                BioEnergyDelta::new(-3.125),
                Fraction::new(0.5),
            ),
            &mut changes,
        );

        assert_eq!(
            changes.bond_requests[0].donation_energy,
            BioEnergy::new(1.25)
        );
        assert_eq!(changes.energy, BioEnergyDelta::new(-1.5625));
    }

    #[test]
    fn thruster_layer_adds_force() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(ThrusterCellLayerSpecialty::new()),
        );
        let mut changes = CellChanges::new(1, false);
        layer.execute_control_request(
            &fully_budgeted(&ThrusterCellLayerSpecialty::force_x_request(0, 1.0)),
            &mut changes,
        );
        layer.execute_control_request(
            &fully_budgeted(&ThrusterCellLayerSpecialty::force_y_request(0, -1.0)),
            &mut changes,
        );

        let env = LocalEnvironment::new();
        let mut changes2 = CellChanges::new(1, false);
        layer.calculate_automatic_changes(&env, &mut changes2, 0);

        assert_eq!(changes.thrust, Force::new(1.0, -1.0));
    }

    #[test]
    fn thruster_layer_force_is_limited_by_budget() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(ThrusterCellLayerSpecialty::new()),
        );
        let mut changes = CellChanges::new(1, false);
        layer.execute_control_request(
            &budgeted(
                &ThrusterCellLayerSpecialty::force_x_request(0, 1.0),
                BioEnergyDelta::new(1.0),
                Fraction::new(0.5),
            ),
            &mut changes,
        );
        layer.execute_control_request(
            &budgeted(
                &ThrusterCellLayerSpecialty::force_y_request(0, -1.0),
                BioEnergyDelta::new(1.0),
                Fraction::new(0.25),
            ),
            &mut changes,
        );

        let env = LocalEnvironment::new();
        let mut changes2 = CellChanges::new(1, false);
        layer.calculate_automatic_changes(&env, &mut changes2, 0);

        assert_eq!(changes.thrust, Force::new(0.5, -0.25));
    }

    #[test]
    fn thruster_layer_force_is_limited_by_health() {
        let mut layer = CellLayer::new(
            Area::new(1.0),
            Density::new(1.0),
            Color::Green,
            Box::new(ThrusterCellLayerSpecialty::new()),
        )
        .with_health(Health::new(0.5));
        let mut changes = CellChanges::new(1, false);
        layer.execute_control_request(
            &fully_budgeted(&ThrusterCellLayerSpecialty::force_x_request(0, 1.0)),
            &mut changes,
        );
        layer.execute_control_request(
            &fully_budgeted(&ThrusterCellLayerSpecialty::force_y_request(0, -1.0)),
            &mut changes,
        );

        let env = LocalEnvironment::new();
        let mut changes2 = CellChanges::new(1, false);
        layer.calculate_automatic_changes(&env, &mut changes2, 0);

        assert_eq!(changes.thrust, Force::new(0.5, -0.5));
    }

    fn simple_cell_layer(area: Area, density: Density) -> CellLayer {
        CellLayer::new(
            area,
            density,
            Color::Green,
            Box::new(NullCellLayerSpecialty::new()),
        )
    }

    fn simple_cell_layer_with_radii(
        outer_radius: Length,
        inner_radius: Length,
        density: Density,
    ) -> CellLayer {
        CellLayer::new_with_radii(
            outer_radius,
            inner_radius,
            density,
            Color::Green,
            Box::new(NullCellLayerSpecialty::new()),
        )
    }

    fn fully_budgeted_healing_request(
        layer_index: usize,
        value: Value1D,
    ) -> BudgetedControlRequest {
        fully_budgeted(&CellLayer::healing_request(
            layer_index,
            HealthDelta::new(value),
        ))
    }

    fn fully_budgeted(control_request: &ControlRequest) -> BudgetedControlRequest {
        budgeted(control_request, BioEnergyDelta::ZERO, Fraction::ONE)
    }

    fn budgeted(
        control_request: &ControlRequest,
        energy_delta: BioEnergyDelta,
        budgeted_fraction: Fraction,
    ) -> BudgetedControlRequest {
        BudgetedControlRequest::new(
            &CostedControlRequest::unlimited(control_request, energy_delta),
            budgeted_fraction,
        )
    }

    fn costed_and_budgeted(
        control_request: &ControlRequest,
        allowed_value: Value1D,
        energy_delta: BioEnergyDelta,
        budgeted_fraction: Fraction,
    ) -> BudgetedControlRequest {
        BudgetedControlRequest::new(
            &CostedControlRequest::limited(control_request, allowed_value, energy_delta),
            budgeted_fraction,
        )
    }
}
