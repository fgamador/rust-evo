use crate::biology::control_requests::*;
use crate::biology::genome::*;
use crate::biology::layers::*;
use crate::physics::quantities::*;
use std::fmt::Debug;

pub trait CellControl: Debug {
    fn run(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest>;

    fn spawn(&mut self) -> Box<dyn CellControl>;
}

#[derive(Debug)]
pub struct CellStateSnapshot {
    pub radius: Length,
    pub area: Area,
    pub mass: Mass,
    pub center: Position,
    pub velocity: Velocity,
    pub energy: BioEnergy,
    pub layers: Vec<CellLayerStateSnapshot>,
    pub bond_0_exists: bool,
}

impl CellStateSnapshot {
    pub const ZEROS: CellStateSnapshot = CellStateSnapshot {
        radius: Length::ZERO,
        area: Area::ZERO,
        mass: Mass::ZERO,
        center: Position::ORIGIN,
        velocity: Velocity::ZERO,
        energy: BioEnergy::ZERO,
        layers: Vec::new(),
        bond_0_exists: false,
    };
}

#[derive(Debug)]
pub struct CellLayerStateSnapshot {
    pub area: Area,
    pub mass: Mass,
    pub health: Health,
}

#[derive(Debug)]
pub struct NullControl {}

impl NullControl {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        NullControl {}
    }
}

impl CellControl for NullControl {
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(Self::new())
    }
}

#[derive(Clone, Debug)]
pub struct ContinuousRequestsControl {
    requests: Vec<ControlRequest>,
}

impl ContinuousRequestsControl {
    pub fn new(requests: Vec<ControlRequest>) -> Self {
        ContinuousRequestsControl { requests }
    }
}

impl CellControl for ContinuousRequestsControl {
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        self.requests.clone()
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct ContinuousResizeControl {
    layer_index: usize,
    resize_amount: AreaDelta,
}

impl ContinuousResizeControl {
    pub fn new(layer_index: usize, resize_amount: AreaDelta) -> Self {
        ContinuousResizeControl {
            layer_index,
            resize_amount,
        }
    }
}

impl CellControl for ContinuousResizeControl {
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![CellLayer::resize_request(
            self.layer_index,
            self.resize_amount,
        )]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Debug)]
pub struct SimpleThrusterControl {
    thruster_layer_index: usize,
    force: Force,
}

impl SimpleThrusterControl {
    pub fn new(thruster_layer_index: usize, force: Force) -> Self {
        SimpleThrusterControl {
            thruster_layer_index,
            force,
        }
    }
}

impl CellControl for SimpleThrusterControl {
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![
            ControlRequest::new(self.thruster_layer_index, 2, 0, self.force.x()),
            ControlRequest::new(self.thruster_layer_index, 3, 0, self.force.y()),
        ]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}

#[derive(Debug)]
pub struct NeuralNetControl {
    nnet: SparseNeuralNet,
    randomness: SeededMutationRandomness,
}

impl NeuralNetControl {
    fn new(genome: SparseNeuralNetGenome, randomness: SeededMutationRandomness) -> Self {
        NeuralNetControl {
            nnet: SparseNeuralNet::new(genome),
            randomness,
        }
    }
}

impl CellControl for NeuralNetControl {
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(NeuralNetControl {
            nnet: self.nnet.spawn(&mut self.randomness),
            randomness: self.randomness.clone(),
        })
    }
}

pub struct NeuralNetControlBuilder {
    genome: SparseNeuralNetGenome,
}

impl NeuralNetControlBuilder {
    pub fn new(transfer_fn: TransferFn) -> Self {
        NeuralNetControlBuilder {
            genome: SparseNeuralNetGenome::new(transfer_fn),
        }
    }

    pub fn add_input_node<F>(&mut self, _get_value: F) -> VecIndex
    where
        F: Fn(&CellStateSnapshot) -> NodeValue,
    {
        0
    }

    pub fn add_output_node<F>(
        &mut self,
        _from_value_weights: &[(VecIndex, Coefficient)],
        _bias: Coefficient,
        _to_request: F,
    ) -> VecIndex
    where
        F: Fn(NodeValue) -> ControlRequest,
    {
        0
    }

    pub fn build(self, randomness: SeededMutationRandomness) -> NeuralNetControl {
        NeuralNetControl::new(self.genome, randomness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continuous_resize_control_returns_request_to_grow_specified_layer() {
        let mut control = ContinuousResizeControl::new(1, AreaDelta::new(0.5));
        let requests = control.run(&CellStateSnapshot::ZEROS);
        assert_eq!(
            requests,
            vec![CellLayer::resize_request(1, AreaDelta::new(0.5))]
        );
    }

    #[test]
    fn simple_thruster_control_returns_requests_for_force() {
        let mut control = SimpleThrusterControl::new(2, Force::new(1.0, -1.0));
        let requests = control.run(&CellStateSnapshot::ZEROS);
        assert_eq!(
            requests,
            vec![
                ControlRequest::new(2, 2, 0, 1.0),
                ControlRequest::new(2, 3, 0, -1.0)
            ]
        );
    }

    #[test]
    #[ignore]
    fn can_build_neural_net_control() {
        let mut builder = NeuralNetControlBuilder::new(TransferFn::IDENTITY);

        let energy_input_index =
            builder.add_input_node(|cell_state| cell_state.energy.value() as NodeValue);
        builder.add_output_node(&[(energy_input_index, 10.0)], 2.0, |value| {
            CellLayer::resize_request(0, AreaDelta::new(value as f64))
        });

        let mut control = builder.build(SeededMutationRandomness::new(
            0,
            &MutationParameters::NO_MUTATION,
        ));
        let requests = control.run(&CellStateSnapshot {
            energy: BioEnergy::new(3.0),
            ..CellStateSnapshot::ZEROS
        });

        assert_eq!(
            requests,
            vec![CellLayer::resize_request(0, AreaDelta::new(32.0))]
        );
    }
}
