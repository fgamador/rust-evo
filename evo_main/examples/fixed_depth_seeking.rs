use evo_domain::biology::cell::Cell;
use evo_domain::biology::control::*;
use evo_domain::biology::control_requests::*;
use evo_domain::biology::layers::*;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;
use std::f64::consts::PI;

fn main() {
    init_and_run(|_seed| create_world());
}

const FLUID_DENSITY: f64 = 0.001;
const FLOAT_LAYER_DENSITY: f64 = 0.0001;
const OTHER_LAYER_DENSITY: f64 = 0.002;
const GRAVITY: f64 = -0.05;

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_standard_influences()
        .with_per_cell_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(
                GRAVITY,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(
                GRAVITY,
                FLUID_DENSITY,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.005)))),
        ])
        .with_cells(vec![
            Cell::new(
                Position::new(150.0, -300.0),
                Velocity::new(0.0, 0.0),
                vec![
                    simple_cell_layer(
                        Area::new(100.0 * PI),
                        Density::new(FLOAT_LAYER_DENSITY),
                        Tissue::AirBubble,
                    ),
                    simple_cell_layer(
                        Area::new(300.0 * PI),
                        Density::new(OTHER_LAYER_DENSITY),
                        Tissue::Photosynthetic,
                    ),
                ],
            )
            .with_control(Box::new(FixedDepthSeekingControl::new(-150.0))),
            Cell::new(
                Position::new(250.0, -100.0),
                Velocity::new(0.0, 0.0),
                vec![
                    simple_cell_layer(
                        Area::new(50.0 * PI),
                        Density::new(FLOAT_LAYER_DENSITY),
                        Tissue::AirBubble,
                    ),
                    simple_cell_layer(
                        Area::new(150.0 * PI),
                        Density::new(OTHER_LAYER_DENSITY),
                        Tissue::Photosynthetic,
                    ),
                ],
            )
            .with_control(Box::new(FixedDepthSeekingControl::new(-250.0))),
        ])
}

fn simple_cell_layer(area: Area, density: Density, color: Tissue) -> CellLayer {
    CellLayer::new(
        area,
        density,
        color,
        Box::new(NullCellLayerSpecialty::new()),
    )
}

#[derive(Clone, Debug)]
pub struct FixedDepthSeekingControl {
    target_y: f64,
}

impl FixedDepthSeekingControl {
    pub fn new(target_y: f64) -> Self {
        FixedDepthSeekingControl { target_y }
    }

    fn float_layer_resize_request(&self, cell_state: &CellStateSnapshot) -> ControlRequest {
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
}

impl CellControl for FixedDepthSeekingControl {
    fn run(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![self.float_layer_resize_request(cell_state)]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}
