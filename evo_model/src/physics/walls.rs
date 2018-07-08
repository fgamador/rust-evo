use physics::quantities::*;

#[derive(Debug, PartialEq)]
pub struct Circle {
    pub center: Position,
    pub radius: Length,
}

impl Circle {
    pub fn new(center: Position, radius: Length) -> Circle {
        Circle { center, radius }
    }

    pub fn center(&self) -> Position {
        return self.center;
    }

    pub fn to_bounding_box(&self) -> BoundingBox {
        BoundingBox::new(Position::new(self.center().x() - self.radius.value(),
                                       self.center().y() - self.radius.value()),
                         Position::new(self.center().x() + self.radius.value(),
                                       self.center().y() + self.radius.value()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox {
    min_corner: Position,
    max_corner: Position,
}

impl BoundingBox {
    pub fn new(min_corner: Position, max_corner: Position) -> BoundingBox {
        BoundingBox { min_corner, max_corner }
    }

    pub fn min_corner(&self) -> Position {
        self.min_corner
    }

    pub fn max_corner(&self) -> Position {
        self.max_corner
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Overlap<'a> {
    circle: &'a Circle,
    incursion: Displacement,
}

impl<'a> Overlap<'a> {
    pub fn new(circle: &Circle, overlap: Displacement) -> Overlap {
        Overlap { circle, incursion: overlap }
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

    pub fn find_overlaps<'a>(&self, circles: &'a Vec<Circle>) -> Vec<Overlap<'a>> {
        let mut overlaps = vec![];
        let zero = Displacement::new(0.0, 0.0);
        for ref circle in circles {
            let circle_box = circle.to_bounding_box();
            let min_corner_overlap = self.min_corner.minus(circle_box.min_corner()).max(zero);
            let max_corner_overlap = self.max_corner.minus(circle_box.max_corner()).min(zero);
            let overlap = min_corner_overlap.plus(max_corner_overlap);
            if overlap != zero {
                overlaps.push(Overlap::new(circle, overlap));
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
        let subject = Circle::new(Position::new(-0.5, 0.5), Length::new(1.0));
        assert_eq!(BoundingBox::new(Position::new(-1.5, -0.5),
                                    Position::new(0.5, 1.5)),
                   subject.to_bounding_box());
    }

    #[test]
    fn no_overlaps() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circles = vec![Circle::new(Position::new(8.5, 0.75), Length::new(1.0))];
        let overlaps = subject.find_overlaps(&circles);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn min_corner_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circles = vec![Circle::new(Position::new(-9.5, -4.25), Length::new(1.0))];
        let overlaps = subject.find_overlaps(&circles);
        assert_eq!(1, overlaps.len());
        assert_eq!(Overlap::new(&circles[0], Displacement::new(0.5, 0.25)), overlaps[0]);
    }

    #[test]
    fn max_corner_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circles = vec![Circle::new(Position::new(9.5, 1.75), Length::new(1.0))];
        let overlaps = subject.find_overlaps(&circles);
        assert_eq!(1, overlaps.len());
        assert_eq!(Overlap::new(&circles[0], Displacement::new(-0.5, -0.75)), overlaps[0]);
    }
}
