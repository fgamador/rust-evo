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
        .with_sunlight(0.0, 10.0)
        .with_cell(Cell::new(
            Position::new(200.0, -50.0), Velocity::ZERO,
            vec![
                Box::new(CellLayer::new(Area::new(200.0 * PI), Density::new(1.0), Color::Green,
                                        Box::new(PhotoCellLayerSpecialty::new(1.0)))
                    .with_resize_parameters(LayerResizeParameters {
                        growth_energy_delta: BioEnergyDelta::new(-1.0),
                        ..LayerResizeParameters::UNLIMITED
                    })
                    .with_health_parameters(LayerHealthParameters {
                        healing_energy_delta: BioEnergyDelta::ZERO,
                        entropic_damage_health_delta: -0.006,
                    })),
            ])
            .with_control(Box::new(GrowThenHealControl::new(0, 100, AreaDelta::new(10.0), 100, 0.01))))
}

#[derive(Clone, Debug)]
pub struct GrowThenHealControl {
    layer_index: usize,
    growth_ticks: u32,
    growth_delta_area: AreaDelta,
    healing_ticks: u32,
    healing_delta: f64,
    ticks: u32,
}

impl GrowThenHealControl {
    pub fn new(layer_index: usize, growth_ticks: u32, growth_delta_area: AreaDelta, healing_ticks: u32, healing_delta: f64) -> Self {
        GrowThenHealControl {
            layer_index,
            growth_ticks,
            growth_delta_area,
            healing_ticks,
            healing_delta,
            ticks: 0,
        }
    }
}

impl CellControl for GrowThenHealControl {
    fn box_clone(&self) -> Box<CellControl> {
        Box::new(self.clone())
    }

    fn get_control_requests(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let request =
            if self.ticks <= self.growth_ticks {
                CellLayer::resize_request(self.layer_index, self.growth_delta_area)
            } else {
                CellLayer::healing_request(self.layer_index, self.healing_delta)
            };

        self.ticks += 1;
        if self.ticks >= self.growth_ticks + self.healing_ticks {
            self.ticks = 0;
        }

        vec![request]
    }
}
