pub mod events;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Event {
    Rendered,
    Updated,
}

pub struct ViewModel {
    pub updated: bool,
}

impl ViewModel {
    pub fn update(&mut self) {
        self.updated = true;
        println!("Updated");
    }

    pub fn render(&mut self) {
        self.updated = false;
        println!("Rendered");
    }
}
