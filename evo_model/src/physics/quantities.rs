use std::f64::consts::PI;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Angle {
    radians: f64,
}

impl Angle {
    pub fn from_radians(radians: f64) -> Self {
        Angle { radians: Angle::normalize_radians(radians) }
    }

    fn normalize_radians(radians: f64) -> f64 {
        let mut normalized_radians = radians;
        while normalized_radians < 0.0 {
            normalized_radians += 2.0 * PI;
        }
        while normalized_radians > 2.0 * PI {
            normalized_radians -= 2.0 * PI;
        }
        normalized_radians
    }

    #[allow(dead_code)]
    pub fn radians(&self) -> f64 {
        self.radians
    }
}

impl Sub for Angle {
    type Output = Deflection;

    fn sub(self, rhs: Angle) -> Self::Output {
        Deflection::from_radians(self.radians - rhs.radians)
    }
}

impl Add<Deflection> for Angle {
    type Output = Angle;

    fn add(self, rhs: Deflection) -> Self::Output {
        Angle::from_radians(self.radians + rhs.radians)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Deflection {
    radians: f64,
}

impl Deflection {
    pub fn from_radians(radians: f64) -> Self {
        Deflection { radians }
    }

    #[allow(dead_code)]
    pub fn radians(&self) -> f64 {
        self.radians
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
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

    pub fn sqr(&self) -> Area {
        Area::new(self.value * self.value)
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

impl Mul<Length> for f64 {
    type Output = Length;

    fn mul(self, rhs: Length) -> Self::Output {
        Length::new(self * rhs.value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
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

    pub fn sqrt(&self) -> Length {
        Length::new(self.value.sqrt())
    }
}

impl Add<Area> for Area {
    type Output = Area;

    fn add(self, rhs: Area) -> Self::Output {
        Area::new(self.value + rhs.value)
    }
}

impl AddAssign for Area {
    fn add_assign(&mut self, rhs: Area) {
        self.value += rhs.value;
    }
}

impl Sub<Area> for Area {
    type Output = Area;

    fn sub(self, rhs: Area) -> Self::Output {
        Area::new(self.value - rhs.value)
    }
}

impl SubAssign for Area {
    fn sub_assign(&mut self, rhs: Area) {
        self.value -= rhs.value;
    }
}

impl Mul<f64> for Area {
    type Output = Area;

    fn mul(self, rhs: f64) -> Self::Output {
        Area::new(self.value * rhs)
    }
}

impl Mul<Area> for f64 {
    type Output = Area;

    fn mul(self, rhs: Area) -> Self::Output {
        Area::new(self * rhs.value)
    }
}

impl Div<f64> for Area {
    type Output = Area;

    fn div(self, rhs: f64) -> Self::Output {
        Area::new(self.value / rhs)
    }
}

impl Mul<Density> for Area {
    type Output = Mass;

    fn mul(self, rhs: Density) -> Self::Output {
        Mass::new(self.value * rhs.value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    x: f64,
    y: f64,
}

impl Position {
    pub const ORIGIN: Position = Position { x: 0.0, y: 0.0 };

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

    pub fn to_polar_radius(&self, origin: Position) -> Length {
        (*self - origin).length()
    }

    pub fn to_polar_angle(&self, origin: Position) -> Angle {
        let displacement = *self - origin;
        let radians = displacement.y.atan2(displacement.x);
        Angle::from_radians(if radians >= 0.0 { radians } else { radians + 2.0 * PI })
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
    pub const ZERO: Displacement = Displacement { x: 0.0, y: 0.0 };

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

    pub fn length(&self) -> Length {
        Length::new(self.x.hypot(self.y))
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
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
    pub const ZERO: Velocity = Velocity { x: 0.0, y: 0.0 };

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
pub struct Acceleration {
    x: f64,
    y: f64,
}

impl Acceleration {
    pub fn new(x: f64, y: f64) -> Self {
        Acceleration { x, y }
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
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

impl Add<Mass> for Mass {
    type Output = Mass;

    fn add(self, rhs: Mass) -> Self::Output {
        Mass::new(self.value + rhs.value)
    }
}

impl Mul<Acceleration> for Mass {
    type Output = Force;

    fn mul(self, rhs: Acceleration) -> Self::Output {
        Force::new(self.value * rhs.x(), self.value * rhs.y())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Density {
    value: f64,
}

impl Density {
    pub fn new(value: f64) -> Self {
        if value < 0.0 {
            panic!("Negative density: {}", value);
        }

        Density { value }
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

impl Neg for Force {
    type Output = Force;

    fn neg(self) -> Self::Output {
        Force::new(-self.x, -self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Torque {
    value: f64,
}

impl Torque {
    pub fn new(value: f64) -> Self {
        Torque { value }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Neg for Torque {
    type Output = Torque;

    fn neg(self) -> Self::Output {
        Torque::new(-self.value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct BioEnergy {
    value: f64,
}

impl BioEnergy {
    pub fn new(value: f64) -> Self {
        if value < 0.0 {
            panic!("Negative energy: {}", value);
        }

        BioEnergy { value }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn min(&self, e: BioEnergy) -> BioEnergy {
        BioEnergy::new(self.value.min(e.value))
    }
}

impl Add<BioEnergyDelta> for BioEnergy {
    type Output = BioEnergy;

    fn add(self, rhs: BioEnergyDelta) -> Self::Output {
        BioEnergy::new(self.value + rhs.value)
    }
}

impl AddAssign<BioEnergyDelta> for BioEnergy {
    fn add_assign(&mut self, rhs: BioEnergyDelta) {
        self.value += rhs.value;
    }
}

impl Sub<BioEnergyDelta> for BioEnergy {
    type Output = BioEnergy;

    fn sub(self, rhs: BioEnergyDelta) -> Self::Output {
        BioEnergy::new(self.value - rhs.value)
    }
}

impl Add<BioEnergy> for BioEnergy {
    type Output = BioEnergy;

    fn add(self, rhs: BioEnergy) -> Self::Output {
        BioEnergy::new(self.value + rhs.value)
    }
}

impl AddAssign for BioEnergy {
    fn add_assign(&mut self, rhs: BioEnergy) {
        self.value += rhs.value;
    }
}

impl Sub<BioEnergy> for BioEnergy {
    type Output = BioEnergy;

    fn sub(self, rhs: BioEnergy) -> Self::Output {
        BioEnergy::new(self.value - rhs.value)
    }
}

impl SubAssign for BioEnergy {
    fn sub_assign(&mut self, rhs: BioEnergy) {
        self.value -= rhs.value;
    }
}

impl Mul<f64> for BioEnergy {
    type Output = BioEnergy;

    fn mul(self, rhs: f64) -> Self::Output {
        BioEnergy::new(self.value * rhs)
    }
}

impl Mul<BioEnergy> for f64 {
    type Output = BioEnergy;

    fn mul(self, rhs: BioEnergy) -> Self::Output {
        BioEnergy::new(self * rhs.value)
    }
}

impl Div<f64> for BioEnergy {
    type Output = BioEnergy;

    fn div(self, rhs: f64) -> Self::Output {
        BioEnergy::new(self.value / rhs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct BioEnergyDelta {
    value: f64,
}

impl BioEnergyDelta {
    pub const ZERO: BioEnergyDelta = BioEnergyDelta { value: 0.0 };

    pub fn new(value: f64) -> Self {
        BioEnergyDelta { value }
    }

    #[allow(dead_code)]
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Add<BioEnergyDelta> for BioEnergyDelta {
    type Output = BioEnergyDelta;

    fn add(self, rhs: BioEnergyDelta) -> Self::Output {
        BioEnergyDelta::new(self.value + rhs.value)
    }
}

impl Mul<f64> for BioEnergyDelta {
    type Output = BioEnergyDelta;

    fn mul(self, rhs: f64) -> Self::Output {
        BioEnergyDelta::new(self.value * rhs)
    }
}

impl Mul<BioEnergyDelta> for f64 {
    type Output = BioEnergyDelta;

    fn mul(self, rhs: BioEnergyDelta) -> Self::Output {
        BioEnergyDelta::new(self * rhs.value)
    }
}

impl Div<f64> for BioEnergyDelta {
    type Output = BioEnergyDelta;

    fn div(self, rhs: f64) -> Self::Output {
        BioEnergyDelta::new(self.value / rhs)
    }
}

impl Neg for BioEnergyDelta {
    type Output = BioEnergyDelta;

    fn neg(self) -> Self::Output {
        BioEnergyDelta::new(-self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_negative_angle() {
        assert_eq!(Angle::from_radians(2.0 * PI - 1.0), Angle::from_radians(-1.0));
    }

    #[test]
    fn normalize_overlarge_angle() {
        assert_eq!(Angle::from_radians(PI), Angle::from_radians(3.0 * PI));
    }

    #[test]
    fn subtract_angles() {
        assert_eq!(Deflection::from_radians(1.0),
                   Angle::from_radians(2.5) - Angle::from_radians(1.5));
    }

    #[test]
    fn add_deflection_to_angle() {
        assert_eq!(Angle::from_radians(3.0), Angle::from_radians(1.0) + Deflection::from_radians(2.0));
    }

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
    fn multiply_area_by_density() {
        assert_eq!(Mass::new(3.0), Area::new(2.0) * Density::new(1.5));
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
    fn polar_radius_off_origin() {
        let radius = Position::new(4.0, 5.0).to_polar_radius(Position::new(1.0, 1.0));
        assert_eq!(Length::new(5.0), radius);
    }

    #[test]
    fn polar_angle_at_origin() {
        let angle = Position::new(0.0, 1.0).to_polar_angle(Position::new(0.0, 0.0));
        assert_eq!(Angle::from_radians(PI / 2.0), angle);
    }

    #[test]
    fn polar_angle_off_origin() {
        let angle = Position::new(1.0, 2.0).to_polar_angle(Position::new(1.0, 1.0));
        assert_eq!(Angle::from_radians(PI / 2.0), angle);
    }

    #[test]
    fn polar_angle_with_negative_y() {
        let angle = Position::new(0.0, -1.0).to_polar_angle(Position::new(0.0, 0.0));
        assert_eq!(Angle::from_radians(3.0 * PI / 2.0), angle);
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
    fn multiply_mass_by_acceleration() {
        assert_eq!(Force::new(0.75, -0.25),
                   Mass::new(0.5) * Acceleration::new(1.5, -0.5));
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

    #[test]
    fn negate_force() {
        assert_eq!(Force::new(-0.75, 0.25), -Force::new(0.75, -0.25));
    }

    #[test]
    fn negate_torque() {
        assert_eq!(Torque::new(-0.75), -Torque::new(0.75));
    }
}
