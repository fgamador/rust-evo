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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Impulse {
    x: f64,
}

impl Position {
    pub fn new(x: f64) -> Position {
        Position { x }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }
}

impl Duration {
    pub fn new(value: f64) -> Duration {
        Duration { value }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Velocity {
    pub fn new(x: f64) -> Velocity {
        Velocity { x }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn to_displacement(&self, duration: Duration) -> Displacement {
        Displacement::new(self.x * duration.value)
    }
}

impl Impulse {
    pub fn new(x: f64) -> Impulse {
        Impulse { x }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displaced_position() {
        let subject = Position::new(1.5);
        assert_eq!(Position::new(2.0), subject.plus(Displacement::new(0.5)));
    }

    #[test]
    fn velocity_to_displacement() {
        let subject = Velocity::new(1.5);
        assert_eq!(Displacement::new(0.75), subject.to_displacement(Duration::new(0.5)));
    }
}
