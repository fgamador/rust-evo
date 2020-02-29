#[macro_use]
extern crate evo_model_derive;
#[macro_use]
extern crate log;

extern crate rand;
extern crate rand_pcg;
extern crate smallvec;

pub mod biology;
pub mod environment;
pub mod genome;
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
    DebugPrint,
    Exit,
    SingleTick,
    PlayToggle,
}
