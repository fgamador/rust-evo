use physics::quantities::*;
use std::f64::consts::PI;

pub trait Circle {
    fn radius(&self) -> Length;

    fn center(&self) -> Position;

    fn area(&self) -> Area {
        self.radius() * self.radius() * PI
    }

    fn to_bounding_box(&self) -> Rectangle {
        Rectangle::new(Position::new(self.min_x(),
                                     self.center().y() - self.radius().value()),
                       Position::new(self.center().x() + self.radius().value(),
                                     self.center().y() + self.radius().value()))
    }

    fn min_x(&self) -> f64 {
        self.center().x() - self.radius().value()
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
}
