use std::ops::Mul;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Length {
    value: f64,
}

impl Length {
    pub fn new(value: f64) -> Self {
        if value < 0.0 {
            panic!("Negative length: {}", value);
        }

        Length { value }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Mul for Length {
    type Output = Area;

    fn mul(self, rhs: Self) -> Self::Output {
        Area::new(self.value * rhs.value)
    }
}

impl Mul<f64> for Length {
    type Output = Length;

    fn mul(self, rhs: f64) -> Self::Output {
        Length::new(self.value * rhs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area {
    value: f64,
}

impl Area {
    pub fn new(value: f64) -> Self {
        if value < 0.0 {
            panic!("Negative area: {}", value);
        }

        Area { value }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Mul<f64> for Area {
    type Output = Area;

    fn mul(self, rhs: f64) -> Self::Output {
        Area::new(self.value * rhs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    x: f64,
    y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[allow(dead_code)]
    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn plus(&self, d: Displacement) -> Position {
        Position::new(self.x + d.x, self.y + d.y)
    }

    pub fn minus(&self, pos: Position) -> Displacement {
        Displacement::new(self.x - pos.x, self.y - pos.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Displacement {
    x: f64,
    y: f64,
}

impl Displacement {
    pub fn new(x: f64, y: f64) -> Self {
        Displacement { x, y }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[allow(dead_code)]
    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn plus(&self, d: Displacement) -> Displacement {
        Displacement::new(self.x + d.x, self.y + d.y)
    }

    pub fn max(&self, d: Displacement) -> Displacement {
        Displacement::new(self.x.max(d.x), self.y.max(d.y))
    }

    pub fn min(&self, d: Displacement) -> Displacement {
        Displacement::new(self.x.min(d.x), self.y.min(d.y))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Duration {
    value: f64,
}

impl Duration {
    pub fn new(value: f64) -> Self {
        Duration { value }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> f64 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity {
    x: f64,
    y: f64,
}

impl Velocity {
    pub fn new(x: f64, y: f64) -> Self {
        Velocity { x, y }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[allow(dead_code)]
    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn plus(&self, dv: DeltaV) -> Velocity {
        Velocity::new(self.x + dv.x, self.y + dv.y)
    }

    pub fn to_displacement(&self, duration: Duration) -> Displacement {
        Displacement::new(self.x * duration.value, self.y * duration.value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeltaV {
    x: f64,
    y: f64,
}

impl DeltaV {
    pub fn new(x: f64, y: f64) -> Self {
        DeltaV { x, y }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[allow(dead_code)]
    pub fn y(&self) -> f64 {
        self.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Impulse {
    x: f64,
    y: f64,
}

impl Impulse {
    pub fn new(x: f64, y: f64) -> Self {
        Impulse { x, y }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[allow(dead_code)]
    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn to_delta_v(&self, mass: Mass) -> DeltaV {
        DeltaV::new(self.x / mass.value, self.y / mass.value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mass {
    value: f64,
}

impl Mass {
    pub fn new(value: f64) -> Self {
        Mass { value }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> f64 {
        self.value
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Force {
    x: f64,
    y: f64,
}

impl Force {
    pub fn new(x: f64, y: f64) -> Self {
        Force { x, y }
    }

    #[allow(dead_code)]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[allow(dead_code)]
    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn plus(&self, f: Force) -> Force {
        Force::new(self.x + f.x, self.y + f.y)
    }

    pub fn to_impulse(&self, d: Duration) -> Impulse {
        Impulse::new(self.x * d.value, self.y * d.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn negative_length() {
        Length::new(-1.0);
    }

    #[test]
    #[should_panic]
    fn negative_area() {
        Area::new(-1.0);
    }

    #[test]
    fn multiply_lengths() {
        let subject = Length::new(2.0);
        assert_eq!(Area::new(3.0), subject * Length::new(1.5));
    }

    #[test]
    fn multiply_length_by_scalar() {
        let subject = Length::new(2.0);
        assert_eq!(Length::new(3.0), subject * 1.5);
    }

    #[test]
    fn multiply_area_by_scalar() {
        let subject = Area::new(2.0);
        assert_eq!(Area::new(3.0), subject * 1.5);
    }

    #[test]
    fn displace_position() {
        let subject = Position::new(1.5, 1.5);
        assert_eq!(Position::new(2.0, 1.0), subject.plus(Displacement::new(0.5, -0.5)));
    }

    #[test]
    fn subtract_positions() {
        let subject = Position::new(2.0, 1.0);
        assert_eq!(Displacement::new(0.5, -0.5), subject.minus(Position::new(1.5, 1.5)));
    }

    #[test]
    fn add_displacements() {
        let subject = Displacement::new(1.5, 1.5);
        assert_eq!(Displacement::new(2.0, 1.0), subject.plus(Displacement::new(0.5, -0.5)));
    }

    #[test]
    fn displacement_max() {
        let subject = Displacement::new(1.5, -0.5);
        assert_eq!(Displacement::new(1.5, -0.25), subject.max(Displacement::new(0.5, -0.25)));
    }

    #[test]
    fn displacement_min() {
        let subject = Displacement::new(1.5, -0.25);
        assert_eq!(Displacement::new(0.5, -0.5), subject.min(Displacement::new(0.5, -0.5)));
    }

    #[test]
    fn change_velocity() {
        let subject = Velocity::new(1.5, 1.5);
        assert_eq!(Velocity::new(1.0, 2.0), subject.plus(DeltaV::new(-0.5, 0.5)));
    }

    #[test]
    fn velocity_to_displacement() {
        let subject = Velocity::new(1.5, -0.5);
        assert_eq!(Displacement::new(0.75, -0.25), subject.to_displacement(Duration::new(0.5)));
    }

    #[test]
    fn impulse_to_delta_v() {
        let subject = Impulse::new(1.5, -0.5);
        assert_eq!(DeltaV::new(0.75, -0.25), subject.to_delta_v(Mass::new(2.0)));
    }

    #[test]
    fn force_to_impulse() {
        let subject = Force::new(1.5, -0.5);
        assert_eq!(Impulse::new(0.75, -0.25), subject.to_impulse(Duration::new(0.5)));
    }

    #[test]
    fn add_forces() {
        let subject = Force::new(1.5, -0.5);
        assert_eq!(Force::new(0.75, -0.25), subject.plus(Force::new(-0.75, 0.25)));
    }
}
