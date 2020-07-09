use evo_domain::biology::cell::Cell;
use evo_domain::biology::control::*;
use evo_domain::biology::control_requests::*;
use evo_domain::biology::layers::*;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::init_and_run;
use std::f64::consts::PI;

fn main() {
    init_and_run(create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_perimeter_walls()
        .with_pair_collisions()
        .with_influences(vec![
            Box::new(BondForces::new()),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.0005)))),
        ])
        .with_cell(create_cell().with_initial_position(Position::new(200.0, -100.0)))
}

fn create_cell() -> Cell {
    Cell::new(
        Position::ORIGIN,
        Velocity::ZERO,
        vec![
            CellLayer::new(
                Area::new(5.0 * PI),
                Density::new(0.002),
                Color::Green,
                Box::new(NullCellLayerSpecialty::new()),
            ),
            CellLayer::new(
                Area::new(5.0 * PI),
                Density::new(0.002),
                Color::Yellow,
                Box::new(BondingCellLayerSpecialty::new()),
            ),
        ],
    )
    .with_control(Box::new(BuddingControl::new(1)))
    .with_initial_energy(BioEnergy::new(4.0))
}

#[derive(Clone, Debug)]
pub struct BuddingControl {
    budding_layer_index: usize,
    budding_ticks: u32,
    budding_angle: Angle,
    adult_tick: u32,
}

impl BuddingControl {
    fn new(budding_layer_index: usize) -> Self {
        BuddingControl {
            budding_layer_index,
            budding_ticks: 100,
            budding_angle: Angle::from_radians(0.0),
            adult_tick: 0,
        }
    }

    fn is_adult(cell_state: &CellStateSnapshot) -> bool {
        cell_state.area >= Area::new(1000.0)
    }

    fn adult_requests(&mut self) -> Vec<ControlRequest> {
        self.adult_tick += 1;
        if self.adult_tick < self.budding_ticks {
            return vec![BondingCellLayerSpecialty::retain_bond_request(
                self.budding_layer_index,
                1,
                true,
            )];
        }

        self.adult_tick = 0;
        self.budding_angle += Deflection::from_radians(PI / 4.0);
        vec![
            BondingCellLayerSpecialty::retain_bond_request(self.budding_layer_index, 1, true),
            BondingCellLayerSpecialty::budding_angle_request(
                self.budding_layer_index,
                1,
                self.budding_angle,
            ),
            BondingCellLayerSpecialty::donation_energy_request(
                self.budding_layer_index,
                1,
                BioEnergy::new(1.0),
            ),
        ]
    }

    fn youth_requests(&self) -> Vec<ControlRequest> {
        vec![
            CellLayer::resize_request(0, AreaDelta::new(5.0)),
            CellLayer::resize_request(self.budding_layer_index, AreaDelta::new(5.0)),
            BondingCellLayerSpecialty::retain_bond_request(self.budding_layer_index, 0, true),
        ]
    }
}

impl CellControl for BuddingControl {
    fn run(&mut self, cell_state: &CellStateSnapshot) -> Vec<ControlRequest> {
        if Self::is_adult(cell_state) {
            self.adult_requests()
        } else {
            self.youth_requests()
        }
    }

    fn spawn(&mut self) -> Box<dyn CellControl> {
        Box::new(self.clone())
    }
}
