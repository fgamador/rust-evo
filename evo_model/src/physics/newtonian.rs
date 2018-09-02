use physics::quantities::*;

pub trait NewtonianBody {
    fn position(&self) -> Position;
    fn velocity(&self) -> Velocity;
    fn move_for(&mut self, duration: Duration);
    fn kick(&mut self, impulse: Impulse);
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewtonianState {
    pub mass: Mass,
    pub position: Position,
    pub velocity: Velocity,
}

impl NewtonianState {
    pub fn new(mass: Mass, position: Position, velocity: Velocity) -> NewtonianState {
        NewtonianState { mass, position, velocity }
    }
}

impl NewtonianBody for NewtonianState {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coasting() {
        let mut subject = SimpleBody::new(
            Mass::new(2.0), Position::new(-1.0, 1.5), Velocity::new(1.0, 2.0));
        subject.move_for(Duration::new(0.5));
        assert_eq!(Position::new(-0.5, 2.5), subject.position());
        assert_eq!(Velocity::new(1.0, 2.0), subject.velocity());
    }

    #[test]
    fn kicked() {
        let mut subject = SimpleBody::new(
            Mass::new(2.0), Position::new(-1.0, 2.0), Velocity::new(1.0, -1.0));
        subject.kick(Impulse::new(0.5, 0.5));
        assert_eq!(Position::new(-1.0, 2.0), subject.position());
        assert_eq!(Velocity::new(1.25, -0.75), subject.velocity());
    }

    struct SimpleBody {
        state: NewtonianState,
    }

    impl SimpleBody {
        fn new(mass: Mass, position: Position, velocity: Velocity) -> SimpleBody {
            SimpleBody {
                state: NewtonianState::new(mass, position, velocity)
            }
        }
    }

    impl NewtonianBody for SimpleBody {
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
    }
}
