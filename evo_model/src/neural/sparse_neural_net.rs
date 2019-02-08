// Inspired by NEAT: "Evolving Neural Networks through Augmenting Topologies"
// by Kenneth O. Stanley and Risto Miikkulainen
// http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf

use std::f32;

struct Op {
    input_index: u16,
    output_index: u16,
    weight: f32,
    op_fn: fn(op: &Op, node_values: &mut Vec<f32>),
}

impl Op {
    fn add_weighted(op: &Op, node_values: &mut Vec<f32>) {
        node_values[op.output_index as usize] += op.weight * node_values[op.input_index as usize];
    }
}

pub struct NeuralNet {
    num_inputs: u16,
    num_outputs: u16,
    node_values: Vec<f32>,
    ops: Vec<Op>,
    initial_weight: f32,
    activation_fn: fn(f32) -> f32,
}

impl NeuralNet {
    pub fn new(num_inputs: u16, num_outputs: u16, initial_weight: f32, activation_fn: fn(f32) -> f32) -> Self {
        let mut nnet = NeuralNet {
            num_inputs,
            num_outputs,
            node_values: vec![],
            ops: Vec::with_capacity((num_inputs * num_outputs + num_outputs) as usize),
            initial_weight,
            activation_fn,
        };
        nnet.node_values.resize((num_inputs + num_outputs) as usize, 0.0);
        for output_index in 0..num_outputs {
            for input_index in 0..num_inputs {
                nnet.ops.push(Op { input_index, output_index, weight: initial_weight, op_fn: Op::add_weighted });
            }
        }
        for output_index in 0..num_outputs {
            nnet.ops.push(Op { input_index: 0, output_index, weight: 0.0, op_fn: Op::add_weighted });
        }
        nnet
    }

    pub fn set_input(&mut self, index: usize, val: f32) {
        assert!(index < self.num_inputs as usize);
        self.node_values[index] = val;
    }

    pub fn run(&mut self) {
        // TODO initial hack
        self.node_values[1] = (self.activation_fn)(self.initial_weight * self.node_values[0]);
    }

    pub fn output(&self, index: usize) -> f32 {
        assert!(index < self.num_outputs as usize);
        self.node_values[self.num_inputs as usize + index]
    }

    pub fn identity(val: f32) -> f32 {
        val
    }

    pub fn plus_one(val: f32) -> f32 {
        val + 1.0
    }

    pub fn sigmoidal(val: f32) -> f32 {
        1.0_f32 / (1.0_f32 + (-4.9_f32 * val).exp())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplest_net() {
        let mut nnet = NeuralNet::new(1, 1, 0.5, NeuralNet::plus_one);
        nnet.set_input(0, 3.0);
        nnet.run();
        assert_eq!(nnet.output(0), 2.5);
    }

    //#[test]
    fn initial_fully_connected() {
        let mut nnet = NeuralNet::new(3, 2, 0.5, NeuralNet::plus_one);
        nnet.set_input(0, 2.0);
        nnet.set_input(1, 3.0);
        nnet.set_input(2, 4.0);
        nnet.run();
        assert_eq!(nnet.output(0), 5.5);
        assert_eq!(nnet.output(1), 5.5);
    }

    // TODO clear between runs
}
