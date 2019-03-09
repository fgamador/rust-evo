extern crate evo_view_model;
#[macro_use]
extern crate evo_model_derive;

pub mod biology;
pub mod environment;
pub mod neural;
pub mod physics;
pub mod world;

use biology::cell::Cell;
use physics::quantities::*;

pub trait TickCallbacks {
    fn after_influences(&mut self, subtick_duration: Duration);

    fn after_movement(&mut self) -> Vec<Cell>;
}
