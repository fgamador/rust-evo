use physics::state_vars::*;

pub trait Newtonian {
    fn position(&self) -> &Position;
    fn vx(&self) -> f64;
    //    fn add_force(&self, fx: f64);
    fn step(&mut self);
}

pub struct NewtonianState {
    pub position: Position,
    pub velocity: Velocity,
//    pub mass: f64,
}

impl NewtonianState {
    fn new(x: f64, vx: f64) -> NewtonianState {
        NewtonianState { position: Position::new(x), velocity: Velocity::new(vx) }
    }
}

impl Newtonian for NewtonianState {
    fn position(&self) -> &Position {
        &self.position
    }

    fn vx(&self) -> f64 {
        self.velocity.x()
    }

    fn step(&mut self) {
        self.position = self.position.plus(&self.velocity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stationary() {
        let mut subject = SimpleNewtonian::new(0.0, 0.0);
        subject.step();
        assert_eq!(0.0, subject.position().x());
        assert_eq!(0.0, subject.vx());
    }

    #[test]
    fn coasting() {
        let mut subject = SimpleNewtonian::new(0.0, 1.0);
        subject.step();
        assert_eq!(1.0, subject.position().x());
        assert_eq!(1.0, subject.vx());
    }

    struct SimpleNewtonian {
        state: NewtonianState,
    }

    impl SimpleNewtonian {
        fn new(x: f64, vx: f64) -> SimpleNewtonian {
            SimpleNewtonian {
                state: NewtonianState::new(x, vx)
            }
        }
    }

    impl Newtonian for SimpleNewtonian {
        fn position(&self) -> &Position {
            self.state.position()
        }

        fn vx(&self) -> f64 {
            self.state.vx()
        }

        fn step(&mut self) {
            self.state.step();
        }
    }
}
