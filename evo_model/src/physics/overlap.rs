use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;
use std::cmp::Ordering;

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

pub fn find_graph_pair_overlaps<'a, C, E>(graph: &'a mut SortableGraph<C, E>, on_overlap: fn(&mut C, Overlap))
    where C: Circle
{
    find_pair_overlaps(&mut graph.nodes, &mut graph.node_indexes, on_overlap);
}

fn find_pair_overlaps<'a, C>(circles: &'a mut [C], mut indexes: &'a mut Vec<usize>, on_overlap: fn(&mut C, Overlap))
    where C: Circle
{
    sort_by_min_x(&circles, &mut indexes);

    let mut overlaps: Vec<(usize, Overlap)> = Vec::with_capacity(circles.len() * 2);

    for (i, index1) in indexes.iter().enumerate() {
        for index2 in &indexes[(i + 1)..] {
            let circle1 = &circles[*index1];
            let circle2 = &circles[*index2];

            // crucial optimization that works only if we are iterating through circles in min_x order
            assert!(circle2.min_x() >= circle1.min_x());
            if (circle2.min_x()) >= circle1.max_x() {
                break;
            }

            if let Some(overlap) = get_overlap(circle1, circle2) {
                overlaps.push((*index1, Overlap::new(overlap)));
                overlaps.push((*index2, Overlap::new(-overlap)));
            }
        }
    }

    for (index, overlap) in overlaps {
        on_overlap(&mut circles[index], overlap);
    }
}

fn sort_by_min_x<C: Circle>(circles: &[C], indexes: &mut [usize]) {
    // TODO convert this to insertion sort
    indexes.sort_unstable_by(|i1, i2| cmp_by_min_x(&circles[*i1], &circles[*i2]));
}

fn cmp_by_min_x<C: Circle>(c1: &C, c2: &C) -> Ordering {
    c1.min_x().partial_cmp(&c2.min_x()).unwrap()
}

fn get_overlap<C: Circle>(circle1: &C, circle2: &C) -> Option<Displacement> {
    let mut pair = PossibleCirclePairOverlap::new(circle1, circle2);
    if pair.bounding_boxes_overlap() && pair.circles_overlap() {
        Some(pair.get_incursion())
    } else {
        None
    }
}

struct PossibleCirclePairOverlap {
    x_offset: f64,
    y_offset: f64,
    just_touching_center_sep: f64,
    center_sep_sqr: f64,
}

impl PossibleCirclePairOverlap {
    fn new<C: Circle>(circle1: &C, circle2: &C) -> Self {
        PossibleCirclePairOverlap {
            x_offset: circle1.center().x() - circle2.center().x(),
            y_offset: circle1.center().y() - circle2.center().y(),
            just_touching_center_sep: circle1.radius().value() + circle2.radius().value(),
            center_sep_sqr: 0.0,
        }
    }

    fn bounding_boxes_overlap(&self) -> bool {
        self.x_offset.abs() < self.just_touching_center_sep && self.y_offset.abs() < self.just_touching_center_sep
    }

    fn circles_overlap(&mut self) -> bool {
        self.center_sep_sqr = sqr(self.x_offset) + sqr(self.y_offset);
        self.center_sep_sqr < sqr(self.just_touching_center_sep)
    }

    fn get_incursion(&self) -> Displacement {
        let center_sep = self.center_sep_sqr.sqrt();
        let overlap_mag = self.just_touching_center_sep - center_sep;
        let x_incursion = (self.x_offset / center_sep) * overlap_mag;
        let y_incursion = (self.y_offset / center_sep) * overlap_mag;
        Displacement::new(x_incursion, y_incursion)
    }
}

fn sqr(x: f64) -> f64 {
    x * x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_wall_overlaps() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circle = SpyCircle::new(Position::new(8.5, 0.75), Length::new(1.0));
        let mut circles = vec![circle];
        subject.find_overlaps(&mut circles, on_overlap);
        assert!(!circles[0].overlapped);
    }

    #[test]
    fn min_corner_wall_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circle = SpyCircle::new(Position::new(-9.5, -4.25), Length::new(1.0));
        let mut circles = vec![circle];
        subject.find_overlaps(&mut circles, on_overlap);
        assert_eq!(Overlap::new(Displacement::new(0.5, 0.25)), circles[0].overlap);
    }

    #[test]
    fn max_corner_wall_overlap() {
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));
        let circle = SpyCircle::new(Position::new(9.5, 1.75), Length::new(1.0));
        let mut circles = vec![circle];
        subject.find_overlaps(&mut circles, on_overlap);
        assert_eq!(Overlap::new(Displacement::new(-0.5, -0.75)), circles[0].overlap);
    }

    #[test]
    fn pair_overlap() {
        // {3, 4, 5} triangle (as {6, 8, 10})
        let circle1 = SpyCircle::new(Position::new(0.0, 0.0), Length::new(7.0));
        let circle2 = SpyCircle::new(Position::new(6.0, 8.0), Length::new(8.0));

        let overlap = get_overlap(&circle1, &circle2).unwrap();

        // overlap/hypotenuse 5 has legs 3 and 4
        assert_eq!(Displacement::new(-3.0, -4.0), overlap);
    }

    #[test]
    fn pair_x_and_y_overlap_without_circle_overlap() {
        let circle1 = SpyCircle::new(Position::new(0.0, 0.0), Length::new(1.0));
        let circle2 = SpyCircle::new(Position::new(1.5, 1.5), Length::new(1.0));

        let overlap = get_overlap(&circle1, &circle2);

        assert_eq!(None, overlap);
    }

    #[test]
    fn graph_pair_overlap() {
        let mut graph: SortableGraph<SpyCircle, DummyBond> = SortableGraph::new();
        graph.add_node(SpyCircle::new(Position::new(0.0, 0.0), Length::new(1.0)));
        graph.add_node(SpyCircle::new(Position::new(1.5, 0.0), Length::new(1.0)));

        find_graph_pair_overlaps(&mut graph, on_overlap);

        assert!(graph.nodes()[0].overlapped);
        assert!(graph.nodes()[1].overlapped);
    }

    #[test]
    fn graph_pairs_overlap_after_movement() {
        let mut graph: SortableGraph<SpyCircle, DummyBond> = SortableGraph::new();
        graph.add_node(SpyCircle::new(Position::new(0.0, 0.0), Length::new(1.0)));
        graph.add_node(SpyCircle::new(Position::new(3.0, 0.0), Length::new(1.0)));
        graph.add_node(SpyCircle::new(Position::new(6.0, 0.0), Length::new(1.0)));

        graph.nodes_mut()[2].center = Position::new(1.5, 0.0);

        find_graph_pair_overlaps(&mut graph, on_overlap);

        assert!(graph.nodes()[0].overlapped);
        assert!(graph.nodes()[1].overlapped);
        assert!(graph.nodes()[2].overlapped);
    }

    #[test]
    fn overlap_to_force() {
        let overlap = Overlap::new(Displacement::new(2.0, -3.0));
        assert_eq!(Force::new(2.0, -3.0), overlap.to_force());
    }

    fn on_overlap(circle: &mut SpyCircle, overlap: Overlap) {
        circle.overlapped = true;
        circle.overlap = overlap;
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct SpyCircle {
        pub center: Position,
        pub radius: Length,
        pub overlapped: bool,
        pub overlap: Overlap,
    }

    impl SpyCircle {
        pub fn new(center: Position, radius: Length) -> SpyCircle {
            SpyCircle { center, radius, overlapped: false, overlap: Overlap::new(Displacement::new(0.0, 0.0)) }
        }
    }

    impl Circle for SpyCircle {
        fn radius(&self) -> Length {
            return self.radius;
        }

        fn center(&self) -> Position {
            return self.center;
        }
    }

    struct DummyBond {}
}
