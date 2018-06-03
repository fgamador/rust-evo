#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    x: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Displacement {
    x: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Duration {
    value: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

    pub fn plus(&self, d: Displacement) -> Position {
        Position::new(self.x + d.x)
    }
}

impl Displacement {
    pub fn new(x: f64) -> Displacement {
        Displacement { x }
    }

    pub fn x(&self) -> f64 {
        self.x
    }
}

impl Duration {
    pub fn new(value: f64) -> Duration {
        Duration { value }
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Velocity {
    pub fn new(x: f64) -> Velocity {
        Velocity { x }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn to_displacement(&self, duration: Duration) -> Displacement {
        Displacement::new(self.x * duration.value)
    }
}
