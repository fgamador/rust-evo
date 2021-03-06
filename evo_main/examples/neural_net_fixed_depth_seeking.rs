use evo_domain::biology::cell::Cell;
use evo_domain::biology::control::*;
use evo_domain::biology::genome::*;
use evo_domain::biology::layers::*;
use evo_domain::environment::influences::*;
use evo_domain::physics::quantities::*;
use evo_domain::world::World;
use evo_main::main_support::*;
use std::f64::consts::PI;

const GOAL_DEPTH: Value1D = -100.0;

const GRAVITY: Value1D = -0.05;
const FLUID_DENSITY: Value1D = 0.0005;
const FLOAT_LAYER_DENSITY: Value1D = 0.0004;
const PHOTO_LAYER_DENSITY: Value1D = 0.00075;

const FLOAT_LAYER_INDEX: usize = 0;
const PHOTO_LAYER_INDEX: usize = 1;

fn main() {
    init_and_run(create_world);
}

fn create_world(seed: u64) -> World {
    World::new(Position::new(-200.0, -400.0), Position::new(200.0, 0.0))
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
        .with_cells(vec![Cell::new(
            Position::new(0.0, -200.0),
            Velocity::new(0.0, 0.0),
            vec![
                simple_cell_layer(
                    Area::new(10.0 * PI),
                    Density::new(FLOAT_LAYER_DENSITY),
                    Tissue::AirBubble,
                ),
                simple_cell_layer(
                    Area::new(300.0 * PI),
                    Density::new(PHOTO_LAYER_DENSITY),
                    Tissue::Photosynthetic,
                ),
            ],
        )
        .with_control(Box::new(create_control(SeededMutationRandomness::new(
            seed,
            &MutationParameters::NO_MUTATION,
        ))))])
}

fn simple_cell_layer(area: Area, density: Density, tissue: Tissue) -> CellLayer {
    CellLayer::new(
        area,
        density,
        tissue,
        Box::new(NullCellLayerSpecialty::new()),
    )
}

fn create_control(randomness: SeededMutationRandomness) -> NeuralNetControl {
    let mut builder = NeuralNetControlBuilder::new(TransferFn::IDENTITY);

    let center_y_input_index = builder.add_input_node("<y", |cell_state| cell_state.center.y());
    let float_layer_area_input_index = builder.add_input_node("<float area", |cell_state| {
        cell_state.layers[FLOAT_LAYER_INDEX].area.value()
    });
    let photo_layer_area_input_index = builder.add_input_node("<photo area", |cell_state| {
        cell_state.layers[PHOTO_LAYER_INDEX].area.value()
    });

    const NEUTRALLY_BUOYANT_AREA_RATIO: Value1D =
        (FLUID_DENSITY - PHOTO_LAYER_DENSITY) / (FLOAT_LAYER_DENSITY - FLUID_DENSITY);

    let desired_y_delta_index = builder.add_hidden_node(
        "desired y delta",
        &[(center_y_input_index, -1.0)],
        GOAL_DEPTH as f32,
    );

    builder.add_output_node2(
        ">float resize",
        &[
            (
                photo_layer_area_input_index,
                NEUTRALLY_BUOYANT_AREA_RATIO as f32,
            ),
            (desired_y_delta_index, 100.0),
            (float_layer_area_input_index, -1.0),
        ],
        0.0,
        &[|value| CellLayer::resize_request(FLOAT_LAYER_INDEX, AreaDelta::new(value))],
    );

    builder.build(randomness)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn zero_resize_when_at_rest_at_goal_depth() {
//         assert_eq!(
//             calc_requested_float_layer_resize(Position::new(0.0, GOAL_DEPTH), Velocity::ZERO),
//             0.0
//         );
//     }
//
//     #[test]
//     fn negative_resize_when_rising_at_goal_depth() {
//         assert!(
//             calc_requested_float_layer_resize(
//                 Position::new(0.0, GOAL_DEPTH),
//                 Velocity::new(0.0, 1.0)
//             ) < 0.0
//         );
//     }
//
//     #[test]
//     fn positive_resize_when_falling_at_goal_depth() {
//         assert!(
//             calc_requested_float_layer_resize(
//                 Position::new(0.0, GOAL_DEPTH),
//                 Velocity::new(0.0, -1.0)
//             ) > 0.0
//         );
//     }
//
//     fn calc_requested_float_layer_resize(position: Position, velocity: Velocity) -> Value1D {
//         let mut subject = create_control(SeededMutationRandomness::new(
//             0,
//             &MutationParameters::NO_MUTATION,
//         ));
//
//         let mut cell_state = CellStateSnapshot::ZEROS;
//         cell_state.center = position;
//         cell_state.velocity = velocity;
//
//         let control_requests = subject.run(&cell_state);
//         assert_eq!(control_requests.len(), 1);
//
//         let float_layer_resize_request = &control_requests[0];
//         assert_eq!(float_layer_resize_request.layer_index(), FLOAT_LAYER_INDEX);
//         assert_eq!(
//             float_layer_resize_request.channel_index(),
//             CellLayer::RESIZE_CHANNEL_INDEX
//         );
//
//         float_layer_resize_request.requested_value()
//     }
// }
