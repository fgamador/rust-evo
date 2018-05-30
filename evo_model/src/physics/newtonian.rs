use physics::quantities::*;

pub trait Newtonian {
    fn position(&self) -> Position;
    fn velocity(&self) -> Velocity;
    //    fn add_force(&self, fx: f64);
    fn move_for(&mut self, duration: Duration);
}

pub struct NewtonianState {
    pub position: Position,
    pub velocity: Velocity,
//    pub mass: f64,
}

impl NewtonianState {
    fn new(position: Position, velocity: Velocity) -> NewtonianState {
        NewtonianState { position, velocity }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stationary() {
        let mut subject = SimpleNewtonian::new(0.0, 0.0);
        subject.move_for(Duration::new(1.0));
        assert_eq!(0.0, subject.position().x());
        assert_eq!(0.0, subject.velocity().x());
    }

    #[test]
    fn coasting() {
        let mut subject = SimpleNewtonian::new(0.0, 1.0);
        subject.move_for(Duration::new(1.0));
        assert_eq!(1.0, subject.position().x());
        assert_eq!(1.0, subject.velocity().x());
    }

    #[test]
    fn coasting_for_non_unit_duration() {
        let mut subject = SimpleNewtonian::new(0.0, 1.0);
        subject.move_for(Duration::new(0.5));
        assert_eq!(0.5, subject.position().x());
        assert_eq!(1.0, subject.velocity().x());
    }

    struct SimpleNewtonian {
        state: NewtonianState,
    }

    impl SimpleNewtonian {
        fn new(x: f64, vx: f64) -> SimpleNewtonian {
            SimpleNewtonian {
                state: NewtonianState::new(Position::new(x), Velocity::new(vx))
            }
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
    }
}
