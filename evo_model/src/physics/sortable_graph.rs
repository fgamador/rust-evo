use physics::bond::*;

#[derive(Debug)]
pub struct SortableGraph<T> {
    pub balls: Vec<T>,
    pub indexes: Vec<usize>,
    bonds: Vec<Bond>,
}

impl<T> SortableGraph<T> {
    pub fn new() -> Self {
        SortableGraph {
            balls: vec![],
            indexes: vec![],
            bonds: vec![],
        }
    }

    pub fn add_ball(&mut self, ball: T) {
        self.indexes.push(self.balls.len());
        self.balls.push(ball);
    }

    pub fn add_bond(&mut self, bond: Bond) {
        self.bonds.push(bond);
    }

    pub fn balls(&self) -> &[T] {
        &self.balls
    }

    pub fn balls_mut(&mut self) -> &mut [T] {
        &mut self.balls
    }
}
