use physics::ball::*;
use physics::quantities::*;
use physics::walls::*;

#[derive(Debug)]
pub struct World {}

impl World {
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        World {}
    }

    pub fn add_ball(&mut self, ball: Ball) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_bounce() {
        let mut world = World::new(Position::new(0.0, -5.0), Position::new(10.0, 1.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.0, 1.0), Velocity::new(1.0, 1.0)));
    }
}
