use physics::quantities::*;

pub trait Body {
    fn position(&self) -> Position;
    fn velocity(&self) -> Velocity;
    // fn add_force(&mut self, force: Force);
    // fn clear_forces(&mut self);
    fn move_for(&mut self, duration: Duration);
    fn kick(&mut self, impulse: Impulse);
}

#[derive(Debug, PartialEq)]
pub struct State {
    pub mass: Mass,
    pub position: Position,
    pub velocity: Velocity,
}

impl State {
    pub fn new(mass: Mass, position: Position, velocity: Velocity) -> State {
        State { mass, position, velocity }
    }
}

impl Body for State {
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

#[derive(Debug, PartialEq)]
pub struct Forces {
    net_force: Force,
}

impl Forces {
    pub fn new(initial_x: f64, initial_y: f64) -> Forces {
        Forces { net_force: Force::new(initial_x, initial_y) }
    }

    pub fn add_force(&mut self, f: Force) {
        self.net_force = self.net_force.plus(f);
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

    #[test]
    fn coasting() {
        let mut subject = SimpleBody::new(Mass::new(2.0), Position::new(-1.0, 1.5), Velocity::new(1.0, 2.0));
        subject.move_for(Duration::new(0.5));
        assert_eq!(State::new(Mass::new(2.0), Position::new(-0.5, 2.5), Velocity::new(1.0, 2.0)), *subject.state());
    }

    #[test]
    fn kicked() {
        let mut subject = SimpleBody::new(Mass::new(2.0), Position::new(-1.0, 2.0), Velocity::new(1.0, -1.0));
        subject.kick(Impulse::new(0.5, 0.5));
        assert_eq!(State::new(Mass::new(2.0), Position::new(-1.0, 2.0), Velocity::new(1.25, -0.75)), *subject.state());
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

    struct SimpleBody {
        state: State,
    }

    impl SimpleBody {
        fn new(mass: Mass, position: Position, velocity: Velocity) -> SimpleBody {
            SimpleBody {
                state: State::new(mass, position, velocity)
            }
        }

        fn state(&self) -> &State {
            &self.state
        }
    }

    impl Body for SimpleBody {
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
