use crate::physics::handles::*;
use crate::physics::quantities::*;
use crate::physics::shapes::Circle;

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
