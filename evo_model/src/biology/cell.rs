use environment::environment::*;
use physics::newtonian::*;
use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;
use std::ptr;

#[derive(Clone, Debug)]
pub struct Cell {
    graph_node_data: GraphNodeData,
    radius: Length,
    state: NewtonianState,
    environment: LocalEnvironment,
}

impl Cell {
    pub fn new(radius: Length, mass: Mass, position: Position, velocity: Velocity) -> Cell {
        Cell {
            graph_node_data: GraphNodeData::new(),
            radius,
            state: NewtonianState::new(mass, position, velocity),
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
        self.state.position
    }
}

impl NewtonianBody for Cell {
    fn position(&self) -> Position {
        self.state.position()
    }

    fn velocity(&self) -> Velocity {
        self.state.velocity()
    }

    fn move_for(&mut self, duration: Duration) {
        self.state.move_for(duration);
    }

    fn kick(&mut self, impulse: Impulse) {
        self.state.kick(impulse);
    }

    fn forces(&self) -> &Forces {
        self.state.forces()
    }

    fn forces_mut(&mut self) -> &mut Forces {
        self.state.forces_mut()
    }

    fn exert_forces(&mut self, duration: Duration) {
        self.state.exert_forces(duration);
    }
}

impl GraphNode for Cell {
    fn node_handle(&self) -> NodeHandle {
        self.graph_node_data.handle()
    }

    fn graph_node_data(&self) -> &GraphNodeData {
        &self.graph_node_data
    }

    fn graph_node_data_mut(&mut self) -> &mut GraphNodeData {
        &mut self.graph_node_data
    }
}

impl HasLocalEnvironment for Cell {
    fn environment(&self) -> &LocalEnvironment {
        &self.environment
    }

    fn environment_mut(&mut self) -> &mut LocalEnvironment {
        &mut self.environment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_use_pointer_equality() {
        let cell1 = Cell::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        let cell2 = Cell::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        assert_eq!(cell1, cell1);
        assert_ne!(cell1, cell2);
    }
}
