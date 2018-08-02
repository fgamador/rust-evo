use physics::ball::*;
use physics::bond::*;

#[derive(Debug)]
pub struct BallGraph {
    pub balls: Vec<Ball>,
    pub indexes: Vec<usize>,
    bonds: Vec<Bond>,
}

impl BallGraph {
    pub fn new() -> Self {
        BallGraph {
            balls: vec![],
            indexes: vec![],
            bonds: vec![],
        }
    }

    pub fn add_ball(&mut self, ball: Ball) {
        self.indexes.push(self.balls.len());
        self.balls.push(ball);
    }

    pub fn add_bond(&mut self, bond: Bond) {
        self.bonds.push(bond);
    }

    pub fn balls(&self) -> &[Ball] {
        &self.balls
    }

    pub fn balls_mut(&mut self) -> &mut [Ball] {
        &mut self.balls
    }
}
