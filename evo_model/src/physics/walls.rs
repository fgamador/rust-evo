use physics::quantities::*;
use std::fmt::Debug;

pub trait Circle {
    fn radius(&self) -> Length;

    fn center(&self) -> Position;

    fn to_bounding_box(&self) -> Rectangle {
        Rectangle::new(Position::new(self.center().x() - self.radius().value(),
                                     self.center().y() - self.radius().value()),
                       Position::new(self.center().x() + self.radius().value(),
                                     self.center().y() + self.radius().value()))
    }
}

#[derive(Debug, PartialEq)]
pub struct SimpleCircle {
    pub center: Position,
    pub radius: Length,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Overlap<'a, C>
    where C: 'a + Circle + Debug + PartialEq
{
    circle: &'a C,
    incursion: Displacement,
}

impl<'a, C> Overlap<'a, C>
    where C: 'a + Circle + Debug + PartialEq
{
    pub fn new(circle: &C, incursion: Displacement) -> Overlap<C> {
        Overlap { circle, incursion }
    }
}

pub struct Walls {
    min_corner: Position,
    max_corner: Position,
}

impl Walls {
    pub fn new(min_corner: Position, max_corner: Position) -> Walls {
        Walls { min_corner, max_corner }
    }

    pub fn find_overlaps<'a, C>(&self, circles: &'a Vec<C>) -> Vec<Overlap<'a, C>>
        where C: 'a + Circle + Debug + PartialEq
    {
        let mut overlaps = vec![];
        let zero = Displacement::new(0.0, 0.0);
        for ref circle in circles {
            let circle_box = circle.to_bounding_box();
            let min_corner_overlap = self.min_corner.minus(circle_box.min_corner()).max(zero);
            let max_corner_overlap = self.max_corner.minus(circle_box.max_corner()).min(zero);
            let overlap = min_corner_overlap.plus(max_corner_overlap);
            if overlap != zero {
                overlaps.push(Overlap::new(*circle, overlap));
            }
        }
        overlaps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_bounding_box() {
        let subject = SimpleCircle::new(Position::new(-0.5, 0.5), Length::new(1.0));
        assert_eq!(Rectangle::new(Position::new(-1.5, -0.5),
                                  Position::new(0.5, 1.5)),
                   subject.to_bounding_box());
    }

    #[test]
    fn no_overlaps() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circles = vec![SimpleCircle::new(Position::new(8.5, 0.75), Length::new(1.0))];
        let overlaps = subject.find_overlaps(&circles);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn min_corner_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circles = vec![SimpleCircle::new(Position::new(-9.5, -4.25), Length::new(1.0))];
        let overlaps = subject.find_overlaps(&circles);
        assert_eq!(1, overlaps.len());
        assert_eq!(Overlap::new(&circles[0], Displacement::new(0.5, 0.25)), overlaps[0]);
    }

    #[test]
    fn max_corner_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circles = vec![SimpleCircle::new(Position::new(9.5, 1.75), Length::new(1.0))];
        let overlaps = subject.find_overlaps(&circles);
        assert_eq!(1, overlaps.len());
        assert_eq!(Overlap::new(&circles[0], Displacement::new(-0.5, -0.75)), overlaps[0]);
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
