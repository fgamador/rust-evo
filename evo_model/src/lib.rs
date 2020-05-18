pub mod biology;
pub mod environment;
pub mod physics;
pub mod world;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserAction {
    DebugPrint,
    Exit,
    PlayToggle,
    SelectCellToggle { x: f64, y: f64 },
    SingleTick,
}
