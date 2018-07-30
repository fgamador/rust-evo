use physics::ball::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bond {
    ball1_id: BallId,
    ball2_id: BallId,
}

impl Bond {
    pub fn new(ball1: &Ball, ball2: &Ball) -> Self {
        Bond { ball1_id: ball1.id(), ball2_id: ball2.id() }
    }

    pub fn ball1<'a>(&self, balls: &'a [Ball]) -> &'a Ball {
        self.ball1_id.ball(balls)
    }

    pub fn ball2<'a>(&self, balls: &'a [Ball]) -> &'a Ball {
        self.ball2_id.ball(balls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics::quantities::*;

    #[test]
    fn create_bond() {
        let mut ball1 = Ball::new(Length::new(1.0), Mass::new(1.0),
                                  Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        ball1.set_id(BallId::new(0));
        let mut ball2 = Ball::new(Length::new(1.0), Mass::new(1.0),
                                  Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));
        ball2.set_id(BallId::new(1));
        let balls = vec![ball1, ball2];

        let bond = Bond::new(&balls[0], &balls[1]);

        assert_eq!(BallId::new(0), bond.ball1(&balls).id());
        assert_eq!(BallId::new(1), bond.ball2(&balls).id());
    }
}
