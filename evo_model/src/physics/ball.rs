use physics::newtonian;
use physics::quantities::*;
use physics::walls::Walls;

struct Ball {
    state: newtonian::State,
}

impl Ball {
    fn new(mass: Mass, position: Position, velocity: Velocity) -> Ball {
        Ball {
            state: newtonian::State::new(mass, position, velocity)
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
    fn wall_corner_overlap_adds_force() {
        let subject = Ball::new(Mass::new(2.0), Position::new(-9.5, 1.75), Velocity::new(1.0, 2.0));
        let walls = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
//        let circles = vec![Circle::new(Position::new(-9.5, -4.25), Length::new(1.0))];
//        let overlaps = subject.find_overlaps(&circles);
//        assert_eq!(1, overlaps.len());
//        assert_eq!(Overlap::new(&circles[0], Displacement::new(0.5, 0.25)), overlaps[0]);
    }
}
