extern crate evo_main;
extern crate evo_model;
extern crate evo_view_model;

use evo_model::biology::control::*;
use evo_model::biology::control_requests::*;
use evo_model::biology::layers::*;
use evo_model::biology::cell::Cell;
use evo_model::environment::influences::*;
use evo_model::neural::sparse_neural_net::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_main::main_support::init_and_run;
use evo_view_model::Color;
use std::f64::consts::PI;
use std::fmt::Error;
use std::fmt::Formatter;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(-0.05)))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(-0.03, 0.001)))),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.005))))
        ])
        .with_cells(vec![
            Cell::new(
                Position::new(200.0, -200.0), Velocity::new(0.0, 0.0),
                vec![
                    Box::new(simple_cell_layer(
                        Area::new(100.0 * PI), Density::new(0.0004), Color::White)),
                    Box::new(simple_cell_layer(
                        Area::new(300.0 * PI), Density::new(0.00075), Color::Green)),
                ])
                .with_control(Box::new(NeuralNetControl::new(0))),
        ])
}

fn simple_cell_layer(area: Area, density: Density, color: Color) -> CellLayer {
    CellLayer::new(area, density, color, Box::new(NullCellLayerSpecialty::new()))
}

pub struct NeuralNetControl {
    nnet: SparseNeuralNet,
    float_layer_index: usize,
}

impl NeuralNetControl {
    pub fn new(float_layer_index: usize) -> Self {
        NeuralNetControl {
            nnet: SparseNeuralNet::fully_connected(1, 1, -1.0, SparseNeuralNet::sigmoidal),
            float_layer_index,
        }
    }
}

impl CellControl for NeuralNetControl {
    fn box_clone(&self) -> Box<CellControl> {
        Box::new(NullControl {})
    }

    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        self.nnet.set_input(0, cell_state.center.y() as f32);
        self.nnet.run();
        let desired_delta_area = self.nnet.output(0) as f64;
        vec![CellLayer::resize_request(self.float_layer_index, AreaDelta::new(desired_delta_area))]
    }
}

impl std::fmt::Debug for NeuralNetControl {
    fn fmt(&self, _f: &mut Formatter) -> Result<(), Error> {
        unimplemented!()
    }
}
