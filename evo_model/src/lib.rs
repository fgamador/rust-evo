pub mod biology;
pub mod environment;
pub mod physics;
pub mod world;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserAction {
    DebugPrint,
    Exit,
    PlayToggle,
    SelectCell { x: f32, y: f32 },
    SingleTick,
}
