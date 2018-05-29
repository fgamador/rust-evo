pub struct Position {
    x: f64,
}

pub struct Velocity {
    x: f64,
}

impl Position {
    pub fn new(x: f64) -> Position {
        Position { x }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn plus(&self, v: &Velocity) -> Position {
        Position::new(self.x + v.x)
    }
}

impl Velocity {
    pub fn new(x: f64) -> Velocity {
        Velocity { x }
    }

    pub fn x(&self) -> f64 {
        self.x
    }
}
