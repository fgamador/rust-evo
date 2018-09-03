use environment::environment::*;
use physics::newtonian::*;
use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;
use std::ptr;

#[derive(Clone, Debug)]
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

impl NewtonianBody for Ball {
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

impl GraphNode for Ball {
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

impl WithLocalEnvironment for Ball {
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
    use physics::overlap::*;

    #[test]
    fn balls_use_pointer_equality() {
        let ball1 = Ball::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        let ball2 = Ball::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        assert_eq!(ball1, ball1);
        assert_ne!(ball1, ball2);
    }

    #[test]
    fn clear_ball_environment() {
        let mut ball = Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        ball.environment_mut().add_overlap(Overlap::new(Displacement::new(1.0, 1.0)));
        ball.environment_mut().clear();
        assert!(ball.environment().overlaps().is_empty());
    }
}
