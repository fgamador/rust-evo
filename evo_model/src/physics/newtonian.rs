pub trait Newtonian {
    fn x(&self) -> f64;
    fn vx(&self) -> f64;
    //    fn add_force(&self, fx: f64);
    fn step(&mut self);
}

//pub struct Position {
//    pub x: f64,
//    pub y: f64,
//}
//
//pub struct Velocity {
//    pub x: f64,
//    pub y: f64,
//}
//
//impl Position {
//    fn plus(&self, v: &Velocity) -> Position {
//        Position {
//            x: self.x + v.x,
//            y: self.y + v.y,
//        }
//    }
//}

pub struct NewtonianState {
    pub x: f64,
    pub vx: f64,
//    pub mass: f64,
}

impl NewtonianState {
    fn new(x: f64, vx: f64) -> NewtonianState {
        NewtonianState { x, vx }
    }
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
        let mut subject = SimpleNewtonian::new(0.0, 0.0);
        subject.step();
        assert_eq!(0.0, subject.x());
        assert_eq!(0.0, subject.vx());
    }

    #[test]
    fn coasting() {
        let mut subject = SimpleNewtonian::new(0.0, 1.0);
        subject.step();
        assert_eq!(1.0, subject.x());
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
