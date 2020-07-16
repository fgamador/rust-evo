use evo_domain::biology::cell::Cell;
use evo_domain::biology::control::*;
use evo_domain::biology::control_requests::*;
use evo_domain::biology::layers::*;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::init_and_run;
use std::f64;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

const FLUID_DENSITY: f64 = 0.001;
const FLOAT_LAYER_DENSITY: f64 = 0.0001;
const PHOTO_LAYER_DENSITY: f64 = 0.002;
const BUDDING_LAYER_DENSITY: f64 = 0.002;
const GRAVITY: f64 = -0.05;
const OVERLAP_DAMAGE_HEALTH_DELTA: f64 = -0.1;

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(1000.0, 0.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_sunlight(0.0, 1.0)
        .with_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(
                GRAVITY,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(
                GRAVITY,
                FLUID_DENSITY,
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

fn create_float_layer() -> CellLayer {
    const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::new(-0.1),
        max_growth_rate: 10.0,
        shrinkage_energy_delta: BioEnergyDelta::new(-0.01),
        max_shrinkage_rate: 0.5,
    };
    const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: -0.01,
        overlap_damage_health_delta: OVERLAP_DAMAGE_HEALTH_DELTA,
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(FLOAT_LAYER_DENSITY),
        Color::White,
        Box::new(NullCellLayerSpecialty::new()),
    )
    .with_resize_parameters(&LAYER_RESIZE_PARAMS)
    .with_health_parameters(&LAYER_HEALTH_PARAMS)
}

fn create_photo_layer() -> CellLayer {
    const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::new(-1.0),
        max_growth_rate: 10.0,
        shrinkage_energy_delta: BioEnergyDelta::new(0.0),
        max_shrinkage_rate: 0.1,
    };
    const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: -0.01,
        overlap_damage_health_delta: OVERLAP_DAMAGE_HEALTH_DELTA,
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(PHOTO_LAYER_DENSITY),
        Color::Green,
        Box::new(PhotoCellLayerSpecialty::new(0.5)),
    )
    .with_resize_parameters(&LAYER_RESIZE_PARAMS)
    .with_health_parameters(&LAYER_HEALTH_PARAMS)
}

fn create_budding_layer() -> CellLayer {
    const LAYER_RESIZE_PARAMS: LayerResizeParameters = LayerResizeParameters {
        growth_energy_delta: BioEnergyDelta::new(0.0),
        max_growth_rate: f64::INFINITY,
        shrinkage_energy_delta: BioEnergyDelta::new(0.0),
        max_shrinkage_rate: 1.0,
    };
    const LAYER_HEALTH_PARAMS: LayerHealthParameters = LayerHealthParameters {
        healing_energy_delta: BioEnergyDelta::new(-1.0),
        entropic_damage_health_delta: -0.01,
        overlap_damage_health_delta: OVERLAP_DAMAGE_HEALTH_DELTA,
    };

    CellLayer::new(
        Area::new(5.0 * PI),
        Density::new(BUDDING_LAYER_DENSITY),
        Color::Yellow,
        Box::new(BondingCellLayerSpecialty::new()),
    )
    .with_resize_parameters(&LAYER_RESIZE_PARAMS)
    .with_health_parameters(&LAYER_HEALTH_PARAMS)
}

#[derive(Clone, Debug)]
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
        let float_layer = &cell_state.layers[0];
        let y_ratio = self.target_y / cell_state.center.y();
        let target_density = y_ratio * FLUID_DENSITY;
        let target_float_area = self.calc_target_float_area(cell_state, target_density);
        let desired_delta_area = target_float_area - float_layer.area.value();
        CellLayer::resize_request(0, AreaDelta::new(desired_delta_area))
    }

    fn calc_target_float_area(&self, cell_state: &CellStateSnapshot, target_density: f64) -> f64 {
        let photo_layer = &cell_state.layers[1];
        let budding_layer = &cell_state.layers[2];
        ((photo_layer.area.value() * (PHOTO_LAYER_DENSITY - target_density)
            + (budding_layer.area.value() * (BUDDING_LAYER_DENSITY - target_density)))
            / (target_density - FLOAT_LAYER_DENSITY))
            .max(0.0)
    }

    fn budding_requests(&mut self) -> Vec<ControlRequest> {
        self.tick += 1;
        if self.tick < self.budding_ticks {
            return vec![];
        }

        self.tick = 0;
        self.budding_angle += Deflection::from_radians(PI / 4.0);
        vec![
            BondingCellLayerSpecialty::retain_bond_request(2, 0, true),
            BondingCellLayerSpecialty::budding_angle_request(2, 0, self.budding_angle),
            BondingCellLayerSpecialty::donation_energy_request(2, 0, BioEnergy::new(100.0)),
        ]
    }

    fn healing_requests(&self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        let mut requests = Vec::with_capacity(cell_state.layers.len());
        for (i, layer) in cell_state.layers.iter().enumerate() {
            let delta_health = 1.0 - layer.health.value();
            requests.push(CellLayer::healing_request(i, delta_health));
        }
        requests
    }
}

impl CellControl for DuckweedControl {
    fn run(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        if Self::is_adult(cell_state) {
            self.adult_requests(cell_state)
        } else {
            self.youth_requests(cell_state)
        }
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}
