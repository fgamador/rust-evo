// Inspired by NEAT: "Evolving Neural Networks through Augmenting Topologies"
// by Kenneth O. Stanley and Risto Miikkulainen
// http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf

use std::f32;

pub struct NeuralNet {
    num_inputs: u16,
    num_outputs: u16,
    node_values: Vec<f32>,
    initial_weight: f32,
    activation_fn: fn(f32) -> f32,
}

impl NeuralNet {
    pub fn new(num_inputs: u16, num_outputs: u16, initial_weight: f32, activation_fn: fn(f32) -> f32) -> Self {
        let mut nnet = NeuralNet {
            num_inputs,
            num_outputs,
            node_values: vec![],
            initial_weight,
            activation_fn,
        };
        nnet.node_values.resize((num_inputs + num_outputs) as usize, 0.0);
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

//    #[test]
//    fn initial_fully_connected() {
//        let mut nnet = NeuralNet::new(3, 2, 0.5);
//        nnet.set_input(0, 2.0);
//        nnet.set_input(1, 3.0);
//        nnet.set_input(2, 4.0);
//        nnet.run();
//        assert_eq!(nnet.output(0), 2.0);
//        assert_eq!(nnet.output(1), 2.0);
//    }
}
