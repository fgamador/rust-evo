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

const FLUID_DENSITY: f64 = 0.001;
const FLOAT_LAYER_DENSITY: f64 = 0.0004;
const OTHER_LAYER_DENSITY: f64 = 0.00075;

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(-0.05)))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(
                -0.03,
                FLUID_DENSITY,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.005)))),
        ])
        .with_cells(vec![
            Cell::new(
                Position::new(150.0, -300.0),
                Velocity::new(0.0, 0.0),
                vec![
                    Box::new(simple_cell_layer(
                        Area::new(100.0 * PI),
                        Density::new(FLOAT_LAYER_DENSITY),
                        Color::White,
                    )),
                    Box::new(simple_cell_layer(
                        Area::new(300.0 * PI),
                        Density::new(OTHER_LAYER_DENSITY),
                        Color::Green,
                    )),
                ],
            )
            .with_control(Box::new(FixedDepthSeekingControl::new(-150.0))),
            Cell::new(
                Position::new(250.0, -100.0),
                Velocity::new(0.0, 0.0),
                vec![
                    Box::new(simple_cell_layer(
                        Area::new(50.0 * PI),
                        Density::new(FLOAT_LAYER_DENSITY),
                        Color::White,
                    )),
                    Box::new(simple_cell_layer(
                        Area::new(150.0 * PI),
                        Density::new(OTHER_LAYER_DENSITY),
                        Color::Green,
                    )),
                ],
            )
            .with_control(Box::new(FixedDepthSeekingControl::new(-250.0))),
        ])
}

fn simple_cell_layer(area: Area, density: Density, color: Color) -> CellLayer {
    CellLayer::new(
        area,
        density,
        color,
        Box::new(NullCellLayerSpecialty::new()),
    )
}

#[derive(Debug)]
pub struct FixedDepthSeekingControl {
    target_y: f64,
}

impl FixedDepthSeekingControl {
    pub fn new(target_y: f64) -> Self {
        FixedDepthSeekingControl { target_y }
    }

    fn _float_layer_resize_request(&self, cell_state: &CellStateSnapshot) -> ControlRequest {
        let y_ratio = self.target_y / cell_state.center.y();
        let target_density = y_ratio * FLUID_DENSITY;
        let float_layer = &cell_state.layers[0];
        let other_layer = &cell_state.layers[1];
        let target_float_area = (other_layer.area.value() * (OTHER_LAYER_DENSITY - target_density)
            / (target_density - FLOAT_LAYER_DENSITY))
            .max(0.0);
        let desired_delta_area = target_float_area - float_layer.area.value();
        CellLayer::resize_request(0, AreaDelta::new(desired_delta_area))
    }

    fn float_layer_resize_request(&self, cell_state: &CellStateSnapshot) -> ControlRequest {
        let y_offset = cell_state.center.y() - self.target_y;
        let target_velocity_y = -y_offset / 100.0;
        let target_delta_vy = target_velocity_y - cell_state.velocity.y();
        let desired_delta_area = target_delta_vy * 10.0;
        CellLayer::resize_request(0, AreaDelta::new(desired_delta_area))
    }
}

impl CellControl for FixedDepthSeekingControl {
    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![self.float_layer_resize_request(cell_state)]
    }
}
