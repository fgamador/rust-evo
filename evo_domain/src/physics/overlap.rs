use crate::physics::quantities::*;
use crate::physics::shapes::*;
use crate::physics::sortable_graph::*;
use crate::physics::util::*;
use std::cmp::Ordering;
use std::ops::Neg;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Overlap {
    incursion: Displacement,
    width: f64,
}

impl Overlap {
    pub fn new(incursion: Displacement, width: f64) -> Self {
        Overlap { incursion, width }
    }

    pub fn incursion(&self) -> Displacement {
        self.incursion
    }

    pub fn magnitude(&self) -> f64 {
        self.incursion.length().value()
    }
}

impl Neg for Overlap {
    type Output = Overlap;

    fn neg(self) -> Self::Output {
        Overlap::new(-self.incursion, self.width)
    }
}

#[derive(Debug)]
pub struct Walls {
    min_corner: Position,
    max_corner: Position,
}

impl Walls {
    pub fn new(min_corner: Position, max_corner: Position) -> Walls {
        Walls {
            min_corner,
            max_corner,
        }
    }

    pub fn find_overlaps<C, E, ME>(
        &self,
        graph: &mut NodeGraph<C, E, ME>,
    ) -> Vec<(NodeHandle, Overlap)>
    where
        C: Circle + GraphNode,
        E: GraphEdge,
        ME: GraphMetaEdge,
    {
        let mut overlaps: Vec<(NodeHandle, Overlap)> = Vec::with_capacity(graph.nodes().len() / 2);

        for circle in graph.nodes() {
            if let Some(incursion) = self.calc_incursion(circle) {
                overlaps.push((
                    circle.node_handle(),
                    Overlap::new(incursion, circle.radius().value()),
                ));
            }
        }

        overlaps
    }

    fn calc_incursion<C>(&self, circle: &C) -> Option<Displacement>
    where
        C: Circle + GraphNode,
    {
        let circle_box = circle.to_bounding_box();
        let min_corner_incursion =
            (self.min_corner - circle_box.min_corner()).max(Displacement::ZERO);
        let max_corner_incursion =
            (self.max_corner - circle_box.max_corner()).min(Displacement::ZERO);
        let incursion = min_corner_incursion + max_corner_incursion;
        if incursion != Displacement::ZERO {
            Some(incursion)
        } else {
            None
        }
    }
}

pub fn find_pair_overlaps<C, E, ME>(
    graph: &mut NodeGraph<C, E, ME>,
    cell_handles: &mut SortableHandles,
) -> Vec<((NodeHandle, Overlap), (NodeHandle, Overlap))>
where
    C: Circle + GraphNode,
    E: GraphEdge,
    ME: GraphMetaEdge,
{
    let nodes = &graph.nodes();
    cell_handles.sort_already_mostly_sorted_node_handles(nodes, cmp_by_min_x);

    let mut overlaps: Vec<((NodeHandle, Overlap), (NodeHandle, Overlap))> =
        Vec::with_capacity(graph.nodes().len() * 2);

    for (i, handle1) in cell_handles.node_handles().iter().enumerate() {
        for handle2 in &cell_handles.node_handles()[(i + 1)..] {
            let circle1 = graph.node(*handle1);
            let circle2 = graph.node(*handle2);

            // crucial optimization that works only if we are iterating through circles in min_x order
            assert!(circle2.min_x() >= circle1.min_x());
            if (circle2.min_x()) >= circle1.max_x() {
                break;
            }

            if graph.have_edge(circle1, circle2) {
                continue;
            }

            if let Some(overlap) = calc_overlap(circle1, circle2) {
                overlaps.push(((*handle1, overlap), (*handle2, -overlap)));
            }
        }
    }

    overlaps
}

fn cmp_by_min_x<C: Circle>(c1: &C, c2: &C) -> Ordering {
    c1.min_x().partial_cmp(&c2.min_x()).unwrap()
}

pub fn calc_overlap<C: Circle>(circle1: &C, circle2: &C) -> Option<Overlap> {
    let mut pair = PossibleCirclePairOverlap::new(circle1, circle2);
    if pair.bounding_boxes_overlap() && pair.circles_overlap() {
        let width = circle1.radius().value().min(circle2.radius().value());
        Some(Overlap::new(pair.get_incursion(), width))
    } else {
        None
    }
}

struct PossibleCirclePairOverlap {
    center1_offset: Displacement,
    just_touching_center_sep: f64,
    center_sep_sqr: f64,
}

impl PossibleCirclePairOverlap {
    fn new<C: Circle>(circle1: &C, circle2: &C) -> Self {
        PossibleCirclePairOverlap {
            center1_offset: circle1.center() - circle2.center(),
            just_touching_center_sep: circle1.radius().value() + circle2.radius().value(),
            center_sep_sqr: 0.0,
        }
    }

    fn bounding_boxes_overlap(&self) -> bool {
        self.center1_offset.x().abs() < self.just_touching_center_sep
            && self.center1_offset.y().abs() < self.just_touching_center_sep
    }

    fn circles_overlap(&mut self) -> bool {
        self.center_sep_sqr = self.center1_offset.value().length_squared();
        self.center_sep_sqr < sqr(self.just_touching_center_sep)
    }

    fn get_incursion(&self) -> Displacement {
        let center_sep = self.center_sep_sqr.sqrt();
        let magnitude = self.just_touching_center_sep - center_sep;
        if self.center_sep_sqr == 0.0 {
            // TODO random direction?
            Displacement::new(magnitude, 0.0)
        } else {
            (self.center1_offset / center_sep) * magnitude
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::simple_graph_elements::*;

    #[test]
    fn no_wall_overlaps() {
        let mut graph: NodeGraph<SimpleCircleNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            NodeGraph::new();
        graph.add_node(SimpleCircleNode::new(
            Position::new(8.5, 0.75),
            Length::new(1.0),
        ));
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));

        let overlaps = subject.find_overlaps(&mut graph);

        assert!(overlaps.is_empty());
    }

    #[test]
    fn min_corner_wall_overlap_uses_radius_as_width() {
        let mut graph: NodeGraph<SimpleCircleNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            NodeGraph::new();
        graph.add_node(SimpleCircleNode::new(
            Position::new(-9.5, -4.25),
            Length::new(2.0),
        ));
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));

        let overlaps = subject.find_overlaps(&mut graph);

        assert_eq!(overlaps.len(), 1);
        assert_eq!(
            overlaps[0],
            (
                graph.nodes()[0].node_handle(),
                Overlap::new(Displacement::new(1.5, 1.25), 2.0)
            )
        );
    }

    #[test]
    fn max_corner_wall_overlap_uses_radius_as_width() {
        let mut graph: NodeGraph<SimpleCircleNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            NodeGraph::new();
        graph.add_node(SimpleCircleNode::new(
            Position::new(9.5, 1.75),
            Length::new(2.0),
        ));
        let subject = Walls::new(Position::new(-10.0, -5.0), Position::new(10.0, 2.0));

        let overlaps = subject.find_overlaps(&mut graph);

        assert_eq!(overlaps.len(), 1);
        assert_eq!(
            overlaps[0],
            (
                graph.nodes()[0].node_handle(),
                Overlap::new(Displacement::new(-1.5, -1.75), 2.0)
            )
        );
    }

    #[test]
    fn pair_overlap() {
        // {3, 4, 5} triangle (as {6, 8, 10})
        let circle1 = SimpleCircleNode::new(Position::new(0.0, 0.0), Length::new(7.0));
        let circle2 = SimpleCircleNode::new(Position::new(6.0, 8.0), Length::new(8.0));

        let overlap = calc_overlap(&circle1, &circle2).unwrap();

        // overlap/hypotenuse 5 has legs 3 and 4
        assert_eq!(overlap.incursion(), Displacement::new(-3.0, -4.0));
    }

    #[test]
    fn pair_with_matching_centers() {
        let circle1 = SimpleCircleNode::new(Position::new(0.0, 0.0), Length::new(1.0));
        let circle2 = SimpleCircleNode::new(Position::new(0.0, 0.0), Length::new(1.0));

        let overlap = calc_overlap(&circle1, &circle2).unwrap();

        // just dump the whole incursion into x
        assert_eq!(overlap.incursion(), Displacement::new(2.0, 0.0));
    }

    #[test]
    fn pair_x_and_y_overlap_without_circle_overlap() {
        let circle1 = SimpleCircleNode::new(Position::new(0.0, 0.0), Length::new(1.0));
        let circle2 = SimpleCircleNode::new(Position::new(1.5, 1.5), Length::new(1.0));

        let overlap = calc_overlap(&circle1, &circle2);

        assert_eq!(overlap, None);
    }

    #[test]
    fn graph_pair_overlaps_use_min_radius_as_width() {
        let mut graph: NodeGraph<SimpleCircleNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            NodeGraph::new();
        let mut node_handles = SortableHandles::new();
        let node_handle0 = graph.add_node(SimpleCircleNode::new(
            Position::new(0.0, 0.0),
            Length::new(1.5),
        ));
        let node_handle1 = graph.add_node(SimpleCircleNode::new(
            Position::new(2.0, 0.0),
            Length::new(2.0),
        ));
        node_handles.add_node_handle(node_handle0);
        node_handles.add_node_handle(node_handle1);

        let overlaps = find_pair_overlaps(&mut graph, &mut node_handles);

        assert_eq!(overlaps.len(), 1);
        assert_eq!(
            overlaps[0].0,
            (
                node_handles.node_handles()[0],
                Overlap::new(Displacement::new(-1.5, 0.0), 1.5)
            )
        );
        assert_eq!(
            overlaps[0].1,
            (
                node_handles.node_handles()[1],
                Overlap::new(Displacement::new(1.5, 0.0), 1.5)
            )
        );
    }

    #[test]
    fn bonded_graph_pair_overlap_is_ignored() {
        let mut graph: NodeGraph<SimpleCircleNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            NodeGraph::new();
        let mut node_handles = SortableHandles::new();
        let node_handle0 = graph.add_node(SimpleCircleNode::new(
            Position::new(0.0, 0.0),
            Length::new(1.0),
        ));
        let node_handle1 = graph.add_node(SimpleCircleNode::new(
            Position::new(1.5, 0.0),
            Length::new(1.0),
        ));
        node_handles.add_node_handle(node_handle0);
        node_handles.add_node_handle(node_handle1);

        let edge = SimpleGraphEdge::new(&graph.nodes()[0], &graph.nodes()[1]);
        graph.add_edge(edge, 1, 0);

        let overlaps = find_pair_overlaps(&mut graph, &mut node_handles);

        assert!(overlaps.is_empty());
    }

    #[test]
    fn graph_pairs_overlap_after_movement() {
        let mut graph: NodeGraph<SimpleCircleNode, SimpleGraphEdge, SimpleGraphMetaEdge> =
            NodeGraph::new();
        let mut node_handles = SortableHandles::new();
        let node_handle0 = graph.add_node(SimpleCircleNode::new(
            Position::new(0.0, 0.0),
            Length::new(1.0),
        ));
        let node_handle1 = graph.add_node(SimpleCircleNode::new(
            Position::new(3.0, 0.0),
            Length::new(1.0),
        ));
        let node_handle2 = graph.add_node(SimpleCircleNode::new(
            Position::new(6.0, 0.0),
            Length::new(1.0),
        ));
        node_handles.add_node_handle(node_handle0);
        node_handles.add_node_handle(node_handle1);
        node_handles.add_node_handle(node_handle2);

        graph.nodes_mut()[2].set_center(Position::new(1.5, 0.0));

        let overlaps = find_pair_overlaps(&mut graph, &mut node_handles);

        assert_eq!(overlaps.len(), 2);
        assert_eq!((overlaps[0].0).0, node_handles.node_handles()[0]);
        assert_eq!((overlaps[0].1).0, node_handles.node_handles()[1]);
        assert_eq!((overlaps[1].0).0, node_handles.node_handles()[1]);
        assert_eq!((overlaps[1].1).0, node_handles.node_handles()[2]);
    }
}
