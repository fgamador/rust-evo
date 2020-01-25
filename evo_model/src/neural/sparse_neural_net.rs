// Inspired by NEAT: "Evolving Neural Networks through Augmenting Topologies"
// by Kenneth O. Stanley and Risto Miikkulainen
// http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf

use std::f32;

pub struct Link {
    from_value_index: u16,
    to_value_index: u16,
    weight: f32,
    op_fn: fn(&Link, &mut Vec<f32>),
}

pub struct SparseNeuralNet {
    num_inputs: u16,
    num_outputs: u16,
    node_values: Vec<f32>,
    links: Vec<Link>,
}

impl SparseNeuralNet {
    pub fn fully_connected(
        num_inputs: u16,
        num_outputs: u16,
        initial_weight: f32,
        transfer_fn: fn(&Link, &mut Vec<f32>),
    ) -> Self {
        let mut nnet = SparseNeuralNet {
            num_inputs,
            num_outputs,
            node_values: vec![0.0; (1 + num_inputs + num_outputs) as usize],
            links: vec![],
        };
        nnet.node_values[0] = 1.0; // bias node
        nnet.fully_connect_inputs_and_outputs(initial_weight, transfer_fn);
        nnet
    }

    fn fully_connect_inputs_and_outputs(
        &mut self,
        initial_weight: f32,
        transfer_fn: fn(&Link, &mut Vec<f32>),
    ) {
        self.links
            .reserve(((1 + self.num_inputs) * self.num_outputs) as usize);
        for output_value_index in (1 + self.num_inputs)..=(self.num_inputs + self.num_outputs) {
            for input_value_index in 0..=self.num_inputs {
                self.links.push(Link {
                    from_value_index: input_value_index,
                    to_value_index: output_value_index,
                    weight: initial_weight,
                    op_fn: Self::add_weighted,
                });
            }
            self.links.push(Link {
                from_value_index: 0,
                to_value_index: output_value_index,
                weight: 0.0,
                op_fn: transfer_fn,
            });
        }
    }

    pub fn set_weight(&mut self, from_index: usize, to_index: usize, weight: f32) {
        // TODO need more efficient way
        for op in &mut self.links {
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
        for op in &self.links {
            (op.op_fn)(op, &mut self.node_values);
        }
    }

    pub fn output(&self, index: usize) -> f32 {
        assert!(index < self.num_outputs as usize);
        self.node_values[1 + self.num_inputs as usize + index]
    }

    pub fn clear_computed_values(&mut self) {
        let len = self.node_values.len();
        self.node_values.truncate(1 + self.num_inputs as usize);
        self.node_values.resize(len, 0.0);
    }

    fn add_weighted(op: &Link, node_values: &mut Vec<f32>) {
        node_values[op.to_value_index as usize] +=
            op.weight * node_values[op.from_value_index as usize];
    }

    pub fn identity(_op: &Link, _node_values: &mut Vec<f32>) {}

    pub fn sigmoidal(op: &Link, node_values: &mut Vec<f32>) {
        node_values[op.to_value_index as usize] =
            Self::sigmoidal_fn(node_values[op.to_value_index as usize]);
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

    fn plus_one(op: &Link, node_values: &mut Vec<f32>) {
        node_values[op.to_value_index as usize] += 1.0;
    }
}
