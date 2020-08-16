use evo_domain::biology::cell::Cell;
use evo_domain::biology::control::*;
use evo_domain::biology::control_requests::*;
use evo_domain::biology::layers::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;
use std::f64;
use std::f64::consts::PI;

fn main() {
    let args = parse_command_line();
    init_and_run(create_world(), args);
}

fn create_world() -> World {
    const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::new(-1.0),
        ..LayerResizeParameters::UNLIMITED
    };
    const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::ZERO,
        entropic_damage_health_delta: HealthDelta::new(-0.006),
        ..LayerHealthParameters::DEFAULT
    };

    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_standard_influences()
        .with_sunlight(0.0, 10.0)
        .with_cell(
            Cell::new(
                Position::new(200.0, -50.0),
                Velocity::ZERO,
                vec![CellLayer::new(
                    Area::new(200.0 * PI),
                    Density::new(1.0),
                    Color::Green,
                    Box::new(PhotoCellLayerSpecialty::new(Fraction::ONE)),
                )
                .with_resize_parameters(&LAYER_RESIZE_PARAMS)
                .with_health_parameters(&LAYER_HEALTH_PARAMS)],
            )
            .with_control(Box::new(GrowThenHealControl::new(
                0,
                100,
                AreaDelta::new(10.0),
                100,
                0.01,
            ))),
        )
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
    pub fn new(
        layer_index: usize,
        growth_ticks: u32,
        growth_delta_area: AreaDelta,
        healing_ticks: u32,
        healing_delta: f64,
    ) -> Self {
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
    fn run(&mut self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let request = if self.ticks <= self.growth_ticks {
            CellLayer::resize_request(self.layer_index, self.growth_delta_area)
        } else {
            CellLayer::healing_request(self.layer_index, HealthDelta::new(self.healing_delta))
        };

        self.ticks += 1;
        if self.ticks >= self.growth_ticks + self.healing_ticks {
            self.ticks = 0;
        }

        vec![request]
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}
