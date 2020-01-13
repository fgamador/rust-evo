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
use std::f64;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_sunlight(0.0, 1.0)
        .with_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(-0.05)))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(
                -0.03, 0.001,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.005)))),
        ])
        .with_cells(vec![create_cell()
            .with_initial_position(Position::new(200.0, -50.0))
            .with_initial_energy(BioEnergy::new(100.0))])
}

fn create_cell() -> Cell {
    Cell::new(
        Position::ORIGIN,
        Velocity::ZERO,
        vec![
            create_float_layer(),
            create_photo_layer(),
            create_budding_layer(),
        ],
    )
    .with_control(Box::new(DuckweedControl::new(-50.0)))
}

fn create_float_layer() -> Box<CellLayer> {
    Box::new(
        CellLayer::new(
            Area::new(5.0 * PI),
            Density::new(0.0004),
            Color::White,
            Box::new(NullCellLayerSpecialty::new()),
        )
        .with_resize_parameters(LayerResizeParameters {
            growth_energy_delta: BioEnergyDelta::new(100.0),
            max_growth_rate: 10.0,
            shrinkage_energy_delta: BioEnergyDelta::new(1.0),
            max_shrinkage_rate: 0.5,
        })
        .with_health_parameters(LayerHealthParameters {
            healing_energy_delta: BioEnergyDelta::new(1.0),
            entropic_damage_health_delta: -0.1,
            overlap_damage_health_delta: 0.0,
        }),
    )
}

fn create_photo_layer() -> Box<CellLayer> {
    Box::new(
        CellLayer::new(
            Area::new(5.0 * PI),
            Density::new(0.00075),
            Color::Green,
            Box::new(PhotoCellLayerSpecialty::new(0.01)),
        )
        .with_resize_parameters(LayerResizeParameters {
            growth_energy_delta: BioEnergyDelta::new(1000.0),
            max_growth_rate: 10.0,
            shrinkage_energy_delta: BioEnergyDelta::new(0.0),
            max_shrinkage_rate: 0.1,
        })
        .with_health_parameters(LayerHealthParameters {
            healing_energy_delta: BioEnergyDelta::new(1.0),
            entropic_damage_health_delta: -0.5,
            overlap_damage_health_delta: 0.0,
        }),
    )
}

fn create_budding_layer() -> Box<CellLayer> {
    Box::new(
        CellLayer::new(
            Area::new(5.0 * PI),
            Density::new(0.00075),
            Color::Yellow,
            Box::new(BuddingCellLayerSpecialty::new(create_cell)),
        )
        .with_resize_parameters(LayerResizeParameters {
            growth_energy_delta: BioEnergyDelta::new(0.0),
            max_growth_rate: f64::INFINITY,
            shrinkage_energy_delta: BioEnergyDelta::new(0.0),
            max_shrinkage_rate: 1.0,
        })
        .with_health_parameters(LayerHealthParameters {
            healing_energy_delta: BioEnergyDelta::new(1.0),
            entropic_damage_health_delta: -0.5,
            overlap_damage_health_delta: -0.5,
        }),
    )
}

#[derive(Debug)]
pub struct DuckweedControl {
    target_y: f64,
    budding_ticks: u32,
    budding_angle: Angle,
    tick: u32,
}

impl DuckweedControl {
    pub fn new(target_y: f64) -> Self {
        DuckweedControl {
            target_y,
            budding_ticks: 100,
            budding_angle: Angle::from_radians(0.0),
            tick: 0,
        }
    }

    fn is_adult(cell_state: &CellStateSnapshot) -> bool {
        cell_state.area >= Area::new(1000.0)
    }

    fn adult_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let mut requests = vec![self.float_layer_resize_request(cell_state)];
        requests.append(&mut self.budding_requests());
        requests.append(&mut self.healing_requests(cell_state));
        requests
    }

    fn youth_requests(&self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let mut requests = vec![
            self.float_layer_resize_request(cell_state),
            CellLayer::resize_request(1, AreaDelta::new(5.0)),
            CellLayer::resize_request(2, AreaDelta::new(5.0)),
        ];
        requests.append(&mut self.healing_requests(cell_state));
        requests
    }

    fn float_layer_resize_request(&self, cell_state: &CellStateSnapshot) -> ControlRequest {
        let y_offset = cell_state.center.y() - self.target_y;
        let target_velocity_y = -y_offset / 100.0;
        let target_delta_vy = target_velocity_y - cell_state.velocity.y();
        let desired_delta_area = target_delta_vy * 10.0;
        CellLayer::resize_request(0, AreaDelta::new(desired_delta_area))
    }

    fn budding_requests(&mut self) -> Vec<ControlRequest> {
        self.tick += 1;
        if self.tick < self.budding_ticks {
            return vec![];
        }

        self.tick = 0;
        self.budding_angle += Deflection::from_radians(PI / 4.0);
        vec![
            BuddingCellLayerSpecialty::budding_angle_request(2, self.budding_angle),
            BuddingCellLayerSpecialty::donation_energy_request(2, BioEnergy::new(100.0)),
        ]
    }

    fn healing_requests(&self, _cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        vec![
            CellLayer::healing_request(0, 100.0),
            CellLayer::healing_request(1, 100.0),
            CellLayer::healing_request(2, 100.0),
        ]
    }
}

impl CellControl for DuckweedControl {
    fn get_control_requests(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        if Self::is_adult(cell_state) {
            self.adult_requests(cell_state)
        } else {
            self.youth_requests(cell_state)
        }
    }
}
