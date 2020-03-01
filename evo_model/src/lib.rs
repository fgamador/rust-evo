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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserAction {
    DebugPrint,
    Exit,
    SingleTick,
    PlayToggle,
}
