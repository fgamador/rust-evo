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
            .with_control(Box::new(BuddingControl::new(Area::new(100.0), Area::new(10.0)))))
}

#[derive(Debug)]
pub struct BuddingControl {
    min_parent_area: Area,
    min_child_area: Area,
}

impl BuddingControl {
    pub fn new(min_parent_area: Area, min_child_area: Area) -> Self {
        BuddingControl {
            min_parent_area,
            min_child_area,
        }
    }
}

impl CellControl for BuddingControl {
    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![
            ControlRequest::new(0, 2, 1.0),
            ControlRequest::new(1, 2, PI / 2.0),
            ControlRequest::new(1, 3, 1.0),
        ]
    }
}
