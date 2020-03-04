use crate::biology::cell::Cell;
use crate::biology::control::CellStateSnapshot;
use crate::genome::sparse_neural_net::*;
use crate::physics::quantities::*;
use crate::physics::shapes::Circle;
use std::fmt::Debug;
use std::rc::Rc;

type CreateCellFn = fn(SparseNeuralNetGenome, SeededMutationRandomness) -> Cell;

#[derive(Debug)]
pub struct Reproduction {
    genome: Rc<SparseNeuralNetGenome>,
    randomness: SeededMutationRandomness,
    create_child: CreateCellFn,
}

impl Reproduction {
    pub fn new(
        genome: Rc<SparseNeuralNetGenome>,
        randomness: SeededMutationRandomness,
        create_child: CreateCellFn,
    ) -> Self {
        Self {
            genome,
            randomness,
            create_child,
        }
    }

    pub fn create_and_init_child(
        &mut self,
        cell_state: &CellStateSnapshot,
        budding_angle: Angle,
        donation_energy: BioEnergy,
    ) -> Cell {
        let mut child = (self.create_child)(
            // TODO test that this is called?
            self.genome.copy_with_mutation(&mut self.randomness),
            // TODO test that this is called?
            self.randomness.spawn(),
        );
        let offset = Displacement::from_polar(cell_state.radius + child.radius(), budding_angle);
        child.set_initial_position(cell_state.center + offset);
        child.set_initial_velocity(cell_state.velocity);
        child.set_initial_energy(donation_energy);
        child
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::layers::*;
    use crate::physics::newtonian::NewtonianBody;
    use std::f64::consts::PI;

    #[test]
    fn reproduction_creates_child_with_right_state() {
        let genome = SparseNeuralNetGenome::new(TransferFn::IDENTITY);
        let mut reproduction = Reproduction::new(
            Rc::new(genome),
            SeededMutationRandomness::new(0, &MutationParameters::NO_MUTATION),
            create_child,
        );
        let parent_state = CellStateSnapshot {
            radius: Length::new(2.0),
            center: Position::new(1.0, -1.0),
            velocity: Velocity::new(2.0, -2.0),
            ..CellStateSnapshot::ZEROS
        };

        let child = reproduction.create_and_init_child(
            &parent_state,
            Angle::from_radians(0.0),
            BioEnergy::new(1.0),
        );

        assert_eq!(child.layers().len(), 2);
        assert_eq!(
            child.center(),
            Position::new(
                parent_state.center.x() + parent_state.radius.value() + child.radius().value(),
                parent_state.center.y(),
            )
        );
        assert_eq!(child.velocity(), parent_state.velocity);
        assert_eq!(child.energy(), BioEnergy::new(1.0));
    }

    fn create_child(genome: SparseNeuralNetGenome, _randomness: SeededMutationRandomness) -> Cell {
        Cell::new(
            Position::ORIGIN,
            Velocity::ZERO,
            vec![
                simple_cell_layer(Area::new(PI), Density::new(1.0)),
                simple_cell_layer(Area::new(PI), Density::new(1.0)),
            ],
            Rc::new(genome),
        )
    }

    fn simple_cell_layer(area: Area, density: Density) -> CellLayer {
        CellLayer::new(
            area,
            density,
            Color::Green,
            Box::new(NullCellLayerSpecialty::new()),
        )
    }
}
