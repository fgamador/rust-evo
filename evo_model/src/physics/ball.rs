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
    fn new(radius: Length, mass: Mass, position: Position, velocity: Velocity) -> Ball {
        Ball {
            radius,
            state: newtonian::State::new(mass, position, velocity),
            environment: BallEnvironment::new(),
        }
    }

    fn environment(&mut self) -> &mut BallEnvironment {
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
    fn new() -> Self {
        BallEnvironment { overlaps: vec![] }
    }

    fn add_overlap(&mut self, overlap: Overlap) {
        self.overlaps.push(overlap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::string::String;

    #[test]
    fn foobar() {
        let mut map: HashMap<String, String> = HashMap::new();
        let key = "key".to_string();
        let val = "val".to_string();
        map.insert(key.clone(), val);
        let val2 = get_mut(&mut map, &key);
        *val2 = "val2".to_string();
    }

    fn get_mut<'a>(map: &'a mut HashMap<String, String>, key: &String) -> &'a mut String {
        map.get_mut(key).unwrap()
    }

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
        let ball = Ball::new(Length::new(1.0), Mass::new(2.0),
                             Position::new(-9.5, 1.75), Velocity::new(1.0, 2.0));
        let mut balls = vec![ball];
        let mut overlaps = walls.find_overlaps(&mut balls);
        assert_eq!(1, overlaps.len());
        let (overlapped, overlap) = overlaps.pop().unwrap();
        assert_eq!(Overlap::new(Displacement::new(0.5, -0.75)), overlap);
        overlapped.environment().add_overlap(overlap);
    }
}
