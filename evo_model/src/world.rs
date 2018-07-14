use physics::ball::*;
use physics::quantities::*;
use physics::walls::*;

#[derive(Debug)]
pub struct World {}

impl World {
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        World {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_bounce() {
        let world = World::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
    }
}
