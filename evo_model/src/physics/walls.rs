use physics::quantities::*;
use physics::shapes::*;

#[derive(Debug)]
pub struct Walls {
    min_corner: Position,
    max_corner: Position,
}

impl Walls {
    pub fn new(min_corner: Position, max_corner: Position) -> Walls {
        Walls { min_corner, max_corner }
    }

    pub fn find_overlaps<'a, C>(&self, circles: &'a mut Vec<C>) -> Vec<(&'a mut C, Overlap)>
        where C: Circle
    {
        let mut overlaps = vec![];
        let zero = Displacement::new(0.0, 0.0);
        for circle in circles {
            let circle_box = circle.to_bounding_box();
            let min_corner_overlap = (self.min_corner - circle_box.min_corner()).max(zero);
            let max_corner_overlap = (self.max_corner - circle_box.max_corner()).min(zero);
            let overlap = min_corner_overlap + max_corner_overlap;
            if overlap != zero {
                overlaps.push((circle, Overlap::new(overlap)));
            }
        }
        overlaps
    }
}

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_overlaps() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let mut circles = vec![SimpleCircle::new(Position::new(8.5, 0.75), Length::new(1.0))];
        let overlaps = subject.find_overlaps(&mut circles);
        assert!(overlaps.is_empty());
    }

    #[test]
    fn min_corner_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let mut circles = vec![SimpleCircle::new(Position::new(-9.5, -4.25), Length::new(1.0))];
        let overlaps = subject.find_overlaps(&mut circles);
        assert_eq!(1, overlaps.len());
        assert_eq!(Overlap::new(Displacement::new(0.5, 0.25)), overlaps[0].1);
    }

    #[test]
    fn max_corner_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let mut circles = vec![SimpleCircle::new(Position::new(9.5, 1.75), Length::new(1.0))];
        let overlaps = subject.find_overlaps(&mut circles);
        assert_eq!(1, overlaps.len());
        assert_eq!(Overlap::new(Displacement::new(-0.5, -0.75)), overlaps[0].1);
    }
}
