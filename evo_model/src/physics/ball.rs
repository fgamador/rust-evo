use physics::newtonian;
use physics::newtonian::Body;
use physics::newtonian::Forces;
use physics::quantities::*;
use physics::shapes::*;
use physics::overlap::*;
use std::ptr;

#[derive(Debug)]
pub struct Ball {
    radius: Length,
    state: newtonian::State,
    environment: BallEnvironment,
    forces: Forces,
}

impl Ball {
    pub fn new(radius: Length, mass: Mass, position: Position, velocity: Velocity) -> Ball {
        Ball {
            radius,
            state: newtonian::State::new(mass, position, velocity),
            environment: BallEnvironment::new(),
            forces: Forces::new(0.0, 0.0),
        }
    }

    pub fn id(&self) -> BallId {
        BallId { value: 0 }
    }

    pub fn environment(&self) -> &BallEnvironment {
        &self.environment
    }

    pub fn mut_environment(&mut self) -> &mut BallEnvironment {
        &mut self.environment
    }

    pub fn forces(&self) -> &Forces {
        &self.forces
    }

    pub fn mut_forces(&mut self) -> &mut Forces {
        &mut self.forces
    }

    pub fn add_overlap_forces(&mut self) {
        for overlap in self.environment.overlaps() {
            self.forces.add_force(overlap.to_force());
        }
    }

    pub fn exert_forces(&mut self, duration: Duration) {
        let impulse = self.forces.net_force() * duration;
        self.kick(impulse);
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallId {
    value: u64,
}

impl BallId {
    pub fn new(value: u64) -> Self {
        BallId { value }
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

    pub fn overlaps(&self) -> &Vec<Overlap> {
        &self.overlaps
    }

    pub fn clear(&mut self) {
        self.overlaps.clear();
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
    fn clear_ball_environment() {
        let mut ball = Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        ball.mut_environment().add_overlap(Overlap::new(Displacement::new(1.0, 1.0)));
        ball.mut_environment().clear();
        assert!(ball.environment().overlaps().is_empty());
    }

    #[test]
    fn add_overlap_forces() {
        let mut ball = Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        ball.mut_environment().add_overlap(Overlap::new(Displacement::new(1.0, 1.0)));
        ball.add_overlap_forces();
        assert_eq!(Force::new(1.0, 1.0), ball.forces().net_force());
    }

    #[test]
    fn exert_forces() {
        let mut ball = Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        ball.mut_forces().add_force(Force::new(1.0, 1.0));
        ball.exert_forces(Duration::new(1.0));
        assert_eq!(Velocity::new(2.0, 2.0), ball.velocity());
    }

    #[test]
    fn walls_add_overlap() {
        let walls = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let mut balls = vec![Ball::new(Length::new(1.0), Mass::new(2.0),
                                       Position::new(-9.5, 1.75), Velocity::new(1.0, 2.0))];
        walls.find_overlaps(&mut balls, |ball, overlap| {
            ball.mut_environment().add_overlap(overlap);
        });
        assert_eq!(1, balls[0].environment().overlaps().len());
        assert_eq!(Overlap::new(Displacement::new(0.5, -0.75)), balls[0].environment().overlaps()[0]);
    }
}
