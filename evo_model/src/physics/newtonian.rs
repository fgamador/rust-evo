pub trait Newtonian {
    fn x(&self) -> f64;
    fn vx(&self) -> f64;
    //    fn add_force(&self, fx: f64);
    fn step(&mut self);
}

pub struct NewtonianState {
    pub x: f64,
    pub vx: f64,
//    pub mass: f64,
}

impl Newtonian for NewtonianState {
    fn x(&self) -> f64 {
        self.x
    }

    fn vx(&self) -> f64 {
        self.vx
    }

    fn step(&mut self) {
        self.x += self.vx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stationary() {
        let mut body = SimpleNewtonian { state: NewtonianState { x: 0.0, vx: 0.0 } };
        body.step();
        assert_eq!(0.0, body.x());
        assert_eq!(0.0, body.vx());
    }

    #[test]
    fn coasting() {
        let mut body = SimpleNewtonian { state: NewtonianState { x: 0.0, vx: 1.0 } };
        body.step();
        assert_eq!(1.0, body.x());
        assert_eq!(1.0, body.vx());
    }

    struct SimpleNewtonian {
        state: NewtonianState,
    }

    impl Newtonian for SimpleNewtonian {
        fn x(&self) -> f64 {
            self.state.x()
        }

        fn vx(&self) -> f64 {
            self.state.vx()
        }

        fn step(&mut self) {
            self.state.step();
        }
    }
}
