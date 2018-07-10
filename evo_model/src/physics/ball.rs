use physics::newtonian;
use physics::quantities::*;
use physics::shapes::*;
use physics::walls::*;
use std::ptr;

#[derive(Debug)]
pub struct Ball {
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

//#[derive(Debug)]
//pub struct BallEnvironment<'a> {
//    overlaps: Vec<&'a Overlap<'a, Circle>>,
//}
//
//impl<'a> BallEnvironment<'a> {
//    fn add_overlap(&mut self, overlap: &'a Overlap<'a, Circle>) {
//        self.overlaps.push(overlap);
//    }
//}

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
    fn wall_corner_overlap_adds_force() {
        let walls = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let subject = Ball::new(Length::new(1.0), Mass::new(2.0),
                                Position::new(-9.5, 1.75), Velocity::new(1.0, 2.0));
        let balls = vec![subject];
        let mut overlaps = walls.find_overlaps(&balls);
        assert_eq!(1, overlaps.len());
        let overlap = overlaps.pop().unwrap();
        let subject = &balls[0];
        assert_eq!(Overlap::new(subject, Displacement::new(0.5, -0.75)), overlap);
//        subject.environment().add_overlap(&overlaps[0]);
    }
}
