use crate::physics::quantities::*;
use std::fmt;

// TODO lose this once we can tick a cell in one pass; move any world-scale changes into CellChanges
#[derive(Debug, Clone)]
pub struct WorldChanges {
    pub cells: Vec<CellChanges>,
    // TODO bonds, new_cells, dead_cells, new_bonds, broken_bonds
}

impl WorldChanges {
    pub fn new(num_cells: usize, num_layers: usize) -> Self {
        WorldChanges {
            cells: vec![CellChanges::new(num_layers); num_cells],
        }
    }
}

#[derive(Debug, Clone)]
pub struct CellChanges {
    pub energy: BioEnergyDelta,
    pub thrust: Force,
    pub layers: Vec<CellLayerChanges>,
    pub bond_requests: BondRequests,
}

impl CellChanges {
    pub fn new(num_layers: usize) -> Self {
        CellChanges {
            energy: BioEnergyDelta::ZERO,
            thrust: Force::ZERO,
            layers: vec![CellLayerChanges::new(); num_layers],
            bond_requests: NONE_BOND_REQUESTS,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CellLayerChanges {
    pub health: f64,
    pub area: AreaDelta,
}

impl CellLayerChanges {
    pub fn new() -> Self {
        CellLayerChanges {
            health: 0.0,
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
