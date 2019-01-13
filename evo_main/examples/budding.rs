extern crate evo_main;
extern crate evo_model;
extern crate evo_view_model;

use evo_model::biology::cell::Cell;
use evo_model::biology::control::*;
use evo_model::biology::control_requests::*;
use evo_model::biology::layers::*;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_main::main_support::init_and_run;
use evo_view_model::Color;
use std::f64;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_cell(Cell::new(
            Position::new(200.0, -100.0), Velocity::ZERO,
            vec![
                Box::new(CellLayer::new(Area::new(50.0 * PI), Density::new(1.0), Color::White,
                                        Box::new(EnergyGeneratingCellLayerSpecialty::new()))),
                Box::new(CellLayer::new(Area::new(50.0 * PI), Density::new(1.0), Color::Green,
                                        Box::new(BuddingCellLayerSpecialty::new()))
                    .with_resize_parameters(LayerResizeParameters {
                        growth_energy_delta: BioEnergyDelta::new(-1.0),
                        max_growth_rate: f64::INFINITY,
                        shrinkage_energy_delta: BioEnergyDelta::ZERO,
                        max_shrinkage_rate: f64::INFINITY,
                    }))
            ])
            .with_control(Box::new(BuddingControl {})))
}

#[derive(Debug)]
pub struct BuddingControl {}

impl BuddingControl {
    fn is_parent(cell_state: &CellStateSnapshot) -> bool {
        cell_state.area >= Area::new(100.0)
    }

    fn parent_requests() -> Vec<ControlRequest> {
        vec![
            EnergyGeneratingCellLayerSpecialty::energy_request(0, 1.0),
            BuddingCellLayerSpecialty::budding_angle_request(1, PI / 2.0),
            BuddingCellLayerSpecialty::donation_energy_request(1, 1.0),
        ]
    }

    fn child_requests() -> Vec<ControlRequest> {
        vec![
            CellLayer::resize_request(1, 10.0)
        ]
    }
}

impl CellControl for BuddingControl {
    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        if Self::is_parent(cell_state) {
            Self::parent_requests()
        } else {
            Self::child_requests()
        }
    }
}
