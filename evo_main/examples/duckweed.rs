extern crate evo_main;
extern crate evo_model;

use evo_main::main_support::init_and_run;
use evo_model::biology::cell::Cell;
use evo_model::biology::control::*;
use evo_model::biology::control_requests::*;
use evo_model::biology::layers::*;
use evo_model::environment::influences::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_sunlight(0.0, 10.0)
        .with_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(-0.05)))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(
                -0.03, 0.001,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.005)))),
        ])
        .with_cells(vec![create_child()])
}

fn create_child() -> Cell {
    Cell::new(
        Position::new(200.0, -50.0),
        Velocity::new(0.0, 0.0),
        vec![
            Box::new(CellLayer::new(
                Area::new(200.0 * PI),
                Density::new(0.0004),
                Color::White,
                Box::new(NullCellLayerSpecialty::new()),
            )),
            Box::new(CellLayer::new(
                Area::new(300.0 * PI),
                Density::new(0.00075),
                Color::Green,
                Box::new(PhotoCellLayerSpecialty::new(1.0)),
            )),
        ],
    ).with_control(Box::new(DuckweedControl::new(0, -50.0)))
}

#[derive(Debug)]
pub struct DuckweedControl {
    float_layer_index: usize,
    target_y: f64,
}

impl DuckweedControl {
    pub fn new(float_layer_index: usize, target_y: f64) -> Self {
        DuckweedControl {
            float_layer_index,
            target_y,
        }
    }

    fn float_layer_resize_request(&self, cell_state: &CellStateSnapshot) -> ControlRequest {
        let y_offset = cell_state.center.y() - self.target_y;
        let target_velocity_y = -y_offset / 100.0;
        let target_delta_vy = target_velocity_y - cell_state.velocity.y();
        let desired_delta_area = target_delta_vy * 10.0;
        CellLayer::resize_request(self.float_layer_index, AreaDelta::new(desired_delta_area))
    }
}

impl CellControl for DuckweedControl {
    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![self.float_layer_resize_request(cell_state)]
    }
}
