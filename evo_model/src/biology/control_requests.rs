use physics::quantities::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlRequest {
    pub layer_index: usize,
    pub control_index: usize,
    pub control_value: f64,
}

impl ControlRequest {
    pub fn new(layer_index: usize, control_index: usize, control_value: f64) -> Self {
        ControlRequest {
            layer_index,
            control_index,
            control_value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CostedControlRequest {
    pub control_request: ControlRequest,
    pub energy_delta: BioEnergyDelta,
}

impl CostedControlRequest {
    pub fn new(control_request: ControlRequest, energy_delta: BioEnergyDelta) -> Self {
        CostedControlRequest {
            control_request,
            energy_delta,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BudgetedControlRequest {
    pub control_request: ControlRequest,
    pub energy_delta: BioEnergyDelta,
    pub budgeted_fraction: f64,
}

impl BudgetedControlRequest {
    pub fn new(control_request: ControlRequest, energy_delta: BioEnergyDelta, budgeted_fraction: f64) -> Self {
        BudgetedControlRequest {
            control_request,
            energy_delta,
            budgeted_fraction,
        }
    }
}
