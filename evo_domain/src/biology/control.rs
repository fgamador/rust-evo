use crate::biology::cell::Cell;
use crate::biology::control_requests::*;
use crate::biology::genome::*;
use crate::biology::layers::*;
use crate::physics::nodes_with_handles::*;
use crate::physics::quantities::*;
use smallvec::alloc::fmt::Formatter;
use std::fmt;
use std::sync::Arc;

pub trait CellControl: fmt::Debug + Send + Sync {
    fn run(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest>;

    fn spawn(&mut self) -> Box<dyn CellControl>;

    fn print(&self, _cell_handle: NodeHandle<Cell>) {}
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

type GetValueFns = Vec<(
    VecIndex,
    Box<dyn Fn(&CellStateSnapshot) -> Value1D + Send + Sync>,
)>;
type ValueToRequestFns = Vec<(
    VecIndex,
    Box<dyn Fn(Value1D) -> ControlRequest + Send + Sync>,
)>;

pub struct NeuralNetControl {
    get_value_fns: Arc<GetValueFns>,
    nnet: SparseNeuralNet,
    value_to_request_fns: Arc<ValueToRequestFns>,
    randomness: SeededMutationRandomness,
    node_labels: Arc<Vec<&'static str>>,
}

impl NeuralNetControl {
    fn new(
        get_value_fns: GetValueFns,
        genome: SparseNeuralNetGenome,
        value_to_request_fns: ValueToRequestFns,
        randomness: SeededMutationRandomness,
        node_labels: Vec<&'static str>,
    ) -> Self {
        NeuralNetControl {
            get_value_fns: Arc::new(get_value_fns),
            nnet: SparseNeuralNet::new(genome),
            value_to_request_fns: Arc::new(value_to_request_fns),
            randomness,
            node_labels: Arc::new(node_labels),
        }
    }

    fn set_input_values(&mut self, cell_state: &CellStateSnapshot) {
        for (node_index, get_value_fn) in &*self.get_value_fns {
            self.nnet
                .set_node_value(*node_index, get_value_fn(cell_state) as NodeValue);
        }
    }

    fn get_output_requests(&self) -> Vec<ControlRequest> {
        let mut requests = Vec::with_capacity(self.value_to_request_fns.len());
        for (node_index, value_to_request_fn) in &*self.value_to_request_fns {
            requests.push(value_to_request_fn(
                self.nnet.node_value(*node_index) as Value1D
            ));
        }
        requests
    }
}

impl fmt::Debug for NeuralNetControl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("NeuralNetControl")
            // TODO get_value_fns?
            .field("nnet", &self.nnet)
            // TODO value_to_request_fns?
            .field("randomness", &self.randomness)
            .finish()
    }
}

impl CellControl for NeuralNetControl {
    fn run(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        self.set_input_values(cell_state);
        self.nnet.run();
        self.get_output_requests()
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(NeuralNetControl {
            get_value_fns: Arc::clone(&self.get_value_fns),
            nnet: self.nnet.spawn(&mut self.randomness),
            value_to_request_fns: Arc::clone(&self.value_to_request_fns),
            randomness: self.randomness.clone(),
            node_labels: Arc::clone(&self.node_labels),
        })
    }

    fn print(&self, cell_handle: NodeHandle<Cell>) {
        println!("Cell {} genome:", cell_handle);
        self.nnet.print(&self.node_labels);
    }
}

pub struct NeuralNetControlBuilder {
    get_value_fns: GetValueFns,
    genome: SparseNeuralNetGenome,
    value_to_request_fns: ValueToRequestFns,
    next_index: VecIndex,
    node_labels: Vec<&'static str>,
}

impl NeuralNetControlBuilder {
    pub fn new(transfer_fn: TransferFn) -> Self {
        NeuralNetControlBuilder {
            get_value_fns: vec![],
            genome: SparseNeuralNetGenome::new(transfer_fn),
            value_to_request_fns: vec![],
            next_index: 0,
            node_labels: vec![],
        }
    }

    pub fn add_input_node<F>(&mut self, node_label: &'static str, get_value: F) -> VecIndex
    where
        F: 'static + Fn(&CellStateSnapshot) -> Value1D + Send + Sync,
    {
        let node_index = self.next_node_index();
        self.get_value_fns.push((node_index, Box::new(get_value)));
        self.add_node_label(node_index, node_label);
        node_index
    }

    pub fn add_hidden_node(
        &mut self,
        node_label: &'static str,
        from_value_weights: &[(VecIndex, Coefficient)],
        bias: Coefficient,
    ) -> VecIndex {
        let node_index = self.next_node_index();
        self.genome
            .connect_node(node_index, bias, from_value_weights);
        self.add_node_label(node_index, node_label);
        node_index
    }

    pub fn add_output_node<F>(
        &mut self,
        node_label: &'static str,
        from_value_weights: &[(VecIndex, Coefficient)],
        bias: Coefficient,
        value_to_request: F,
    ) -> VecIndex
    where
        F: 'static + Fn(Value1D) -> ControlRequest + Send + Sync,
    {
        let node_index = self.next_node_index();
        self.genome
            .connect_node(node_index, bias, from_value_weights);
        self.value_to_request_fns
            .push((node_index, Box::new(value_to_request)));
        self.add_node_label(node_index, node_label);
        node_index
    }

    fn next_node_index(&mut self) -> VecIndex {
        let node_index = self.next_index;
        self.next_index += 1;
        node_index
    }

    fn add_node_label(&mut self, node_index: VecIndex, node_label: &'static str) {
        if self.node_labels.len() <= (node_index as usize) {
            self.node_labels.resize((node_index + 1) as usize, "");
        }
        self.node_labels[node_index as usize] = node_label;
    }

    pub fn build(self, randomness: SeededMutationRandomness) -> NeuralNetControl {
        NeuralNetControl::new(
            self.get_value_fns,
            self.genome,
            self.value_to_request_fns,
            randomness,
            self.node_labels,
        )
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
    fn can_build_neural_net_control() {
        let mut builder = NeuralNetControlBuilder::new(TransferFn::IDENTITY);

        let energy_input_index =
            builder.add_input_node("energy", |cell_state| cell_state.energy.value());
        let adjusted_energy_index =
            builder.add_hidden_node("adj energy", &[(energy_input_index, -1.0)], -2.0);
        builder.add_output_node("resize", &[(adjusted_energy_index, 10.0)], 2.0, |value| {
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
            vec![CellLayer::resize_request(0, AreaDelta::new(-48.0))]
        );
    }
}
