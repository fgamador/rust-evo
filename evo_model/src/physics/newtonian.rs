use physics::quantities::*;

pub trait Newtonian {
    fn position(&self) -> Position;
    fn velocity(&self) -> Velocity;
    // fn add_force(&mut self, force: Force);
    // fn clear_forces(&mut self);
    fn move_for(&mut self, duration: Duration);
    fn kick(&mut self, impulse: Impulse);
}

#[derive(Debug, PartialEq)]
pub struct NewtonianState {
    pub position: Position,
    pub velocity: Velocity,
    pub mass: Mass,
}

impl NewtonianState {
    fn new(position: Position, velocity: Velocity, mass: Mass) -> NewtonianState {
        NewtonianState { position, velocity, mass }
    }
}

impl Newtonian for NewtonianState {
    fn position(&self) -> Position {
        self.position
    }

    fn velocity(&self) -> Velocity {
        self.velocity
    }

    fn move_for(&mut self, duration: Duration) {
        self.position = self.position.plus(self.velocity.to_displacement(duration));
    }

    fn kick(&mut self, impulse: Impulse) {
        self.velocity = self.velocity.plus(impulse.to_delta_v(self.mass));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coasting() {
        let mut subject = SimpleNewtonian::new(Position::new(-1.0, 0.0), Velocity::new(1.0, 0.0), Mass::new(2.0));
        subject.move_for(Duration::new(0.5));
        assert_eq!(NewtonianState::new(Position::new(-0.5, 0.0), Velocity::new(1.0, 0.0), Mass::new(2.0)), *subject.state());
    }

    #[test]
    fn kicked() {
        let mut subject = SimpleNewtonian::new(Position::new(-1.0, 0.0), Velocity::new(1.0, 0.0), Mass::new(2.0));
        subject.kick(Impulse::new(0.5));
        assert_eq!(NewtonianState::new(Position::new(-1.0, 0.0), Velocity::new(1.25, 0.0), Mass::new(2.0)), *subject.state());
    }

    struct SimpleNewtonian {
        state: NewtonianState,
    }

    impl SimpleNewtonian {
        fn new(position: Position, velocity: Velocity, mass: Mass) -> SimpleNewtonian {
            SimpleNewtonian {
                state: NewtonianState::new(position, velocity, mass)
            }
        }

        fn state(&self) -> &NewtonianState {
            &self.state
        }
    }

    impl Newtonian for SimpleNewtonian {
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
