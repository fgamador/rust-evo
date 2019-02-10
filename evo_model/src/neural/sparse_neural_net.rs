// Inspired by NEAT: "Evolving Neural Networks through Augmenting Topologies"
// by Kenneth O. Stanley and Risto Miikkulainen
// http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf

use std::f32;

pub struct Op {
    input_index: u16,
    output_index: u16,
    weight: f32,
    op_fn: fn(&Op, &mut Vec<f32>),
}

pub struct SparseNeuralNet {
    num_inputs: u16,
    num_outputs: u16,
    node_values: Vec<f32>,
    ops: Vec<Op>,
}

impl SparseNeuralNet {
    pub fn new(num_inputs: u16, num_outputs: u16, initial_weight: f32, transfer_op: fn(&Op, &mut Vec<f32>)) -> Self {
        let mut nnet = SparseNeuralNet {
            num_inputs,
            num_outputs,
            node_values: vec![],
            ops: vec![],
        };
        nnet.init_node_values(num_inputs, num_outputs);
        nnet.init_ops(num_inputs, num_outputs, initial_weight, transfer_op);
        nnet
    }

    fn init_node_values(&mut self, num_inputs: u16, num_outputs: u16) {
        self.node_values.resize((num_inputs + num_outputs) as usize, 0.0);
    }

    fn init_ops(&mut self, num_inputs: u16, num_outputs: u16, initial_weight: f32, transfer_op: fn(&Op, &mut Vec<f32>)) {
        self.ops.reserve((num_inputs * num_outputs + num_outputs) as usize);
        for output_index in num_inputs..num_inputs + num_outputs {
            for input_index in 0..num_inputs {
                self.ops.push(Op { input_index, output_index, weight: initial_weight, op_fn: Self::add_weighted });
            }
            self.ops.push(Op { input_index: 0, output_index, weight: 0.0, op_fn: transfer_op });
        }
    }

    // TODO test
    pub fn clear_node_values(&mut self) {
        let len = self.node_values.len();
        self.node_values.clear();
        self.node_values.resize(len, 0.0);

//        extern crate libc;
//        use std::mem;
//
//        fn main() {
//            let mut buffer: Vec<u32> = vec![42; 20];
//            println!("{:?}", buffer);
//
//            // overwrite the buffer with all zeros
//            unsafe {
//                libc::memset(
//                    buffer.as_mut_ptr() as _,
//                    0,
//                    buffer.len() * mem::size_of::<u32>(),
//                );
//            }
//            println!("{:?}", buffer);
//        }
    }

    pub fn set_input(&mut self, index: usize, val: f32) {
        assert!(index < self.num_inputs as usize);
        self.node_values[index] = val;
    }

    pub fn run(&mut self) {
        for op in &self.ops {
            (op.op_fn)(op, &mut self.node_values);
        }
    }

    pub fn output(&self, index: usize) -> f32 {
        assert!(index < self.num_outputs as usize);
        self.node_values[self.num_inputs as usize + index]
    }

    pub fn sigmoidal(op: &Op, node_values: &mut Vec<f32>) {
        node_values[op.output_index as usize] = Self::sigmoidal_fn(node_values[op.output_index as usize]);
    }

    fn sigmoidal_fn(val: f32) -> f32 {
        1.0_f32 / (1.0_f32 + (-4.9_f32 * val).exp())
    }

    fn add_weighted(op: &Op, node_values: &mut Vec<f32>) {
        node_values[op.output_index as usize] += op.weight * node_values[op.input_index as usize];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_fully_connected() {
        let mut nnet = SparseNeuralNet::new(3, 2, 0.5, plus_one);
        nnet.set_input(0, 2.0);
        nnet.set_input(1, 3.0);
        nnet.set_input(2, 4.0);
        nnet.run();
        assert_eq!(nnet.output(0), 5.5);
        assert_eq!(nnet.output(1), 5.5);
    }

    // TODO clear between runs

    pub fn plus_one(op: &Op, node_values: &mut Vec<f32>) {
        node_values[op.output_index as usize] += 1.0;
    }
}
