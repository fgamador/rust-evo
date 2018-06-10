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
            overlaps.push(Overlap::new(circle, Displacement::new(0.0, 0.0)));
        }
        overlaps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_circles() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circles = vec![];
        let overlaps = subject.find_overlaps(&circles);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn max_corner_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circles = vec![Circle::new(Position::new(9.5, 1.6), Length::new(1.0))];
        let overlaps = subject.find_overlaps(&circles);
        assert_eq!(1, overlaps.len());
//        assert_eq!(Overlap::new(&circles[0], Displacement::new(-0.5, -0.4)), overlaps[0]);
    }
}
