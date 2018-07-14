use physics::ball::*;
use physics::newtonian::Body;
use physics::quantities::*;
use physics::walls::*;

#[derive(Debug)]
pub struct World {
    balls: Vec<Ball>,
}

impl World {
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        World { balls: vec![] }
    }

    pub fn add_ball(&mut self, ball: Ball) {
        self.balls.push(ball);
    }

    pub fn balls(&self) -> &[Ball] {
        &self.balls
    }

    pub fn tick(&mut self) {
        for ball in &mut self.balls {
            ball.move_for(Duration::new(1.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_moves_balls() {
        let mut world = World::new(Position::new(0.0, -5.0), Position::new(10.0, 1.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(5.0, -2.0), Velocity::new(1.0, 1.0)));
        world.tick();
        let ball = &world.balls()[0];
        assert_eq!(Position::new(6.0, -1.0), ball.position());
    }

//    #[test]
//    fn wall_bounce() {
//        let mut world = World::new(Position::new(0.0, -5.0), Position::new(10.0, 1.0));
//        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
//                                 Position::new(8.0, -1.0), Velocity::new(1.0, 1.0)));
//        for i in 0..3 {
//            world.tick();
//        }
//    }
}
