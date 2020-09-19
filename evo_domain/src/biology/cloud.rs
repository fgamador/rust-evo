use crate::physics::handles::*;
use crate::physics::quantities::*;
use crate::physics::shapes::Circle;

#[derive(Debug, Clone, Copy)]
pub struct CloudParameters {
    pub resize_factor: Value1D,
}

impl CloudParameters {
    pub const DEFAULT: CloudParameters = CloudParameters { resize_factor: 1.0 };

    pub fn validate(&self) {
        assert!(self.resize_factor > 0.0);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cloud {
    handle: Handle<Cloud>,
    position: Position,
    radius: Length,
}

impl Cloud {
    pub fn new(position: Position, radius: Length) -> Self {
        Cloud {
            handle: Handle::unset(),
            position,
            radius,
        }
    }

    pub fn tick(&mut self, parameters: &CloudParameters) {
        self.radius *= parameters.resize_factor;
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
            resize_factor: 1.25,
        };
        let mut cloud = Cloud::new(Position::ORIGIN, Length::new(2.0));

        cloud.tick(&parameters);

        assert_eq!(cloud.radius(), Length::new(2.5));
    }
}
