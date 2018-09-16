use physics::quantities::*;
use std::f64::consts::PI;

pub trait Circle {
    fn radius(&self) -> Length;

    fn center(&self) -> Position;

    fn area(&self) -> Area {
        self.radius() * self.radius() * PI
    }

    fn to_bounding_box(&self) -> Rectangle {
        Rectangle::new(Position::new(self.min_x(), self.min_y()),
                       Position::new(self.max_x(), self.max_y()))
    }

    fn min_x(&self) -> f64 {
        self.center().x() - self.radius().value()
    }

    fn max_x(&self) -> f64 {
        self.center().x() + self.radius().value()
    }

    fn min_y(&self) -> f64 {
        self.center().y() - self.radius().value()
    }

    fn max_y(&self) -> f64 {
        self.center().y() + self.radius().value()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rectangle {
    min_corner: Position,
    max_corner: Position,
}

impl Rectangle {
    pub fn new(min_corner: Position, max_corner: Position) -> Rectangle {
        Rectangle { min_corner, max_corner }
    }

    pub fn min_corner(&self) -> Position {
        self.min_corner
    }

    pub fn max_corner(&self) -> Position {
        self.max_corner
    }

    pub fn overlaps(&self, other: Rectangle) -> bool {
        let self_x_range = FloatRange::new(self.min_corner.x(), self.max_corner.x());
        let self_y_range = FloatRange::new(self.min_corner.y(), self.max_corner.y());
        let other_x_range = FloatRange::new(other.min_corner.x(), other.max_corner.x());
        let other_y_range = FloatRange::new(other.min_corner.y(), other.max_corner.y());
        self_x_range.overlaps(other_x_range) && self_y_range.overlaps(other_y_range)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatRange {
    min: f64,
    max: f64,
}

impl FloatRange {
    pub fn new(min: f64, max: f64) -> Self {
        if min > max {
            panic!("Min {} is greater than max {}", min, max);
        }

        FloatRange { min, max }
    }

    pub fn overlaps(&self, other: FloatRange) -> bool {
        self.max > other.min && self.min < other.max
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimpleCircle {
    pub center: Position,
    pub radius: Length,
}

impl SimpleCircle {
    pub fn new(center: Position, radius: Length) -> SimpleCircle {
        SimpleCircle { center, radius }
    }
}

impl Circle for SimpleCircle {
    fn radius(&self) -> Length {
        return self.radius;
    }

    fn center(&self) -> Position {
        return self.center;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn circle_knows_area() {
        let subject = SimpleCircle::new(Position::new(0.0, 0.0), Length::new(2.0));
        assert_eq!(Area::new(PI * 4.0), subject.area());
    }

    #[test]
    fn circle_bounding_box() {
        let subject = SimpleCircle::new(Position::new(-0.5, 0.5), Length::new(1.0));
        assert_eq!(Rectangle::new(Position::new(-1.5, -0.5),
                                  Position::new(0.5, 1.5)),
                   subject.to_bounding_box());
    }

    #[test]
    fn float_range_overlap() {
        assert!(!FloatRange::new(0.0, 0.9).overlaps(FloatRange::new(1.0, 2.0)));
        assert!(!FloatRange::new(0.0, 1.0).overlaps(FloatRange::new(1.0, 2.0)));
        assert!(FloatRange::new(0.0, 1.1).overlaps(FloatRange::new(1.0, 2.0)));
        assert!(FloatRange::new(1.1, 1.9).overlaps(FloatRange::new(1.0, 2.0)));
        assert!(FloatRange::new(1.9, 3.0).overlaps(FloatRange::new(1.0, 2.0)));
        assert!(!FloatRange::new(2.0, 3.0).overlaps(FloatRange::new(1.0, 2.0)));
        assert!(!FloatRange::new(2.1, 3.0).overlaps(FloatRange::new(1.0, 2.0)));
    }

    #[test]
    fn rectangles_with_only_x_overlap() {
        let rect1 = Rectangle::new(Position::new(0.0, 0.0),
                                   Position::new(1.5, 1.0));
        let rect2 = Rectangle::new(Position::new(1.0, 1.0),
                                   Position::new(2.0, 2.0));
        assert!(!rect1.overlaps(rect2));
    }

    #[test]
    fn rectangles_with_only_y_overlap() {
        let rect1 = Rectangle::new(Position::new(0.0, 0.0),
                                   Position::new(1.0, 1.5));
        let rect2 = Rectangle::new(Position::new(1.0, 1.0),
                                   Position::new(2.0, 2.0));
        assert!(!rect1.overlaps(rect2));
    }

    #[test]
    fn rectangles_that_overlap() {
        let rect1 = Rectangle::new(Position::new(0.0, 0.0),
                                   Position::new(1.5, 1.5));
        let rect2 = Rectangle::new(Position::new(1.0, 1.0),
                                   Position::new(2.0, 2.0));
        assert!(rect1.overlaps(rect2));
    }
}
