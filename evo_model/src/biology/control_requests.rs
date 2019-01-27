use physics::quantities::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlRequest {
    pub layer_index: usize,
    pub channel_index: usize,
    pub value: f64,
}

impl ControlRequest {
    pub const ZEROS: ControlRequest =
        ControlRequest { layer_index: 0, channel_index: 0, value: 0.0 };

    pub fn new(layer_index: usize, channel_index: usize, value: f64) -> Self {
        ControlRequest {
            layer_index,
            channel_index,
            value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CostedControlRequest {
    pub layer_index: usize,
    pub channel_index: usize,
    pub value: f64,
    pub energy_delta: BioEnergyDelta,
}

impl CostedControlRequest {
    pub const NULL_REQUEST: CostedControlRequest =
        CostedControlRequest {
            layer_index: 0,
            channel_index: 0,
            value: 0.0,
            energy_delta: BioEnergyDelta::ZERO,
        };

    pub fn new(control_request: ControlRequest, energy_delta: BioEnergyDelta) -> Self {
        CostedControlRequest {
            layer_index: control_request.layer_index,
            channel_index: control_request.channel_index,
            value: control_request.value,
            energy_delta,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BudgetedControlRequest {
    pub layer_index: usize,
    pub channel_index: usize,
    pub value: f64,
    pub energy_delta: BioEnergyDelta,
    pub budgeted_fraction: f64,
}

impl BudgetedControlRequest {
    pub const NULL_REQUEST: BudgetedControlRequest =
        BudgetedControlRequest {
            layer_index: 0,
            channel_index: 0,
            value: 0.0,
            energy_delta: BioEnergyDelta::ZERO,
            budgeted_fraction: 1.0,
        };

    pub fn new(costed_request: CostedControlRequest, budgeted_fraction: f64) -> Self {
        BudgetedControlRequest {
            layer_index: costed_request.layer_index,
            channel_index: costed_request.channel_index,
            value: costed_request.value,
            energy_delta: costed_request.energy_delta,
            budgeted_fraction,
        }
    }
}
