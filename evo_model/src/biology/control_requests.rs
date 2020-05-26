use crate::physics::quantities::*;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlRequest {
    id: ControlRequestId,
    requested_value: f64,
}

impl ControlRequest {
    pub const NULL_REQUEST: ControlRequest = ControlRequest {
        id: ControlRequestId::ZEROS,
        requested_value: 0.0,
    };

    pub fn new(
        layer_index: usize,
        channel_index: usize,
        value_index: usize,
        requested_value: f64,
    ) -> Self {
        ControlRequest {
            id: ControlRequestId::new(layer_index, channel_index, value_index),
            requested_value,
        }
    }

    pub fn layer_index(&self) -> usize {
        self.id.layer_index()
    }

    pub fn channel_index(&self) -> usize {
        self.id.channel_index()
    }

    pub fn value_index(&self) -> usize {
        self.id.value_index()
    }

    pub fn requested_value(&self) -> f64 {
        self.requested_value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

    pub fn layer_index(self) -> usize {
        self.layer_index as usize
    }

    pub fn channel_index(self) -> usize {
        self.channel_index as usize
    }

    pub fn value_index(self) -> usize {
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
    id: ControlRequestId,
    requested_value: f64,
    allowed_value: f64,
    energy_delta: BioEnergyDelta,
}

impl CostedControlRequest {
    pub const NULL_REQUEST: CostedControlRequest = CostedControlRequest {
        id: ControlRequestId::ZEROS,
        requested_value: 0.0,
        allowed_value: 0.0,
        energy_delta: BioEnergyDelta::ZERO,
    };

    pub fn new(control_request: ControlRequest, energy_delta: BioEnergyDelta) -> Self {
        CostedControlRequest {
            id: control_request.id,
            requested_value: control_request.requested_value,
            allowed_value: 0.0,
            energy_delta,
        }
    }

    pub fn layer_index(&self) -> usize {
        self.id.layer_index()
    }

    pub fn channel_index(&self) -> usize {
        self.id.channel_index()
    }

    pub fn value_index(&self) -> usize {
        self.id.value_index()
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
    id: ControlRequestId,
    requested_value: f64,
    allowed_value: f64,
    energy_delta: BioEnergyDelta,
    budgeted_fraction: f64,
}

impl BudgetedControlRequest {
    pub const NULL_REQUEST: BudgetedControlRequest = BudgetedControlRequest {
        id: ControlRequestId::ZEROS,
        requested_value: 0.0,
        allowed_value: 0.0,
        energy_delta: BioEnergyDelta::ZERO,
        budgeted_fraction: 1.0,
    };

    pub fn new(costed_request: CostedControlRequest, budgeted_fraction: f64) -> Self {
        BudgetedControlRequest {
            id: costed_request.id,
            requested_value: costed_request.requested_value,
            allowed_value: costed_request.allowed_value,
            energy_delta: costed_request.energy_delta,
            budgeted_fraction,
        }
    }

    pub fn layer_index(&self) -> usize {
        self.id.layer_index()
    }

    pub fn channel_index(&self) -> usize {
        self.id.channel_index()
    }

    pub fn value_index(&self) -> usize {
        self.id.value_index()
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
            "{}: requested: {:.4}, allowed: {:.4}, cost: {:.4}, budget: {:.4})",
            self.id,
            self.requested_value,
            self.allowed_value,
            -self.energy_delta.value(),
            self.budgeted_fraction
        )
    }
}
