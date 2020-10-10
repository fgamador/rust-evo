use evo_domain::biology::cell::Cell;
use evo_domain::biology::control::*;
use evo_domain::biology::genome::*;
use evo_domain::biology::layers::*;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;
use std::f64::consts::PI;

const FLOAT_LAYER_INDEX: usize = 0;
//const PHOTO_LAYER_INDEX: usize = 1;

fn main() {
    init_and_run(|_seed| create_world());
}

fn create_world() -> World {
    World::new(Position::new(0.0, -400.0), Position::new(400.0, 0.0))
        .with_standard_influences()
        .with_per_cell_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(-0.05)))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(
                -0.03, 0.001,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.005)))),
        ])
        .with_cells(vec![Cell::new(
            Position::new(200.0, -200.0),
            Velocity::new(0.0, 0.0),
            vec![
                simple_cell_layer(
                    Area::new(100.0 * PI),
                    Density::new(0.0004),
                    Tissue::AirBubble,
                ),
                simple_cell_layer(
                    Area::new(300.0 * PI),
                    Density::new(0.00075),
                    Tissue::Photosynthetic,
                ),
            ],
        )
        .with_control(Box::new(create_control(SeededMutationRandomness::new(
            0,
            &MutationParameters::NO_MUTATION,
        ))))])
}

fn simple_cell_layer(area: Area, density: Density, color: Tissue) -> CellLayer {
    CellLayer::new(
        area,
        density,
        color,
        Box::new(NullCellLayerSpecialty::new()),
    )
}

fn create_control(randomness: SeededMutationRandomness) -> NeuralNetControl {
    let mut builder = NeuralNetControlBuilder::new(TransferFn::IDENTITY);

    let cell_y_input_index =
        builder.add_input_node("<center y", |cell_state| cell_state.center.y());
    builder.add_output_node(
        ">float resize",
        &[(cell_y_input_index, -1.0)],
        -100.0,
        |value| CellLayer::resize_request(FLOAT_LAYER_INDEX, AreaDelta::new(value)),
    );

    builder.build(randomness)
}
