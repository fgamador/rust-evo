#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    x: f64,
    y: f64,
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
    y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeltaV {
    x: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Impulse {
    x: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mass {
    value: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Force {
    x: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Position {
        Position { x, y }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn plus(&self, d: Displacement) -> Position {
        Position::new(self.x + d.x, 0.0)
    }
}

impl Displacement {
    pub fn new(x: f64, y: f64) -> Displacement {
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
    pub fn new(x: f64, y: f64) -> Velocity {
        Velocity { x, y }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn plus(&self, dv: DeltaV) -> Velocity {
        Velocity::new(self.x + dv.x, 0.0)
    }

    pub fn to_displacement(&self, duration: Duration) -> Displacement {
        Displacement::new(self.x * duration.value, 0.0)
    }
}

impl DeltaV {
    pub fn new(x: f64, y: f64) -> DeltaV {
        DeltaV { x }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }
}

impl Impulse {
    pub fn new(x: f64, y: f64) -> Impulse {
        Impulse { x }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn to_delta_v(&self, mass: Mass) -> DeltaV {
        DeltaV::new(self.x / mass.value, 0.0)
    }
}

impl Mass {
    pub fn new(value: f64) -> Mass {
        Mass { value }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Force {
    pub fn new(x: f64, y: f64) -> Force {
        Force { x }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn to_impulse(&self, d: Duration) -> Impulse {
        Impulse::new(self.x * d.value, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displace_position() {
        let subject = Position::new(1.5, 0.0);
        assert_eq!(Position::new(2.0, 0.0), subject.plus(Displacement::new(0.5, 0.0)));
    }

    #[test]
    fn change_velocity() {
        let subject = Velocity::new(1.5, 0.0);
        assert_eq!(Velocity::new(1.0, 0.0), subject.plus(DeltaV::new(-0.5, 0.0)));
    }

    #[test]
    fn velocity_to_displacement() {
        let subject = Velocity::new(1.5, 0.0);
        assert_eq!(Displacement::new(0.75, 0.0), subject.to_displacement(Duration::new(0.5)));
    }

    #[test]
    fn impulse_to_delta_v() {
        let subject = Impulse::new(1.5, 0.0);
        assert_eq!(DeltaV::new(0.75, 0.0), subject.to_delta_v(Mass::new(2.0)));
    }

    #[test]
    fn force_to_impulse() {
        let subject = Force::new(1.5, 0.0);
        assert_eq!(Impulse::new(0.75, 0.0), subject.to_impulse(Duration::new(0.5)));
    }
}
