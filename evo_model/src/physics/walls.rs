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

    pub fn to_bounding_box(&self) -> BoundingBox {
        BoundingBox::new(Position::new(self.center.x() - self.radius.value(), self.center.y() - self.radius.value()),
                         Position::new(self.center.x() + self.radius.value(), self.center.y() + self.radius.value()))
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
    overlap: Displacement,
}

impl<'a> Overlap<'a> {
    pub fn new(circle: &Circle, overlap: Displacement) -> Overlap {
        Overlap { circle, overlap }
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
        for ref circle in circles {
            let mut overlap_x = 0.0;
            let mut overlap_y = 0.0;
            let circle_box = circle.to_bounding_box();
            if circle_box.min_corner().x() < self.min_corner.x() {
                overlap_x += self.min_corner.x() - circle_box.min_corner().x();
            }
            if circle_box.min_corner().y() < self.min_corner.y() {
                overlap_y += self.min_corner.y() - circle_box.min_corner().y();
            }
            if circle_box.max_corner().x() > self.max_corner.x() {
                overlap_x += self.max_corner.x() - circle_box.max_corner().x();
            }
            if circle_box.max_corner().y() > self.max_corner.y() {
                overlap_y += self.max_corner.y() - circle_box.max_corner().y();
            }
            if overlap_x != 0.0 || overlap_y != 0.0 {
                overlaps.push(Overlap::new(circle, Displacement::new(overlap_x, overlap_y)));
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
