use physics::newtonian;
use physics::quantities::*;

struct Ball {
    state: newtonian::State,
}

impl Ball {
    fn new(position: Position, velocity: Velocity, mass: Mass) -> Ball {
        Ball {
            state: newtonian::State::new(position, velocity, mass)
        }
    }

    fn state(&self) -> &newtonian::State {
        &self.state
    }
}

impl newtonian::Body for Ball {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
    }
}
