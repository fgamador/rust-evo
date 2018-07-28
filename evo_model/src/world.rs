use physics::ball::*;
use physics::bond::*;
use physics::newtonian::Body;
use physics::quantities::*;
use physics::overlap::*;

#[derive(Debug)]
pub struct World {
    boxed_balls: Vec<Box<Ball>>,
    balls: Vec<Ball>,
    bonds: Vec<Bond>,
    walls: Walls,
}

impl World {
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        World {
            boxed_balls: vec![],
            balls: vec![],
            bonds: vec![],
            walls: Walls::new(min_corner, max_corner),
        }
    }

    pub fn add_ball(&mut self, ball: Ball) {
        self.balls.push(ball.clone());
        //self.boxed_balls.push(Box::new(ball));
    }

    pub fn add_bond(&mut self, ball1: BallId, ball2: BallId) {
        self.bonds.push(Bond::new(ball1, ball2));
    }

    pub fn balls(&self) -> &Balls {
        &self.balls
    }

    pub fn boxed_balls(&self) -> &BoxedBalls {
        &self.boxed_balls
    }

    pub fn tick(&mut self) {
        self.walls.find_overlaps(&mut self.balls, &mut self.boxed_balls, |ball, overlap| {
            ball.mut_environment().add_overlap(overlap);
        });

        find_pair_overlaps(&mut self.balls, |ball, overlap| {
            ball.mut_environment().add_overlap(overlap);
        });

        for ball in &mut self.balls {
            ball.add_overlap_forces();
        }

        self.box_balls();

        let tick_duration = Duration::new(1.0);
        for ball in &mut self.boxed_balls {
            ball.exert_forces(tick_duration);
            ball.move_for(tick_duration);
        }

        for ball in &mut self.boxed_balls {
            ball.mut_environment().clear();
            ball.mut_forces().clear();
        }

        self.unbox_balls();
    }

    fn box_balls(&mut self) {
        self.boxed_balls.clear();
        for ball in &self.balls {
            self.boxed_balls.push(Box::new(ball.clone()));
        }
    }

    fn unbox_balls(&mut self) {
        self.balls.clear();
        for ball in &self.boxed_balls {
            self.balls.push((**ball).clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_moves_ball() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(1.0, 1.0)));
        world.tick();
        let ball = &world.boxed_balls()[0];
        assert!(ball.position().x() > 0.0);
        assert!(ball.position().y() > 0.0);
    }

    #[test]
    fn overlaps_do_not_persist() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(9.5, 9.5), Velocity::new(0.0, 0.0)));
        world.tick();
        let ball = &world.boxed_balls()[0];
        assert!(ball.environment().overlaps().is_empty());
    }

    #[test]
    fn forces_do_not_persist() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(9.5, 9.5), Velocity::new(0.0, 0.0)));
        world.tick();
        let ball = &world.boxed_balls()[0];
        assert_eq!(Force::new(0.0, 0.0), ball.forces().net_force());
    }

    #[test]
    fn ball_bounces_off_walls() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(9.5, 9.5), Velocity::new(1.0, 1.0)));

        world.tick();

        let ball = &world.boxed_balls()[0];
        assert!(ball.velocity().x() < 1.0);
        assert!(ball.velocity().y() < 1.0);
    }

    #[test]
    fn balls_bounce_off_each_other() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(1.0, 1.0)));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.4, 1.4), Velocity::new(-1.0, -1.0)));

        world.tick();

        let ball1 = &world.boxed_balls()[0];
        assert!(ball1.velocity().x() < 1.0);
        assert!(ball1.velocity().y() < 1.0);
        let ball2 = &world.boxed_balls()[1];
        assert!(ball2.velocity().x() > -1.0);
        assert!(ball2.velocity().y() > -1.0);
    }

    //#[test]
    fn _bond_pulls_balls_together() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(-1.0, -1.0)));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.5, 1.5), Velocity::new(1.0, 1.0)));
        let ball1 = world.balls()[0].id();
        let ball2 = world.balls()[1].id();
        world.add_bond(ball1, ball2);

        world.tick();

        let ball1 = &world.boxed_balls()[0];
        let ball2 = &world.boxed_balls()[1];
        assert!(ball1.velocity().x() > -1.0);
        assert!(ball1.velocity().y() > -1.0);
        assert!(ball2.velocity().x() < 1.0);
        assert!(ball2.velocity().y() < 1.0);
    }
}
