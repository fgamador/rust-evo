use crate::physics::handles::*;
use crate::physics::quantities::*;
use crate::physics::shapes::Circle;

#[derive(Debug, Clone, Copy)]
pub struct CloudParameters {
    pub resize_factor: Positive,
    pub minimum_concentration: Fraction,
}

impl CloudParameters {
    pub const DEFAULT: CloudParameters = CloudParameters {
        resize_factor: Positive::unchecked(1.0),
        minimum_concentration: Fraction::ZERO,
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cloud {
    handle: Handle<Cloud>,
    position: Position,
    radius: Length,
    concentration: Fraction,
}

impl Cloud {
    pub fn new(position: Position, radius: Length) -> Self {
        Cloud {
            handle: Handle::unset(),
            position,
            radius,
            concentration: Fraction::new(1.0),
        }
    }

    pub fn tick(&mut self, parameters: &CloudParameters) {
        self.radius *= parameters.resize_factor.value();
        self.concentration /= parameters.resize_factor.sqr().value();
    }

    pub fn exists(&self, parameters: &CloudParameters) -> bool {
        self.concentration >= parameters.minimum_concentration
    }

    pub fn concentration(&self) -> Fraction {
        self.concentration
    }
}

impl ObjectWithHandle<Cloud> for Cloud {
    fn handle(&self) -> Handle<Cloud> {
        self.handle
    }

    fn handle_mut(&mut self) -> &mut Handle<Cloud> {
        &mut self.handle
    }
}

impl Circle for Cloud {
    fn radius(&self) -> Length {
        self.radius
    }

    fn center(&self) -> Position {
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_expands_cloud() {
        let parameters = CloudParameters {
            resize_factor: Positive::new(1.25),
            ..CloudParameters::DEFAULT
        };
        let mut cloud = Cloud::new(Position::ORIGIN, Length::new(2.0));

        cloud.tick(&parameters);

        assert_eq!(cloud.radius(), Length::new(2.5));
    }

    #[test]
    fn tick_decreases_concentration() {
        let parameters = CloudParameters {
            resize_factor: Positive::new(2.0),
            ..CloudParameters::DEFAULT
        };
        let mut cloud = Cloud::new(Position::ORIGIN, Length::new(1.0));
        assert_eq!(cloud.concentration(), Fraction::ONE);

        cloud.tick(&parameters);

        assert_eq!(cloud.concentration(), Fraction::new(0.25));
    }

    #[test]
    fn cloud_disappears_below_minimum_concentration() {
        let parameters = CloudParameters {
            resize_factor: Positive::new(10.0),
            minimum_concentration: Fraction::new(0.1),
        };
        let mut cloud = Cloud::new(Position::ORIGIN, Length::new(1.0));
        assert!(cloud.exists(&parameters));

        cloud.tick(&parameters);

        assert!(!cloud.exists(&parameters));
    }
}
