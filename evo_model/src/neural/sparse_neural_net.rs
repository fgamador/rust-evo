// Inspired by NEAT: "Evolving Neural Networks through Augmenting Topologies"
// by Kenneth O. Stanley and Risto Miikkulainen
// http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf

use std::f32;
use std::fmt;
use std::fmt::{Error, Formatter};

type ConnectionWeight = f32;
type NodeIndex = u16;
type NodeValue = f32;
type OpFn = fn(NodeValue, ConnectionWeight, &mut NodeValue);

//impl fmt::Debug for OpFn {
//    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
//        write!(
//            f,
//            "OpFn"
//        )
//    }
//}

pub struct Op {
    op_fn: OpFn,
    from_value_index: NodeIndex,
    to_value_index: NodeIndex,
    weight: ConnectionWeight,
}

impl fmt::Debug for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "OpStruct {{ op_fn: OpFn, from_value_index: {}, to_value_index: {}, weight: {} }}",
            self.from_value_index, self.to_value_index, self.weight
        )
    }
}

impl Op {
    fn bias_op(to_value_index: NodeIndex, bias: ConnectionWeight) -> Self {
        Op {
            op_fn: Self::set_to_weight,
            from_value_index: 0, // dummy
            to_value_index,
            weight: bias,
        }
    }

    fn connection_op(
        from_value_index: NodeIndex,
        to_value_index: NodeIndex,
        weight: ConnectionWeight,
    ) -> Self {
        Op {
            op_fn: Self::add_weighted,
            from_value_index,
            to_value_index,
            weight,
        }
    }

    fn transfer_function_op(transfer_fn: OpFn, to_value_index: NodeIndex) -> Self {
        Op {
            op_fn: transfer_fn,
            from_value_index: 0, // dummy
            to_value_index,
            weight: 0.0, // dummy
        }
    }

    fn run(&self, node_values: &mut Vec<NodeValue>) {
        let from_value = node_values[self.from_value_index as usize];
        let to_value = &mut node_values[self.to_value_index as usize];
        (self.op_fn)(from_value, self.weight, to_value);
    }

    pub fn set_to_weight(
        _from_value: NodeValue,
        weight: ConnectionWeight,
        to_value: &mut NodeValue,
    ) {
        *to_value = weight;
    }

    pub fn add_weighted(from_value: NodeValue, weight: ConnectionWeight, to_value: &mut NodeValue) {
        *to_value += weight * from_value;
    }

    pub fn identity(_from_value: NodeValue, _weight: ConnectionWeight, _to_value: &mut NodeValue) {}

    pub fn sigmoidal(_from_value: NodeValue, _weight: ConnectionWeight, to_value: &mut NodeValue) {
        *to_value = Self::sigmoidal_fn(*to_value);
    }

    fn sigmoidal_fn(val: NodeValue) -> NodeValue {
        1.0_f32 / (1.0_f32 + (-4.9_f32 * val).exp())
    }
}

pub struct SparseNeuralNet {
    node_values: Vec<NodeValue>,
    ops: Vec<Op>,
    transfer_fn: OpFn,
}

impl fmt::Debug for SparseNeuralNet {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "SparseNeuralNet {{ node_values: {:?}, ops: {:?}, transfer_fn: OpFn }}",
            self.node_values, self.ops
        )
    }
}

impl SparseNeuralNet {
    pub fn new(transfer_fn: OpFn) -> Self {
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
        self.ops.push(Op::bias_op(to_value_index, bias));
        for (from_value_index, weight) in from_value_weights {
            self.grow_node_values_if_needed(from_value_index);
            self.ops
                .push(Op::connection_op(from_value_index, to_value_index, weight));
        }
        self.ops
            .push(Op::transfer_function_op(self.transfer_fn, to_value_index));
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
    fn two_layer_fully_connected_no_bias() {
        let mut nnet = SparseNeuralNet::new(plus_one);
        nnet.connect_node(3, 0.0, vec![(0, 0.5), (1, 0.5), (2, 0.5)]);
        nnet.connect_node(4, 0.0, vec![(0, 0.5), (1, 0.5), (2, 0.5)]);

        nnet.set_node_value(0, 2.0);
        nnet.set_node_value(1, 3.0);
        nnet.set_node_value(2, 4.0);
        nnet.run();

        assert_eq!(nnet.node_value(3), 5.5);
        assert_eq!(nnet.node_value(4), 5.5);
    }

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
    fn bias_node() {
        let mut nnet = SparseNeuralNet::new(Op::identity);
        nnet.connect_node(1, 1.0, vec![(0, 1.0)]);

        nnet.set_node_value(0, 3.0);
        nnet.run();

        assert_eq!(nnet.node_value(1), 4.0);
    }

    fn plus_one(_from_value: NodeValue, _weight: ConnectionWeight, to_value: &mut NodeValue) {
        *to_value += 1.0;
    }
}
