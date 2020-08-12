use crate::biology::control_requests::*;
use crate::physics::quantities::*;
use std::fmt;

#[derive(Debug, Clone)]
pub struct CellChanges {
    pub energy: BioEnergyDelta,
    pub energy_changes: Option<Vec<EnergyChange>>,
    pub thrust: Force,
    pub layers: Vec<CellLayerChanges>,
    pub bond_requests: BondRequests,
}

impl CellChanges {
    pub fn new(num_layers: usize, selected: bool) -> Self {
        CellChanges {
            energy: BioEnergyDelta::ZERO,
            energy_changes: if selected { Some(vec![]) } else { None },
            thrust: Force::ZERO,
            layers: vec![CellLayerChanges::new(selected); num_layers],
            bond_requests: NONE_BOND_REQUESTS,
        }
    }

    pub fn add_energy_change(
        &mut self,
        energy_delta: BioEnergyDelta,
        label: &'static str,
        index: usize,
    ) {
        self.energy += energy_delta;

        if let Some(energy_changes) = &mut self.energy_changes {
            energy_changes.push(EnergyChange {
                energy_delta,
                label,
                index,
            });
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnergyChange {
    pub energy_delta: BioEnergyDelta,
    pub label: &'static str,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct CellLayerChanges {
    pub health: HealthDelta,
    pub requested_health: HealthDelta,
    pub allowed_health: HealthDelta,
    pub health_changes: Option<Vec<HealthChange>>,
    pub area: AreaDelta,
    pub requested_area: AreaDelta,
    pub allowed_area: AreaDelta,
}

impl CellLayerChanges {
    pub fn new(selected: bool) -> Self {
        CellLayerChanges {
            health: HealthDelta::ZERO,
            requested_health: HealthDelta::ZERO,
            allowed_health: HealthDelta::ZERO,
            health_changes: if selected { Some(vec![]) } else { None },
            area: AreaDelta::ZERO,
            requested_area: AreaDelta::ZERO,
            allowed_area: AreaDelta::ZERO,
        }
    }

    pub fn add_healing(&mut self, health_delta: HealthDelta, request: &BudgetedControlRequest) {
        self.add_health_change(health_delta, "healing");
        self.requested_health = HealthDelta::new(request.requested_value());
        self.allowed_health = HealthDelta::new(request.allowed_value());
    }

    pub fn add_resize(&mut self, area_delta: AreaDelta, request: &BudgetedControlRequest) {
        self.area += area_delta;
        self.requested_area = AreaDelta::new(request.requested_value());
        self.allowed_area = AreaDelta::new(request.allowed_value());
    }

    pub fn add_health_change(&mut self, health_delta: HealthDelta, label: &'static str) {
        self.health += health_delta;

        if let Some(health_changes) = &mut self.health_changes {
            health_changes.push(HealthChange {
                health_delta,
                label,
            });
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HealthChange {
    pub health_delta: HealthDelta,
    pub label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct BondRequest {
    pub retain_bond: bool,
    pub budding_angle: Angle,
    pub donation_energy: BioEnergy,
}

impl BondRequest {
    pub const MAX_BONDS: usize = 8;

    pub const NONE: BondRequest = BondRequest {
        retain_bond: false,
        budding_angle: Angle::ZERO,
        donation_energy: BioEnergy::ZERO,
    };

    pub fn reset(&mut self) {
        *self = Self::NONE;
    }
}

impl fmt::Display for BondRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(retain: {}, angle: {:.4}, energy: {:.4})",
            self.retain_bond,
            self.budding_angle.radians(),
            self.donation_energy.value(),
        )
    }
}

pub type BondRequests = [BondRequest; BondRequest::MAX_BONDS];

pub const NONE_BOND_REQUESTS: BondRequests = [BondRequest::NONE; BondRequest::MAX_BONDS];
