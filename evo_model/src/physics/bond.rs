use physics::ball::*;
//use physics::quantities::*;
//use physics::shapes::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bond
{
    ball1: BallId,
    ball2: BallId,
}

impl Bond
{
    pub fn new(ball1: BallId, ball2: BallId) -> Self {
        Bond { ball1, ball2 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let _bond = Bond::new(BallId::new(0), BallId::new(0));
        // TODO
    }
}
