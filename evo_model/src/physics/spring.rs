use crate::physics::quantities::*;
use std::fmt::Debug;

pub trait Spring: Debug {
    fn to_force(&self, compression: Displacement) -> Force;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearSpring {
    spring_constant: f64,
}

impl LinearSpring {
    pub fn new(spring_constant: f64) -> Self {
        LinearSpring { spring_constant }
    }
}

impl Spring for LinearSpring {
    fn to_force(&self, compression: Displacement) -> Force {
        Force::new(
            compression.x() * self.spring_constant,
            compression.y() * self.spring_constant,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_to_force() {
        let spring = LinearSpring::new(2.0);
        assert_eq!(
            Force::new(2.0, -3.0),
            spring.to_force(Displacement::new(1.0, -1.5))
        );
    }
}
