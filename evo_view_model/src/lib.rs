pub mod events;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Event {
    Rendered,
    Updated,
}

pub struct ViewModel {
    pub updated: bool,
}
