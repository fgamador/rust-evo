use physics::ball::*;
use physics::quantities::*;
//use physics::shapes::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bond {
    ball1: BallId,
    ball2: BallId,
}

impl Bond {
    pub fn new(ball1: &Ball, ball2: &Ball) -> Self {
        Bond { ball1: ball1.id(), ball2: ball2.id() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_bond() {
        let ball1 = Ball::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        let ball2 = Ball::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        let _bond = Bond::new(&ball1, &ball2);
        // TODO
    }
}
