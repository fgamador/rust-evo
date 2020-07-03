use crate::physics::quantities::*;

pub trait NewtonianBody {
    fn mass(&self) -> Mass;
    fn position(&self) -> Position;
    fn velocity(&self) -> Velocity;
    fn move_for(&mut self, duration: Duration);
    fn kick(&mut self, impulse: Impulse);
    fn forces(&self) -> &Forces;
    fn forces_mut(&mut self) -> &mut Forces;
    fn exert_forces(&mut self, duration: Duration);
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewtonianState {
    pub mass: Mass,
    pub position: Position,
    pub velocity: Velocity,
    pub forces: Forces,
}

impl NewtonianState {
    pub fn new(mass: Mass, position: Position, velocity: Velocity) -> NewtonianState {
        NewtonianState {
            mass,
            position,
            velocity,
            forces: Forces::new(0.0, 0.0),
        }
    }
}

impl NewtonianBody for NewtonianState {
    fn mass(&self) -> Mass {
        self.mass
    }

    fn position(&self) -> Position {
        self.position
    }

    fn velocity(&self) -> Velocity {
        self.velocity
    }

    fn move_for(&mut self, duration: Duration) {
        self.position = self.position + self.velocity * duration;
    }

    fn kick(&mut self, impulse: Impulse) {
        self.velocity = self.velocity + impulse / self.mass;
    }

    fn forces(&self) -> &Forces {
        &self.forces
    }

    fn forces_mut(&mut self) -> &mut Forces {
        &mut self.forces
    }

    fn exert_forces(&mut self, duration: Duration) {
        let impulse = self.forces.net_force() * duration;
        self.kick(impulse);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Forces {
    net_force: Force,
}

impl Forces {
    pub fn new(initial_x: f64, initial_y: f64) -> Forces {
        Forces {
            net_force: Force::new(initial_x, initial_y),
        }
    }

    pub fn add_force(&mut self, f: Force) {
        self.net_force += f;
    }

    pub fn clear(&mut self) {
        self.net_force = Force::new(0.0, 0.0);
    }

    pub fn net_force(&self) -> Force {
        self.net_force
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evo_model_derive::NewtonianBody;

    #[test]
    fn coasting() {
        let mut subject = SimpleBody::new(
            Mass::new(2.0),
            Position::new(-1.0, 1.5),
            Velocity::new(1.0, 2.0),
        );
        subject.move_for(Duration::new(0.5));
        assert_eq!(Position::new(-0.5, 2.5), subject.position());
        assert_eq!(Velocity::new(1.0, 2.0), subject.velocity());
    }

    #[test]
    fn kicked() {
        let mut subject = SimpleBody::new(
            Mass::new(2.0),
            Position::new(-1.0, 2.0),
            Velocity::new(1.0, -1.0),
        );
        subject.kick(Impulse::new(0.5, 0.5));
        assert_eq!(Position::new(-1.0, 2.0), subject.position());
        assert_eq!(Velocity::new(1.25, -0.75), subject.velocity());
    }

    #[test]
    fn net_force() {
        let mut subject = Forces::new(1.5, -0.5);
        subject.add_force(Force::new(0.25, -0.5));
        assert_eq!(Force::new(1.75, -1.0), subject.net_force());
    }

    #[test]
    fn clear_net_force() {
        let mut subject = Forces::new(1.5, -0.5);
        subject.clear();
        assert_eq!(Force::new(0.0, 0.0), subject.net_force());
    }

    #[test]
    fn exert_forces() {
        let mut ball = SimpleBody::new(
            Mass::new(1.0),
            Position::new(1.0, 1.0),
            Velocity::new(1.0, 1.0),
        );
        ball.state.forces.add_force(Force::new(1.0, 1.0));
        ball.exert_forces(Duration::new(1.0));
        assert_eq!(Velocity::new(2.0, 2.0), ball.velocity());
    }

    #[derive(NewtonianBody)]
    struct SimpleBody {
        state: NewtonianState,
    }

    impl SimpleBody {
        fn new(mass: Mass, position: Position, velocity: Velocity) -> SimpleBody {
            SimpleBody {
                state: NewtonianState::new(mass, position, velocity),
            }
        }
    }
}
