// Inspired by NEAT: "Evolving Neural Networks through Augmenting Topologies"
// by Kenneth O. Stanley and Risto Miikkulainen
// http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf

use std::f32;
use std::fmt;
use std::fmt::{Error, Formatter};

// TODO
//type NodeIndex = u16;
//type NodeValue = f32;
//type ConnectionWeight = f32;
type OpFn = fn(f32, f32, &mut f32);

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
    from_value_index: u16,
    to_value_index: u16,
    weight: f32,
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
    fn bias_op(to_value_index: u16, bias: f32) -> Self {
        Op {
            op_fn: SparseNeuralNet::add_weighted,
            from_value_index: 0,
            to_value_index,
            weight: bias,
        }
    }

    fn connection_op(from_value_index: u16, to_value_index: u16, weight: f32) -> Self {
        Op {
            op_fn: SparseNeuralNet::add_weighted,
            from_value_index,
            to_value_index,
            weight,
        }
    }

    fn transfer_function_op(transfer_fn: OpFn, to_value_index: u16) -> Self {
        Op {
            op_fn: transfer_fn,
            from_value_index: 0, // dummy
            to_value_index,
            weight: 0.0, // dummy
        }
    }

    fn run(&self, node_values: &mut Vec<f32>) {
        let from_value = node_values[self.from_value_index as usize];
        let to_value = &mut node_values[self.to_value_index as usize];
        (self.op_fn)(from_value, self.weight, to_value);
    }
}

pub struct SparseNeuralNet {
    num_inputs: u16,
    num_outputs: u16,
    transfer_fn: OpFn,
    node_values: Vec<f32>,
    ops: Vec<Op>,
}

impl fmt::Debug for SparseNeuralNet {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "SparseNeuralNet {{ num_inputs: {}, num_outputs: {}, transfer_fn: OpFn, node_values: {:?}, ops: {:?} }}",
            self.num_inputs, self.num_outputs, self.node_values, self.ops
        )
    }
}

impl SparseNeuralNet {
    pub fn unconnected(num_inputs: u16, num_outputs: u16, transfer_fn: OpFn) -> Self {
        let mut nnet = SparseNeuralNet {
            num_inputs,
            num_outputs,
            transfer_fn,
            node_values: vec![0.0; (1 + num_inputs + num_outputs) as usize],
            ops: vec![],
        };
        nnet.node_values[0] = 1.0; // bias node
        nnet
    }

    pub fn connect_output_node(
        &mut self,
        output_value_index: u16,
        bias: f32,
        input_value_weights: Vec<(u16, f32)>,
    ) {
        let to_value_index = self.output_index_to_node_value_index(output_value_index);
        self.ops.push(Op::bias_op(to_value_index, bias));
        for (input_value_index, weight) in input_value_weights {
            let from_value_index = self.input_index_to_node_value_index(input_value_index);
            self.ops
                .push(Op::connection_op(from_value_index, to_value_index, weight));
        }
        self.ops
            .push(transfer_function_op(self.transfer_fn, to_value_index));
    }

    pub fn fully_connected(
        num_inputs: u16,
        num_outputs: u16,
        initial_weight: f32,
        transfer_fn: OpFn,
    ) -> Self {
        let mut nnet = Self::unconnected(num_inputs, num_outputs, transfer_fn);
        nnet.fully_connect_inputs_and_outputs(initial_weight);
        nnet
    }

    fn fully_connect_inputs_and_outputs(&mut self, initial_weight: f32) {
        self.ops
            .reserve(((1 + self.num_inputs) * self.num_outputs) as usize);
        for output_value_index in (1 + self.num_inputs)..=(self.num_inputs + self.num_outputs) {
            for input_value_index in 0..=self.num_inputs {
                self.ops.push(Op {
                    op_fn: Self::add_weighted,
                    from_value_index: input_value_index,
                    to_value_index: output_value_index,
                    weight: initial_weight,
                });
            }
            self.ops.push(Op {
                op_fn: self.transfer_fn,
                from_value_index: 0,
                to_value_index: output_value_index,
                weight: 0.0,
            });
        }
    }

    pub fn set_weight(&mut self, from_index: usize, to_index: usize, weight: f32) {
        // TODO need more efficient way
        for op in &mut self.ops {
            if op.from_value_index as usize == from_index && op.to_value_index as usize == to_index
            {
                op.weight = weight;
            }
        }
    }

    pub fn set_input(&mut self, index: u16, val: f32) {
        let node_value_index = self.input_index_to_node_value_index(index) as usize;
        self.node_values[node_value_index] = val;
    }

    fn input_index_to_node_value_index(&self, index: u16) -> u16 {
        assert!(index < self.num_inputs);
        1 + index
    }

    pub fn run(&mut self) {
        self.clear_computed_values();
        for op in &self.ops {
            op.run(&mut self.node_values);
        }
    }

    pub fn output(&self, index: u16) -> f32 {
        let node_value_index = self.output_index_to_node_value_index(index) as usize;
        self.node_values[node_value_index]
    }

    fn output_index_to_node_value_index(&self, index: u16) -> u16 {
        assert!(index < self.num_outputs);
        1 + self.num_inputs + index
    }

    pub fn clear_computed_values(&mut self) {
        let original_len = self.node_values.len();
        self.node_values.truncate(1 + self.num_inputs as usize);
        self.node_values.resize(original_len, 0.0);
    }

    pub fn add_weighted(from_value: f32, weight: f32, to_value: &mut f32) {
        *to_value += weight * from_value;
    }

    pub fn identity(_from_value: f32, _weight: f32, _to_value: &mut f32) {}

    pub fn sigmoidal(_from_value: f32, _weight: f32, to_value: &mut f32) {
        *to_value = Self::sigmoidal_fn(*to_value);
    }

    fn sigmoidal_fn(val: f32) -> f32 {
        1.0_f32 / (1.0_f32 + (-4.9_f32 * val).exp())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_layer_fully_connected_no_bias() {
        let mut nnet = SparseNeuralNet::fully_connected(3, 2, 0.5, plus_one);
        nnet.set_weight(0, 4, 0.0);
        nnet.set_weight(0, 5, 0.0);

        nnet.set_input(0, 2.0);
        nnet.set_input(1, 3.0);
        nnet.set_input(2, 4.0);
        nnet.run();

        assert_eq!(nnet.output(0), 5.5);
        assert_eq!(nnet.output(1), 5.5);
    }

    #[test]
    fn two_layer_sparsely_connected() {
        let mut nnet = SparseNeuralNet::unconnected(2, 2, plus_one);
        nnet.connect_output_node(0, 0.5, vec![(0, 0.5)]);
        nnet.connect_output_node(1, 0.0, vec![(0, 0.75), (1, 0.25)]);

        nnet.set_input(0, 2.0);
        nnet.set_input(1, 4.0);
        nnet.run();

        assert_eq!(nnet.output(0), 2.5);
        assert_eq!(nnet.output(1), 3.5);
    }

    #[test]
    fn run_clears_previous_values() {
        let mut nnet = SparseNeuralNet::fully_connected(1, 1, 1.0, SparseNeuralNet::identity);
        nnet.set_weight(0, 2, 0.0);

        nnet.set_input(0, 1.0);
        nnet.run();
        nnet.set_input(0, 3.0);
        nnet.run();

        assert_eq!(nnet.output(0), 3.0);
    }

    #[test]
    fn bias_node() {
        let mut nnet = SparseNeuralNet::fully_connected(1, 1, 1.0, SparseNeuralNet::identity);

        nnet.set_input(0, 3.0);
        nnet.run();

        assert_eq!(nnet.output(0), 4.0);
    }

    fn plus_one(_from_value: f32, _weight: f32, to_value: &mut f32) {
        *to_value += 1.0;
    }
}
