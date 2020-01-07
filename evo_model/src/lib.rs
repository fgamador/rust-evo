#[macro_use]
extern crate evo_model_derive;

pub mod biology;
pub mod environment;
pub mod neural;
pub mod physics;
pub mod world;

use crate::biology::cell::Cell;
use crate::physics::quantities::*;

pub trait TickCallbacks {
    fn after_influences(&mut self, subtick_duration: Duration);

    fn after_movement(&mut self) -> (bool, Vec<Cell>);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserAction {
    Exit,
    SingleTick,
    PauseOrPlay,
}
