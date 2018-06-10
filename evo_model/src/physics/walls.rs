use physics::quantities::*;

pub struct Circle {
    pub center: Position,
    pub radius: f64,
}

pub struct Overlap {
}

pub struct Walls {
    min_corner: Position,
    max_corner: Position,
    overlaps: Vec<Overlap>
}

impl Walls {
    pub fn new(min_corner: Position,
               max_corner: Position) -> Walls {
        Walls { min_corner, max_corner, overlaps: vec![] }
    }

    pub fn find_overlaps(&self, circles: &Vec<Circle>) -> &Vec<Overlap> {
        &self.overlaps
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
}
