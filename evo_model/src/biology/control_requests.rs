use crate::physics::quantities::*;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlRequest {
    layer_index: u16,
    channel_index: u16,
    value_index: u16,
    requested_value: f64,
}

impl ControlRequest {
    pub const ZEROS: ControlRequest = ControlRequest {
        layer_index: 0,
        channel_index: 0,
        value_index: 0,
        requested_value: 0.0,
    };

    pub fn new(
        layer_index: usize,
        channel_index: usize,
        value_index: usize,
        requested_value: f64,
    ) -> Self {
        ControlRequest {
            layer_index: layer_index as u16,
            channel_index: channel_index as u16,
            value_index: value_index as u16,
            requested_value,
        }
    }

    pub fn layer_index(&self) -> usize {
        self.layer_index as usize
    }

    pub fn channel_index(&self) -> usize {
        self.channel_index as usize
    }

    pub fn value_index(&self) -> usize {
        self.value_index as usize
    }

    pub fn requested_value(&self) -> f64 {
        self.requested_value
    }
}

pub struct ControlRequestId {
    layer_index: u16,
    channel_index: u16,
    value_index: u16,
}

impl ControlRequestId {
    pub const ZEROS: ControlRequestId = ControlRequestId {
        layer_index: 0,
        channel_index: 0,
        value_index: 0,
    };

    pub fn new(layer_index: usize, channel_index: usize, value_index: usize) -> Self {
        ControlRequestId {
            layer_index: layer_index as u16,
            channel_index: channel_index as u16,
            value_index: value_index as u16,
        }
    }

    pub fn layer_index(&self) -> usize {
        self.layer_index as usize
    }

    pub fn channel_index(&self) -> usize {
        self.channel_index as usize
    }

    pub fn value_index(&self) -> usize {
        self.value_index as usize
    }
}

impl fmt::Display for ControlRequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            self.layer_index, self.channel_index, self.value_index,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CostedControlRequest {
    layer_index: u16,
    channel_index: u16,
    value_index: u16,
    requested_value: f64,
    allowed_value: f64,
    energy_delta: BioEnergyDelta,
}

impl CostedControlRequest {
    pub const NULL_REQUEST: CostedControlRequest = CostedControlRequest {
        layer_index: 0,
        channel_index: 0,
        value_index: 0,
        requested_value: 0.0,
        allowed_value: 0.0,
        energy_delta: BioEnergyDelta::ZERO,
    };

    pub fn new(control_request: ControlRequest, energy_delta: BioEnergyDelta) -> Self {
        CostedControlRequest {
            layer_index: control_request.layer_index,
            channel_index: control_request.channel_index,
            value_index: control_request.value_index,
            requested_value: control_request.requested_value,
            allowed_value: 0.0,
            energy_delta,
        }
    }

    pub fn layer_index(&self) -> usize {
        self.layer_index as usize
    }

    pub fn channel_index(&self) -> usize {
        self.channel_index as usize
    }

    pub fn value_index(&self) -> usize {
        self.value_index as usize
    }

    pub fn requested_value(&self) -> f64 {
        self.requested_value
    }

    pub fn allowed_value(&self) -> f64 {
        self.allowed_value
    }

    pub fn energy_delta(&self) -> BioEnergyDelta {
        self.energy_delta
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BudgetedControlRequest {
    layer_index: u16,
    channel_index: u16,
    value_index: u16,
    requested_value: f64,
    allowed_value: f64,
    energy_delta: BioEnergyDelta,
    budgeted_fraction: f64,
}

impl BudgetedControlRequest {
    pub const NULL_REQUEST: BudgetedControlRequest = BudgetedControlRequest {
        layer_index: 0,
        channel_index: 0,
        value_index: 0,
        requested_value: 0.0,
        allowed_value: 0.0,
        energy_delta: BioEnergyDelta::ZERO,
        budgeted_fraction: 1.0,
    };

    pub fn new(costed_request: CostedControlRequest, budgeted_fraction: f64) -> Self {
        BudgetedControlRequest {
            layer_index: costed_request.layer_index,
            channel_index: costed_request.channel_index,
            value_index: costed_request.value_index,
            requested_value: costed_request.requested_value,
            allowed_value: costed_request.allowed_value,
            energy_delta: costed_request.energy_delta,
            budgeted_fraction,
        }
    }

    pub fn layer_index(&self) -> usize {
        self.layer_index as usize
    }

    pub fn channel_index(&self) -> usize {
        self.channel_index as usize
    }

    pub fn value_index(&self) -> usize {
        self.value_index as usize
    }

    pub fn requested_value(&self) -> f64 {
        self.requested_value
    }

    pub fn allowed_value(&self) -> f64 {
        self.allowed_value
    }

    pub fn energy_delta(&self) -> BioEnergyDelta {
        self.energy_delta
    }

    pub fn budgeted_fraction(&self) -> f64 {
        self.budgeted_fraction
    }
}

impl fmt::Display for BudgetedControlRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}: requested: {:.4}, allowed: {:.4}, cost: {:.4}, budget: {:.4})",
            self.layer_index,
            self.channel_index,
            self.value_index,
            self.requested_value,
            self.allowed_value,
            -self.energy_delta.value(),
            self.budgeted_fraction
        )
    }
}
