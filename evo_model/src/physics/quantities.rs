use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Angle {
    radians: f64,
}

impl Angle {
    pub fn from_radians(radians: f64) -> Self {
        if radians < 0.0 {
            panic!("Negative angle: {}", radians);
        }

        Angle { radians }
    }

    #[allow(dead_code)]
    pub fn radians(&self) -> f64 {
        self.radians
    }
}

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
}

impl Add<Displacement> for Position {
    type Output = Position;

    fn add(self, rhs: Displacement) -> Self::Output {
        Position::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Position {
    type Output = Displacement;

    fn sub(self, rhs: Position) -> Self::Output {
        Displacement::new(self.x - rhs.x, self.y - rhs.y)
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

    pub fn max(&self, d: Displacement) -> Displacement {
        Displacement::new(self.x.max(d.x), self.y.max(d.y))
    }

    pub fn min(&self, d: Displacement) -> Displacement {
        Displacement::new(self.x.min(d.x), self.y.min(d.y))
    }
}

impl Add for Displacement {
    type Output = Displacement;

    fn add(self, rhs: Displacement) -> Self::Output {
        Displacement::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Neg for Displacement {
    type Output = Displacement;

    fn neg(self) -> Self::Output {
        Displacement::new(-self.x, -self.y)
    }
}

pub const ZERO_DISPLACEMENT: Displacement = Displacement { x: 0.0, y: 0.0 };

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

impl Div<f64> for Duration {
    type Output = Duration;

    fn div(self, rhs: f64) -> Self::Output {
        Duration::new(self.value / rhs)
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
}

impl Add<DeltaV> for Velocity {
    type Output = Velocity;

    fn add(self, rhs: DeltaV) -> Self::Output {
        Velocity::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Mul<Duration> for Velocity {
    type Output = Displacement;

    fn mul(self, rhs: Duration) -> Self::Output {
        Displacement::new(self.x * rhs.value, self.y * rhs.value)
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
}

impl Div<Mass> for Impulse {
    type Output = DeltaV;

    fn div(self, rhs: Mass) -> Self::Output {
        DeltaV::new(self.x / rhs.value, self.y / rhs.value)
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
}

impl Add for Force {
    type Output = Force;

    fn add(self, rhs: Force) -> Self::Output {
        Force::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Force {
    fn add_assign(&mut self, rhs: Force) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Mul<Duration> for Force {
    type Output = Impulse;

    fn mul(self, rhs: Duration) -> Self::Output {
        Impulse::new(self.x * rhs.value, self.y * rhs.value)
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
        assert_eq!(Area::new(3.0), Length::new(2.0) * Length::new(1.5));
    }

    #[test]
    fn multiply_length_by_scalar() {
        assert_eq!(Length::new(3.0), Length::new(2.0) * 1.5);
    }

    #[test]
    fn multiply_area_by_scalar() {
        assert_eq!(Area::new(3.0), Area::new(2.0) * 1.5);
    }

    #[test]
    fn displace_position() {
        assert_eq!(Position::new(2.0, 1.0),
                   Position::new(1.5, 1.5) + Displacement::new(0.5, -0.5));
    }

    #[test]
    fn subtract_positions() {
        assert_eq!(Displacement::new(0.5, -0.5),
                   Position::new(2.0, 1.0) - Position::new(1.5, 1.5));
    }

    #[test]
    fn add_displacements() {
        assert_eq!(Displacement::new(2.0, 1.0),
                   Displacement::new(1.5, 1.5) + Displacement::new(0.5, -0.5));
    }

    #[test]
    fn negate_displacement() {
        assert_eq!(Displacement::new(-1.0, 1.0), -Displacement::new(1.0, -1.0));
    }

    #[test]
    fn displacement_max() {
        assert_eq!(Displacement::new(1.5, -0.25),
                   Displacement::new(1.5, -0.5).max(Displacement::new(0.5, -0.25)));
    }

    #[test]
    fn displacement_min() {
        assert_eq!(Displacement::new(0.5, -0.5),
                   Displacement::new(1.5, -0.25).min(Displacement::new(0.5, -0.5)));
    }

    #[test]
    fn divide_duration_by_scalar() {
        assert_eq!(Duration::new(0.5), Duration::new(1.0) / 2.0);
    }

    #[test]
    fn change_velocity() {
        assert_eq!(Velocity::new(1.0, 2.0),
                   Velocity::new(1.5, 1.5) + DeltaV::new(-0.5, 0.5));
    }

    #[test]
    fn velocity_to_displacement() {
        assert_eq!(Displacement::new(0.75, -0.25),
                   Velocity::new(1.5, -0.5) * Duration::new(0.5));
    }

    #[test]
    fn impulse_to_delta_v() {
        assert_eq!(DeltaV::new(0.75, -0.25),
                   Impulse::new(1.5, -0.5) / Mass::new(2.0));
    }

    #[test]
    fn add_forces() {
        assert_eq!(Force::new(0.75, -0.25),
                   Force::new(1.5, -0.5) + Force::new(-0.75, 0.25));
    }

    #[test]
    fn increase_force() {
        let mut subject = Force::new(1.5, -0.5);
        subject += Force::new(-0.75, 0.25);
        assert_eq!(Force::new(0.75, -0.25), subject);
    }

    #[test]
    fn force_to_impulse() {
        assert_eq!(Impulse::new(0.75, -0.25),
                   Force::new(1.5, -0.5) * Duration::new(0.5));
    }
}
