use physics::quantities::*;

pub trait Newtonian {
    fn position(&self) -> Position;
    fn velocity(&self) -> Velocity;
    // fn add_force(&mut self, force: Force);
    // fn clear_forces(&mut self);
    fn move_for(&mut self, duration: Duration);
}

pub struct NewtonianImpl {
    pub position: Position,
    pub velocity: Velocity,
//    pub mass: f64,
}

impl NewtonianImpl {
    fn new(position: Position, velocity: Velocity) -> NewtonianImpl {
        NewtonianImpl { position, velocity }
    }
}

impl Newtonian for NewtonianImpl {
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
        let mut subject = SimpleNewtonian::new(Position::new(0.0), Velocity::new(0.0));
        subject.move_for(Duration::new(1.0));
        assert_eq!(0.0, subject.position().x());
        assert_eq!(0.0, subject.velocity().x());
    }

    #[test]
    fn coasting_for_non_unit_duration() {
        let mut subject = SimpleNewtonian::new(Position::new(0.0), Velocity::new(1.0));
        subject.move_for(Duration::new(0.5));
        assert_eq!(0.5, subject.position().x());
        assert_eq!(1.0, subject.velocity().x());
    }

    struct SimpleNewtonian {
        newtonian: NewtonianImpl,
    }

    impl SimpleNewtonian {
        fn new(position: Position, velocity: Velocity) -> SimpleNewtonian {
            SimpleNewtonian {
                newtonian: NewtonianImpl::new(position, velocity)
            }
        }
    }

    impl Newtonian for SimpleNewtonian {
        fn position(&self) -> Position {
            self.newtonian.position()
        }

        fn velocity(&self) -> Velocity {
            self.newtonian.velocity()
        }

        fn move_for(&mut self, duration: Duration) {
            self.newtonian.move_for(duration);
        }
    }
}
