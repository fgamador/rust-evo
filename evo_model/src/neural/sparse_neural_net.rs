// Inspired by NEAT: "Evolving Neural Networks through Augmenting Topologies"
// by Kenneth O. Stanley and Risto Miikkulainen
// http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf

use std::f32;
use std::fmt;
use std::fmt::{Error, Formatter};

type Bias = f32;
type ConnectionWeight = f32;
type NodeIndex = u16;
type NodeValue = f32;
type TransferFn = fn(&mut NodeValue);

pub enum Op {
    Bias {
        value_index: NodeIndex,
        bias: Bias,
    },
    Connection {
        from_value_index: NodeIndex,
        to_value_index: NodeIndex,
        weight: ConnectionWeight,
    },
    Transfer {
        value_index: NodeIndex,
        transfer_fn: TransferFn,
    },
}

// TODO
impl fmt::Debug for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Op")
    }
}

impl Op {
    fn run(&self, node_values: &mut Vec<NodeValue>) {
        match self {
            Self::Bias { value_index, bias } => {
                let value = &mut node_values[*value_index as usize];
                *value = *bias;
            }

            Self::Connection {
                from_value_index,
                to_value_index,
                weight,
            } => {
                let from_value = node_values[*from_value_index as usize];
                let to_value = &mut node_values[*to_value_index as usize];
                *to_value += *weight * from_value;
            }

            Self::Transfer {
                value_index,
                transfer_fn,
            } => {
                let value = &mut node_values[*value_index as usize];
                (transfer_fn)(value);
            }
        }
    }

    pub fn identity(_value: &mut NodeValue) {}

    pub fn sigmoidal(value: &mut NodeValue) {
        *value = Self::sigmoidal_fn(*value);
    }

    fn sigmoidal_fn(val: NodeValue) -> NodeValue {
        1.0_f32 / (1.0_f32 + (-4.9_f32 * val).exp())
    }
}

pub struct SparseNeuralNet {
    node_values: Vec<NodeValue>,
    ops: Vec<Op>,
    transfer_fn: TransferFn,
}

impl fmt::Debug for SparseNeuralNet {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "SparseNeuralNet {{ node_values: {:?}, ops: {:?}, transfer_fn: TransferFn }}",
            self.node_values, self.ops
        )
    }
}

impl SparseNeuralNet {
    pub fn new(transfer_fn: TransferFn) -> Self {
        SparseNeuralNet {
            node_values: vec![],
            ops: vec![],
            transfer_fn,
        }
    }

    pub fn connect_node(
        &mut self,
        to_value_index: NodeIndex,
        bias: ConnectionWeight,
        from_value_weights: Vec<(NodeIndex, ConnectionWeight)>,
    ) {
        self.grow_node_values_if_needed(to_value_index);
        self.ops.push(Op::Bias {
            value_index: to_value_index,
            bias,
        });
        for (from_value_index, weight) in from_value_weights {
            self.grow_node_values_if_needed(from_value_index);
            self.ops.push(Op::Connection {
                from_value_index,
                to_value_index,
                weight,
            });
        }
        self.ops.push(Op::Transfer {
            value_index: to_value_index,
            transfer_fn: self.transfer_fn,
        });
    }

    fn grow_node_values_if_needed(&mut self, new_index: NodeIndex) {
        if new_index as usize >= self.node_values.len() {
            self.node_values.resize((new_index + 1) as usize, 0.0);
        }
    }

    pub fn set_node_value(&mut self, index: NodeIndex, value: NodeValue) {
        self.node_values[index as usize] = value;
    }

    pub fn node_value(&self, index: NodeIndex) -> NodeValue {
        self.node_values[index as usize]
    }

    pub fn run(&mut self) {
        for op in &self.ops {
            op.run(&mut self.node_values);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_layer_sparsely_connected() {
        let mut nnet = SparseNeuralNet::new(plus_one);
        nnet.connect_node(2, 0.5, vec![(0, 0.5)]);
        nnet.connect_node(3, 0.0, vec![(0, 0.75), (1, 0.25)]);

        nnet.set_node_value(0, 2.0);
        nnet.set_node_value(1, 4.0);
        nnet.run();

        assert_eq!(nnet.node_value(2), 2.5);
        assert_eq!(nnet.node_value(3), 3.5);
    }

    #[test]
    fn run_clears_previous_values() {
        let mut nnet = SparseNeuralNet::new(Op::identity);
        nnet.connect_node(1, 0.0, vec![(0, 1.0)]);

        nnet.set_node_value(0, 1.0);
        nnet.run();
        nnet.set_node_value(0, 3.0);
        nnet.run();

        assert_eq!(nnet.node_value(1), 3.0);
    }

    #[test]
    fn three_layer() {
        let mut nnet = SparseNeuralNet::new(Op::identity);
        nnet.connect_node(1, 0.5, vec![(0, 0.5)]);
        nnet.connect_node(2, 0.0, vec![(1, 0.5)]);

        nnet.set_node_value(0, 2.0);
        nnet.run();

        assert_eq!(nnet.node_value(2), 0.75);
    }

    fn plus_one(to_value: &mut NodeValue) {
        *to_value += 1.0;
    }
}
