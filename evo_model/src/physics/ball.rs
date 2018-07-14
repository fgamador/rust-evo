use physics::newtonian;
use physics::quantities::*;
use physics::shapes::*;
use physics::walls::*;
use std::ptr;

#[derive(Debug)]
pub struct Ball {
    radius: Length,
    state: newtonian::State,
    environment: BallEnvironment,
}

impl Ball {
    pub fn new(radius: Length, mass: Mass, position: Position, velocity: Velocity) -> Ball {
        Ball {
            radius,
            state: newtonian::State::new(mass, position, velocity),
            environment: BallEnvironment::new(),
        }
    }

    pub fn environment(&mut self) -> &mut BallEnvironment {
        &mut self.environment
    }
}

impl PartialEq for Ball {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
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

#[derive(Debug)]
pub struct BallEnvironment {
    overlaps: Vec<Overlap>,
}

impl BallEnvironment {
    pub fn new() -> Self {
        BallEnvironment { overlaps: vec![] }
    }

    pub fn add_overlap(&mut self, overlap: Overlap) {
        self.overlaps.push(overlap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balls_use_pointer_equality() {
        let ball1 = Ball::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        let ball2 = Ball::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        assert_eq!(ball1, ball1);
        assert_ne!(ball1, ball2);
    }

    #[test]
    fn walls_add_overlap() {
        let walls = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let mut balls = vec![Ball::new(Length::new(1.0), Mass::new(2.0),
                                       Position::new(-9.5, 1.75), Velocity::new(1.0, 2.0))];
        walls.find_overlaps(&mut balls, |ball, overlap| {
            ball.environment().add_overlap(overlap);
        });
        assert_eq!(1, balls[0].environment().overlaps.len());
        assert_eq!(Overlap::new(Displacement::new(0.5, -0.75)), balls[0].environment().overlaps[0]);
    }
}
