use physics::newtonian;
use physics::quantities::*;
use physics::shapes::*;
use physics::walls::*;

#[derive(Debug, PartialEq)]
struct Ball {
    radius: Length,
    state: newtonian::State,
}

impl Ball {
    fn new(radius: Length, mass: Mass, position: Position, velocity: Velocity) -> Ball {
        Ball {
            radius,
            state: newtonian::State::new(mass, position, velocity),
        }
    }

    fn state(&self) -> &newtonian::State {
        &self.state
    }
}

impl Circle for Ball {
    fn radius(&self) -> Length {
        self.radius
    }

    fn center(&self) -> Position {
        self.state.position
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
        let subject = Ball::new(Length::new(1.0), Mass::new(2.0), Position::new(-9.5, 1.75), Velocity::new(1.0, 2.0));
        let walls = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circles = vec![subject];
        let overlaps = walls.find_overlaps(&circles);
        assert_eq!(1, overlaps.len());
        assert_eq!(Overlap::new(&circles[0], Displacement::new(0.5, -0.75)), overlaps[0]);
        // TODO
    }
}
