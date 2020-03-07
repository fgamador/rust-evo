extern crate evo_main;
extern crate evo_model;

use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::biology::control::*;
use evo_model::biology::control_requests::*;
use evo_model::biology::genome::*;
use evo_model::biology::layers::*;
use evo_model::environment::influences::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    let mut genome = SparseNeuralNetGenome::new(TransferFn::IDENTITY);
    genome.connect_node(1, -100.0, &[(0, -1.0)]);

    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(-0.05)))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(
                -0.03, 0.001,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.005)))),
        ])
        .with_cells(vec![Cell::new(
            Position::new(200.0, -200.0),
            Velocity::new(0.0, 0.0),
            vec![
                simple_cell_layer(Area::new(100.0 * PI), Density::new(0.0004), Color::White),
                simple_cell_layer(Area::new(300.0 * PI), Density::new(0.00075), Color::Green),
            ],
        )
        .with_control(Box::new(NeuralNetControl::new(genome)))])
}

fn simple_cell_layer(area: Area, density: Density, color: Color) -> CellLayer {
    CellLayer::new(
        area,
        density,
        color,
        Box::new(NullCellLayerSpecialty::new()),
    )
}

#[derive(Clone, Debug)]
pub struct NeuralNetControl {
    nnet: SparseNeuralNet,
}

impl NeuralNetControl {
    pub fn new(genome: SparseNeuralNetGenome) -> Self {
        NeuralNetControl {
            nnet: SparseNeuralNet::new(genome),
        }
    }
}

impl CellControl for NeuralNetControl {
    fn run(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        self.nnet.set_node_value(0, cell_state.center.y() as f32);
        self.nnet.run();
        vec![CellLayer::resize_request(
            0,
            AreaDelta::new(self.nnet.node_value(1) as f64),
        )]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}
