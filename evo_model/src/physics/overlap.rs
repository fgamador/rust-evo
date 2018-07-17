use physics::quantities::*;
use physics::shapes::*;

// TODO add width to Overlap, or maybe make incursion magnitude an Area

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Overlap
{
    incursion: Displacement,
}

impl Overlap
{
    pub fn new(incursion: Displacement) -> Self {
        Overlap { incursion }
    }

    // TODO move this to a Spring class
    pub fn to_force(&self) -> Force {
        const SPRING_CONSTANT: f64 = 1.0;
        Force::new(self.incursion.x() * SPRING_CONSTANT, self.incursion.y() * SPRING_CONSTANT)
    }
}

#[derive(Debug)]
pub struct Walls {
    min_corner: Position,
    max_corner: Position,
}

impl Walls {
    pub fn new(min_corner: Position, max_corner: Position) -> Walls {
        Walls { min_corner, max_corner }
    }

    pub fn find_overlaps<'a, C>(&self, circles: &'a mut [C], on_overlap: fn(&mut C, Overlap))
        where C: Circle
    {
        let zero = Displacement::new(0.0, 0.0);
        for circle in circles {
            let circle_box = circle.to_bounding_box();
            let min_corner_overlap = (self.min_corner - circle_box.min_corner()).max(zero);
            let max_corner_overlap = (self.max_corner - circle_box.max_corner()).min(zero);
            let overlap = min_corner_overlap + max_corner_overlap;
            if overlap != zero {
                on_overlap(circle, Overlap::new(overlap));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_overlaps() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let mut circles = vec![OverlappableCircle::new(Position::new(8.5, 0.75), Length::new(1.0))];
        subject.find_overlaps(&mut circles, on_overlap);
        assert_eq!(Overlap::new(Displacement::new(0.0, 0.0)), circles[0].overlap);
    }

    #[test]
    fn min_corner_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let mut circles = vec![OverlappableCircle::new(Position::new(-9.5, -4.25), Length::new(1.0))];
        subject.find_overlaps(&mut circles, on_overlap);
        assert_eq!(Overlap::new(Displacement::new(0.5, 0.25)), circles[0].overlap);
    }

    #[test]
    fn max_corner_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let mut circles = vec![OverlappableCircle::new(Position::new(9.5, 1.75), Length::new(1.0))];
        subject.find_overlaps(&mut circles, on_overlap);
        assert_eq!(Overlap::new(Displacement::new(-0.5, -0.75)), circles[0].overlap);
    }

    #[test]
    fn overlap_to_force() {
        let overlap = Overlap::new(Displacement::new(2.0, -3.0));
        assert_eq!(Force::new(2.0, -3.0), overlap.to_force());
    }

    fn on_overlap(circle: &mut OverlappableCircle, overlap: Overlap) {
        circle.overlap = overlap;
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct OverlappableCircle {
        pub center: Position,
        pub radius: Length,
        pub overlap: Overlap,
    }

    impl OverlappableCircle {
        pub fn new(center: Position, radius: Length) -> OverlappableCircle {
            OverlappableCircle { center, radius, overlap: Overlap::new(Displacement::new(0.0, 0.0)) }
        }
    }

    impl Circle for OverlappableCircle {
        fn radius(&self) -> Length {
            return self.radius;
        }

        fn center(&self) -> Position {
            return self.center;
        }
    }
}
