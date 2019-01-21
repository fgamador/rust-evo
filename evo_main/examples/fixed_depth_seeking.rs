extern crate evo_main;
extern crate evo_model;
extern crate evo_view_model;

use evo_model::biology::control::*;
use evo_model::biology::control_requests::*;
use evo_model::biology::layers::*;
use evo_model::biology::cell::Cell;
use evo_model::environment::influences::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_main::main_support::init_and_run;
use evo_view_model::Color;
use std::f64::consts::PI;

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
                Position::new(150.0, -300.0), Velocity::new(0.0, 0.0),
                vec![
                    Box::new(simple_cell_layer(
                        Area::new(100.0 * PI), Density::new(0.0004), Color::White)),
                    Box::new(simple_cell_layer(
                        Area::new(300.0 * PI), Density::new(0.00075), Color::Green)),
                ])
                .with_control(Box::new(FixedDepthSeekingControl::new(0, -150.0))),
            Cell::new(
                Position::new(250.0, -100.0), Velocity::new(0.0, 0.0),
                vec![
                    Box::new(simple_cell_layer(
                        Area::new(50.0 * PI), Density::new(0.0004), Color::White)),
                    Box::new(simple_cell_layer(
                        Area::new(150.0 * PI), Density::new(0.00075), Color::Green)),
                ])
                .with_control(Box::new(FixedDepthSeekingControl::new(0, -250.0))),
        ])
}

fn simple_cell_layer(area: Area, density: Density, color: Color) -> CellLayer {
    CellLayer::new(area, density, color, Box::new(NullCellLayerSpecialty::new()))
}

#[derive(Clone, Debug)]
pub struct FixedDepthSeekingControl {
    float_layer_index: usize,
    target_y: f64,
}

impl FixedDepthSeekingControl {
    pub fn new(float_layer_index: usize, target_y: f64) -> Self {
        FixedDepthSeekingControl {
            float_layer_index,
            target_y,
        }
    }
}

impl CellControl for FixedDepthSeekingControl {
    fn box_clone(&self) -> Box<CellControl> {
        Box::new(self.clone())
    }

    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let y_offset = cell_state.center.y() - self.target_y;
        let target_velocity_y = -y_offset / 100.0;
        let target_delta_vy = target_velocity_y - cell_state.velocity.y();
        let desired_delta_area = target_delta_vy * 10.0;
        vec![CellLayer::resize_request(self.float_layer_index, AreaDelta::new(desired_delta_area))]
    }
}
