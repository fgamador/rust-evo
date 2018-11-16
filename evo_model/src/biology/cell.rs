use environment::environment::*;
use physics::newtonian::*;
use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;
use std::ptr;

#[derive(Clone, Debug, GraphNode, HasLocalEnvironment, NewtonianBody)]
pub struct Cell {
    graph_node_data: GraphNodeData,
    radius: Length,
    newtonian_state: NewtonianState,
    environment: LocalEnvironment,
}

impl Cell {
    pub fn new_old(radius: Length, mass: Mass, position: Position, velocity: Velocity) -> Cell {
        Cell {
            graph_node_data: GraphNodeData::new(),
            radius,
            newtonian_state: NewtonianState::new(mass, position, velocity),
            environment: LocalEnvironment::new(),
        }
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Circle for Cell {
    fn radius(&self) -> Length {
        self.radius
    }

    fn center(&self) -> Position {
        self.newtonian_state.position
    }
}

impl Onion for Cell {
    // TODO rings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_use_pointer_equality() {
        let cell1 = Cell::new_old(Length::new(1.0), Mass::new(1.0),
                                  Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        let cell2 = Cell::new_old(Length::new(1.0), Mass::new(1.0),
                                  Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        assert_eq!(cell1, cell1);
        assert_ne!(cell1, cell2);
    }
}
