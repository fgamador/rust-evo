// Inspired by NEAT: "Evolving Neural Networks through Augmenting Topologies"
// by Kenneth O. Stanley and Risto Miikkulainen
// http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf

use std::f32;

type OpFn = fn(f32, f32, &mut f32);

pub struct Op {
    from_value_index: u16,
    to_value_index: u16,
    op_fn: OpFn,
    weight: f32,
}

pub struct SparseNeuralNet {
    num_inputs: u16,
    num_outputs: u16,
    node_values: Vec<f32>,
    ops: Vec<Op>,
}

impl SparseNeuralNet {
    pub fn fully_connected(
        num_inputs: u16,
        num_outputs: u16,
        initial_weight: f32,
        transfer_fn: OpFn,
    ) -> Self {
        let mut nnet = SparseNeuralNet {
            num_inputs,
            num_outputs,
            node_values: vec![0.0; (1 + num_inputs + num_outputs) as usize],
            ops: vec![],
        };
        nnet.node_values[0] = 1.0; // bias node
        nnet.fully_connect_inputs_and_outputs(initial_weight, transfer_fn);
        nnet
    }

    fn fully_connect_inputs_and_outputs(&mut self, initial_weight: f32, transfer_fn: OpFn) {
        self.ops
            .reserve(((1 + self.num_inputs) * self.num_outputs) as usize);
        for output_value_index in (1 + self.num_inputs)..=(self.num_inputs + self.num_outputs) {
            for input_value_index in 0..=self.num_inputs {
                self.ops.push(Op {
                    from_value_index: input_value_index,
                    to_value_index: output_value_index,
                    op_fn: Self::add_weighted,
                    weight: initial_weight,
                });
            }
            self.ops.push(Op {
                from_value_index: 0,
                to_value_index: output_value_index,
                op_fn: transfer_fn,
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

    pub fn set_input(&mut self, index: usize, val: f32) {
        assert!(index < self.num_inputs as usize);
        self.node_values[1 + index] = val;
    }

    pub fn run(&mut self) {
        self.clear_computed_values();
        for op in &self.ops {
            let from_value = self.node_values[op.from_value_index as usize];
            let to_value = &mut self.node_values[op.to_value_index as usize];
            (op.op_fn)(from_value, op.weight, to_value);
        }
    }

    pub fn output(&self, index: usize) -> f32 {
        assert!(index < self.num_outputs as usize);
        self.node_values[1 + self.num_inputs as usize + index]
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
