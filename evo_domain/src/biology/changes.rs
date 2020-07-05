use crate::physics::quantities::*;

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
    pub energy: BioEnergy,
    pub thrust: Force,
    pub layers: Vec<CellLayerChanges>,
}

impl CellChanges {
    pub fn new(num_layers: usize) -> Self {
        CellChanges {
            energy: BioEnergy::ZERO,
            thrust: Force::ZERO,
            layers: vec![CellLayerChanges::new(); num_layers],
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
