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
    init_and_run(create_world);
}

fn create_world(seed: u64) -> World {
    World::new(Position::new(-200.0, -400.0), Position::new(200.0, 0.0))
        .with_standard_influences()
        .with_per_cell_influences(vec![
            Box::new(SimpleForceInfluence::new(Box::new(WeightForce::new(-0.05)))),
            Box::new(SimpleForceInfluence::new(Box::new(BuoyancyForce::new(
                -0.03, 0.001,
            )))),
            Box::new(SimpleForceInfluence::new(Box::new(DragForce::new(0.005)))),
        ])
        .with_cells(vec![Cell::new(
            Position::new(0.0, -100.0),
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
            seed,
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

    let center_y_input_index =
        builder.add_input_node("<center y", |cell_state| cell_state.center.y());
    let y_velocity_input_index =
        builder.add_input_node("<y velocity", |cell_state| cell_state.velocity.y());
    let y_force_input_index =
        builder.add_input_node("<y force", |cell_state| cell_state.net_force.y());

    let desired_y_velocity_index = builder.add_hidden_node(
        "desired y velocity",
        &[(center_y_input_index, -1.0)],
        -100.0,
    );
    let y_velocity_delta_index = builder.add_hidden_node(
        "y velocity delta",
        &[
            (desired_y_velocity_index, 1.0),
            (y_velocity_input_index, -1.0),
        ],
        0.0,
    );
    let y_force_delta_index = builder.add_hidden_node(
        "y force delta",
        &[(y_velocity_delta_index, 1.0), (y_force_input_index, -1.0)],
        0.0,
    );

    builder.add_output_node(
        ">float resize",
        &[(y_force_delta_index, 1.0)],
        0.0,
        |value| CellLayer::resize_request(FLOAT_LAYER_INDEX, AreaDelta::new(value)),
    );

    builder.build(randomness)
}
