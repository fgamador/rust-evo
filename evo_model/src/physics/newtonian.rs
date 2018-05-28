pub trait Newtonian {
    fn x(&self) -> f64;
    //    fn add_force(&self, fx: f64);
    fn step(&mut self);
}

pub struct NewtonianState {
    pub x: f64,
//    pub vx: f64,
//    pub mass: f64,
}

impl Newtonian for NewtonianState {
    fn x(&self) -> f64 {
        self.x
    }

    fn step(&mut self) {
        // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleNewtonian {
        state: NewtonianState,
    }

    impl Newtonian for SimpleNewtonian {
        fn x(&self) -> f64 {
            self.state.x()
        }

        fn step(&mut self) {
            self.state.step();
        }
    }

    #[test]
    fn no_forces_no_acceleration() {
        let mut body = SimpleNewtonian { state: NewtonianState { x: 0.0 } };
        body.step();
        assert_eq!(0.0, body.x());
    }
}
