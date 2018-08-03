use physics::bond::*;

#[derive(Debug)]
pub struct SortableGraph<B> {
    pub balls: Vec<B>,
    pub indexes: Vec<usize>,
    bonds: Vec<Bond>,
}

impl<B> SortableGraph<B> {
    pub fn new() -> Self {
        SortableGraph {
            balls: vec![],
            indexes: vec![],
            bonds: vec![],
        }
    }

    pub fn add_ball(&mut self, ball: B) {
        self.indexes.push(self.balls.len());
        self.balls.push(ball);
    }

    pub fn add_bond(&mut self, bond: Bond) {
        self.bonds.push(bond);
    }

    pub fn balls(&self) -> &[B] {
        &self.balls
    }

    pub fn balls_mut(&mut self) -> &mut [B] {
        &mut self.balls
    }
}
