// Inspired by NEAT: "Evolving Neural Networks through Augmenting Topologies"
// by Kenneth O. Stanley and Risto Miikkulainen
// http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf

use rand::distributions::Distribution;
use rand::{Rng, SeedableRng};
use rand_distr::Normal;
use rand_pcg::Pcg64Mcg;
use std::collections::HashMap;
use std::f32;
use std::fmt;
use std::fmt::{Error, Formatter};

pub type Coefficient = f32;
pub type VecIndex = u16;
pub type NodeValue = f32;

#[derive(Clone, Debug, PartialEq)]
pub struct SparseNeuralNet {
    genome: SparseNeuralNetGenome,
    node_values: Vec<NodeValue>,
}

impl SparseNeuralNet {
    pub fn new(genome: SparseNeuralNetGenome) -> Self {
        let num_nodes = genome.num_nodes;
        SparseNeuralNet {
            genome,
            node_values: vec![0.0; num_nodes as usize],
        }
    }

    pub fn spawn(&self, randomness: &mut dyn MutationRandomness) -> Self {
        Self::new(self.genome.spawn(randomness))
    }

    pub fn set_node_value(&mut self, index: VecIndex, value: NodeValue) {
        self.node_values[index as usize] = value;
    }

    pub fn node_value(&self, index: VecIndex) -> NodeValue {
        self.node_values[index as usize]
    }

    pub fn run(&mut self) {
        self.genome.run(&mut self.node_values);
    }

    pub fn print(&self, node_labels: &[&str]) {
        self.genome.print(node_labels);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SparseNeuralNetGenome {
    ops: Vec<Op>,
    transfer_fn: TransferFn,
    num_nodes: VecIndex,
}

impl SparseNeuralNetGenome {
    pub fn new(transfer_fn: TransferFn) -> Self {
        SparseNeuralNetGenome {
            ops: vec![],
            transfer_fn,
            num_nodes: 0,
        }
    }

    pub fn connect_node(
        &mut self,
        to_value_index: VecIndex,
        bias: Coefficient,
        from_value_weights: &[(VecIndex, Coefficient)],
    ) {
        self.grow_num_nodes_if_needed(to_value_index);
        self.ops.push(Op::Bias {
            value_index: to_value_index,
            bias,
        });
        for (from_value_index, weight) in from_value_weights {
            self.grow_num_nodes_if_needed(*from_value_index);
            self.ops.push(Op::Connection {
                from_value_index: *from_value_index,
                to_value_index,
                weight: *weight,
            });
        }
        self.ops.push(Op::Transfer {
            value_index: to_value_index,
            transfer_fn: self.transfer_fn,
        });
    }

    fn grow_num_nodes_if_needed(&mut self, new_index: VecIndex) {
        self.num_nodes = self.num_nodes.max(new_index + 1);
    }

    fn run(&self, node_values: &mut [NodeValue]) {
        for op in &self.ops {
            op.run(node_values);
        }
    }

    pub fn spawn(&self, randomness: &mut dyn MutationRandomness) -> Self {
        SparseNeuralNetGenome {
            ops: Self::copy_with_mutated_weights(&self.ops, randomness),
            transfer_fn: self.transfer_fn,
            num_nodes: self.num_nodes,
        }
    }

    fn copy_with_mutated_weights(ops: &[Op], randomness: &mut dyn MutationRandomness) -> Vec<Op> {
        ops.iter()
            .map(|op| op.copy_with_mutated_weight(|weight| randomness.mutate_weight(weight)))
            .collect()
    }

    pub fn print(&self, node_labels: &[&str]) {
        for printable_node in self.get_printable_nodes() {
            printable_node.println(&node_labels);
        }
    }

    fn get_printable_nodes(&self) -> Vec<PrintableNode> {
        let mut printable_nodes = HashMap::new();
        for op in &self.ops {
            match op {
                Op::Bias { value_index, bias } => {
                    Self::get_printable_node(&mut printable_nodes, *value_index).bias = *bias;
                }

                Op::Connection {
                    from_value_index,
                    to_value_index,
                    weight,
                } => {
                    Self::get_printable_node(&mut printable_nodes, *to_value_index)
                        .inputs
                        .push((*weight, *from_value_index));
                }

                Op::Transfer {
                    value_index,
                    transfer_fn,
                } => {
                    Self::get_printable_node(&mut printable_nodes, *value_index).transfer_fn =
                        *transfer_fn;
                }
            }
        }

        let mut printable_nodes = printable_nodes
            .values()
            .cloned()
            .collect::<Vec<PrintableNode>>();
        printable_nodes.sort_unstable_by(|a, b| a.index.partial_cmp(&b.index).unwrap());
        printable_nodes
    }

    fn get_printable_node(
        printable_nodes: &mut HashMap<VecIndex, PrintableNode>,
        value_index: VecIndex,
    ) -> &mut PrintableNode {
        printable_nodes
            .entry(value_index)
            .or_insert_with(|| PrintableNode::new(value_index))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct PrintableNode {
    index: VecIndex,
    inputs: Vec<(Coefficient, VecIndex)>,
    bias: Coefficient,
    transfer_fn: TransferFn,
}

impl PrintableNode {
    fn new(index: VecIndex) -> Self {
        PrintableNode {
            index,
            bias: 0.0,
            inputs: vec![],
            transfer_fn: TransferFn::IDENTITY,
        }
    }

    fn println(&self, node_labels: &[&str]) {
        println!(
            "  {} <- {}{}",
            Self::format_node_index(self.index, node_labels),
            self.format_inputs(node_labels),
            self.format_bias()
        );
    }

    fn format_inputs(&self, node_labels: &[&str]) -> String {
        let mut result = "".to_string();
        for (index, (coefficient, input_node_index)) in self.inputs.iter().enumerate() {
            result += &self.format_input(index, *coefficient, *input_node_index, node_labels);
        }
        result
    }

    fn format_input(
        &self,
        index: usize,
        coefficient: Coefficient,
        input_node_index: VecIndex,
        node_labels: &[&str],
    ) -> String {
        if coefficient == 0.0 {
            return "".to_string();
        }

        let mut result = "".to_string();
        if index > 0 {
            result += " + ";
        }
        #[allow(clippy::float_cmp)]
        if coefficient != 1.0 {
            result += &format!("{:.4}*", coefficient);
        }
        result += &format!("{}", Self::format_node_index(input_node_index, node_labels));
        result
    }

    fn format_node_index(node_index: VecIndex, node_labels: &[&str]) -> String {
        if (node_index as usize) < node_labels.len() {
            format!("[{}]", node_labels[node_index as usize])
        } else {
            format!("[{}]", node_index)
        }
    }

    fn format_bias(&self) -> String {
        if self.bias > 0.0 {
            format!(" + {:.4}", self.bias)
        } else if self.bias < 0.0 {
            format!(" - {:.4}", -self.bias)
        } else {
            "".to_string()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    Bias {
        value_index: VecIndex,
        bias: Coefficient,
    },
    Connection {
        from_value_index: VecIndex,
        to_value_index: VecIndex,
        weight: Coefficient,
    },
    Transfer {
        value_index: VecIndex,
        transfer_fn: TransferFn,
    },
}

impl Op {
    fn run(&self, node_values: &mut [NodeValue]) {
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
                transfer_fn.call(value);
            }
        }
    }

    fn copy_with_mutated_weight<F>(&self, mut mutate_weight: F) -> Self
    where
        F: FnMut(Coefficient) -> Coefficient,
    {
        match self {
            Self::Bias { value_index, bias } => Self::Bias {
                value_index: *value_index,
                bias: mutate_weight(*bias),
            },

            Self::Connection {
                from_value_index,
                to_value_index,
                weight,
            } => Self::Connection {
                from_value_index: *from_value_index,
                to_value_index: *to_value_index,
                weight: mutate_weight(*weight),
            },

            Self::Transfer {
                value_index,
                transfer_fn,
            } => Self::Transfer {
                value_index: *value_index,
                transfer_fn: *transfer_fn,
            },
        }
    }
}

#[derive(Copy)]
pub struct TransferFn {
    the_fn: fn(&mut NodeValue),
}

impl TransferFn {
    pub const IDENTITY: TransferFn = TransferFn {
        the_fn: Self::identity,
    };
    pub const SIGMOIDAL: TransferFn = TransferFn {
        the_fn: Self::sigmoidal,
    };

    pub fn new(the_fn: fn(&mut NodeValue)) -> Self {
        TransferFn { the_fn }
    }

    pub fn call(self, value: &mut NodeValue) {
        (self.the_fn)(value)
    }

    fn identity(_value: &mut NodeValue) {}

    fn sigmoidal(value: &mut NodeValue) {
        *value = Self::sigmoidal_fn(*value);
    }

    fn sigmoidal_fn(val: NodeValue) -> NodeValue {
        1.0_f32 / (1.0_f32 + (-4.9_f32 * val).exp())
    }
}

impl Clone for TransferFn {
    fn clone(&self) -> Self {
        *self
    }
}

impl fmt::Debug for TransferFn {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        // TODO match against constants and print name?
        write!(f, "{}", self.the_fn as usize)
    }
}

impl PartialEq for TransferFn {
    fn eq(&self, other: &Self) -> bool {
        self.the_fn as usize == other.the_fn as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MutationParameters {
    pub weight_mutation_probability: f32,
    pub weight_mutation_stdev: f32,
    pub add_node_probability: f32,
}

impl MutationParameters {
    pub const NO_MUTATION: MutationParameters = MutationParameters {
        weight_mutation_probability: 0.0,
        weight_mutation_stdev: 0.0,
        add_node_probability: 0.0,
    };

    fn _validate(&self) {
        assert!(Self::_is_probability(self.weight_mutation_probability));
    }

    fn _is_probability(num: f32) -> bool {
        0.0 <= num && num <= 1.0
    }
}

pub trait MutationRandomness {
    fn mutate_weight(&mut self, weight: Coefficient) -> Coefficient;
}

#[derive(Clone, Debug)]
pub struct SeededMutationRandomness {
    rng: Pcg64Mcg,
    mutation_parameters: &'static MutationParameters,
}

impl SeededMutationRandomness {
    pub fn new(seed: u64, mutation_parameters: &'static MutationParameters) -> Self {
        SeededMutationRandomness {
            rng: rand_pcg::Pcg64Mcg::seed_from_u64(seed),
            mutation_parameters,
        }
    }

    pub fn spawn(&mut self) -> Self {
        Self::new(self.child_seed(), self.mutation_parameters)
    }

    pub fn child_seed(&mut self) -> u64 {
        self.rng.gen()
    }

    fn should_mutate_this_weight(&mut self) -> bool {
        self.rng
            .gen_bool(self.mutation_parameters.weight_mutation_probability as f64)
    }
}

impl MutationRandomness for SeededMutationRandomness {
    fn mutate_weight(&mut self, weight: Coefficient) -> Coefficient {
        if !self.should_mutate_this_weight() {
            return weight;
        }

        // TODO weight cannot mutate away from 0.0
        let normal = Normal::new(
            weight,
            // weight.abs().sqrt()
            self.mutation_parameters.weight_mutation_stdev * weight.abs(),
        )
        .unwrap();
        normal.sample(&mut self.rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_layer_sparsely_connected() {
        let mut genome = SparseNeuralNetGenome::new(TransferFn::new(plus_one));
        genome.connect_node(2, 0.5, &[(0, 0.5)]);
        genome.connect_node(3, 0.0, &[(0, 0.75), (1, 0.25)]);

        let mut nnet = SparseNeuralNet::new(genome);
        nnet.set_node_value(0, 2.0);
        nnet.set_node_value(1, 4.0);
        nnet.run();

        assert_eq!(nnet.node_value(2), 2.5);
        assert_eq!(nnet.node_value(3), 3.5);
    }

    #[test]
    fn run_clears_previous_values() {
        let mut genome = SparseNeuralNetGenome::new(TransferFn::IDENTITY);
        genome.connect_node(1, 0.0, &[(0, 1.0)]);

        let mut nnet = SparseNeuralNet::new(genome);
        nnet.set_node_value(0, 1.0);
        nnet.run();
        nnet.set_node_value(0, 3.0);
        nnet.run();

        assert_eq!(nnet.node_value(1), 3.0);
    }

    #[test]
    fn three_layer() {
        let mut genome = SparseNeuralNetGenome::new(TransferFn::IDENTITY);
        genome.connect_node(1, 0.5, &[(0, 0.5)]);
        genome.connect_node(2, 0.0, &[(1, 0.5)]);

        let mut nnet = SparseNeuralNet::new(genome);
        nnet.set_node_value(0, 2.0);
        nnet.run();

        assert_eq!(nnet.node_value(2), 0.75);
    }

    #[test]
    fn recurrent_connection() {
        let mut genome = SparseNeuralNetGenome::new(TransferFn::IDENTITY);
        genome.connect_node(1, 0.0, &[(0, 1.0), (2, 2.0)]);
        genome.connect_node(2, 0.0, &[(1, 1.0)]);

        let mut nnet = SparseNeuralNet::new(genome);
        nnet.set_node_value(0, 1.0);
        nnet.run();

        assert_eq!(nnet.node_value(0), 1.0);
        assert_eq!(nnet.node_value(1), 1.0);
        assert_eq!(nnet.node_value(2), 1.0);

        nnet.set_node_value(0, 0.0);
        nnet.run();

        assert_eq!(nnet.node_value(0), 0.0);
        assert_eq!(nnet.node_value(1), 2.0);
        assert_eq!(nnet.node_value(2), 2.0);
    }

    #[test]
    fn spawn_unmutated() {
        let mut genome = SparseNeuralNetGenome::new(TransferFn::SIGMOIDAL);
        genome.connect_node(1, 0.0, &[(0, 1.0), (2, 2.0)]);
        genome.connect_node(2, 0.0, &[(1, 1.0)]);

        let mut randomness = StubMutationRandomness {
            mutated_weights: vec![],
        };
        let copy = genome.spawn(&mut randomness);

        assert_eq!(copy.ops, genome.ops);
        assert_eq!(copy.transfer_fn, TransferFn::SIGMOIDAL);
    }

    #[test]
    fn spawn_with_mutated_weights() {
        let mut genome = SparseNeuralNetGenome::new(TransferFn::SIGMOIDAL);
        genome.connect_node(2, 1.5, &[(0, 1.0), (1, 2.0)]);

        let mut randomness = StubMutationRandomness {
            mutated_weights: vec![(1.5, -0.5), (2.0, 2.25)],
        };
        let copy = genome.spawn(&mut randomness);

        assert_eq!(
            copy.ops,
            vec![
                Op::Bias {
                    value_index: 2,
                    bias: -0.5,
                },
                Op::Connection {
                    from_value_index: 0,
                    to_value_index: 2,
                    weight: 1.0,
                },
                Op::Connection {
                    from_value_index: 1,
                    to_value_index: 2,
                    weight: 2.25,
                },
                Op::Transfer {
                    value_index: 2,
                    transfer_fn: TransferFn::SIGMOIDAL,
                }
            ]
        );
    }

    #[test]
    fn seeded_mutation_randomness_leaves_weight_unmutated() {
        let mut randomness = SeededMutationRandomness::new(0, &MutationParameters::NO_MUTATION);
        assert_eq!(randomness.mutate_weight(1.0), 1.0);
    }

    #[test]
    fn seeded_mutation_randomness_mutates_weight() {
        const ALWAYS_MUTATE: MutationParameters = MutationParameters {
            weight_mutation_probability: 1.0,
            weight_mutation_stdev: 1.0,
            ..MutationParameters::NO_MUTATION
        };

        let mut randomness = SeededMutationRandomness::new(0, &ALWAYS_MUTATE);
        assert_ne!(randomness.mutate_weight(1.0), 1.0);
    }

    #[test]
    #[ignore]
    fn seeded_mutation_randomness_converges_to_zero() {
        const ALWAYS_MUTATE: MutationParameters = MutationParameters {
            weight_mutation_probability: 1.0,
            weight_mutation_stdev: 0.2,
            ..MutationParameters::NO_MUTATION
        };

        let mut randomness = SeededMutationRandomness::new(0, &ALWAYS_MUTATE);
        let mut weight = 100.0;
        for _i in 0..200 {
            println!("{}", weight);
            weight = randomness.mutate_weight(weight);
        }
    }

    fn plus_one(value: &mut NodeValue) {
        *value += 1.0;
    }

    struct StubMutationRandomness {
        mutated_weights: Vec<(Coefficient, Coefficient)>,
    }

    impl MutationRandomness for StubMutationRandomness {
        fn mutate_weight(&mut self, weight: Coefficient) -> Coefficient {
            for (from_weight, to_weight) in &self.mutated_weights {
                if *from_weight == weight {
                    return *to_weight;
                }
            }
            weight
        }
    }
}
