use physics::bond::*;

#[derive(Debug)]
pub struct SortableGraph<T> {
    pub nodes: Vec<T>,
    pub indexes: Vec<usize>,
    bonds: Vec<Bond>,
}

impl<T> SortableGraph<T> {
    pub fn new() -> Self {
        SortableGraph {
            nodes: vec![],
            indexes: vec![],
            bonds: vec![],
        }
    }

    pub fn add_ball(&mut self, ball: T) {
        self.indexes.push(self.nodes.len());
        self.nodes.push(ball);
    }

    pub fn add_bond(&mut self, bond: Bond) {
        self.bonds.push(bond);
    }

    pub fn balls(&self) -> &[T] {
        &self.nodes
    }

    pub fn balls_mut(&mut self) -> &mut [T] {
        &mut self.nodes
    }
}
