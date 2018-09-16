use environment::environment::*;
use physics::newtonian::*;
use physics::quantities::*;
use physics::shapes::Circle;
use physics::sortable_graph::*;
use std::ptr;

#[derive(Clone, Debug, GraphNode, HasLocalEnvironment, NewtonianBody)]
pub struct Ball {
    graph_node_data: GraphNodeData,
    radius: Length,
    state: NewtonianState,
    environment: LocalEnvironment,
}

impl Ball {
    pub fn new(radius: Length, mass: Mass, position: Position, velocity: Velocity) -> Ball {
        Ball {
            graph_node_data: GraphNodeData::new(),
            radius,
            state: NewtonianState::new(mass, position, velocity),
            environment: LocalEnvironment::new(),
        }
    }
}

impl PartialEq for Ball {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl Circle for Ball {
    fn radius(&self) -> Length {
        self.radius
    }

    fn center(&self) -> Position {
        self.state.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balls_use_pointer_equality() {
        let ball1 = Ball::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        let ball2 = Ball::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        assert_eq!(ball1, ball1);
        assert_ne!(ball1, ball2);
    }
}
