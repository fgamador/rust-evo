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
    pub fn new(num_layers: usize) -> Self {
        CellChanges {
            energy: BioEnergyDelta::ZERO,
            energy_changes: None,
            thrust: Force::ZERO,
            layers: vec![CellLayerChanges::new(); num_layers],
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

#[derive(Debug, Clone, Copy)]
pub struct CellLayerChanges {
    pub health: HealthDelta,
    pub area: AreaDelta,
}

impl CellLayerChanges {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        CellLayerChanges {
            health: HealthDelta::ZERO,
            area: AreaDelta::ZERO,
        }
    }
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
