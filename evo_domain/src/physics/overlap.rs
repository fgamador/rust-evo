use crate::physics::node_graph::*;
use crate::physics::quantities::*;
use crate::physics::shapes::*;
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
    ) -> Vec<(NodeHandle<C>, Overlap)>
    where
        C: Circle + GraphNode<C>,
        E: GraphEdge<C>,
        ME: GraphMetaEdge,
    {
        let mut overlaps: Vec<(NodeHandle<C>, Overlap)> =
            Vec::with_capacity(graph.nodes().len() / 2);

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
        C: Circle + GraphNode<C>,
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

type PairOverlap<C> = ((NodeHandle<C>, Overlap), (NodeHandle<C>, Overlap));

pub fn find_pair_overlaps<C, E, ME>(
    graph: &mut NodeGraph<C, E, ME>,
    cell_handles: &mut SortableHandles<C>,
) -> Vec<PairOverlap<C>>
where
    C: Circle + GraphNode<C>,
    E: GraphEdge<C>,
    ME: GraphMetaEdge,
{
    cell_handles.sort_already_mostly_sorted_handles(|h1, h2| match h1 {
        SortableHandle::GraphNode(h1) => match h2 {
            SortableHandle::GraphNode(h2) => {
                cmp_by_min_x(graph.node(h1), graph.node(h2)) == Ordering::Less
            }
            SortableHandle::Cloud => true,
        },
        SortableHandle::Cloud => false,
    });

    let mut overlaps: Vec<PairOverlap<C>> = Vec::with_capacity(graph.nodes().len() * 2);

    for (i, &handle1) in cell_handles.handles().iter().enumerate() {
        for &handle2 in &cell_handles.handles()[(i + 1)..] {
            match handle1 {
                SortableHandle::GraphNode(handle1) => match handle2 {
                    SortableHandle::GraphNode(handle2) => {
                        let circle1 = graph.node(handle1);
                        let circle2 = graph.node(handle2);

                        // crucial optimization that works only if we are iterating through circles in min_x order
                        assert!(circle2.min_x() >= circle1.min_x());
                        if (circle2.min_x()) >= circle1.max_x() {
                            break;
                        }

                        if graph.have_edge(circle1, circle2) {
                            continue;
                        }

                        if let Some(overlap) = calc_overlap(circle1, circle2) {
                            overlaps.push(((handle1, overlap), (handle2, -overlap)));
                        }
                    }
                    SortableHandle::Cloud => {}
                },
                SortableHandle::Cloud => {}
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

#[derive(Debug)]
pub enum SortableHandle<N: NodeWithHandle<N>> {
    Cloud,
    GraphNode(NodeHandle<N>),
}

impl<N: NodeWithHandle<N>> Clone for SortableHandle<N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: NodeWithHandle<N>> Copy for SortableHandle<N> {}

#[derive(Debug)]
pub struct SortableHandles<N: NodeWithHandle<N>> {
    pub handles: Vec<SortableHandle<N>>,
}

impl<N: NodeWithHandle<N>> SortableHandles<N> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        SortableHandles { handles: vec![] }
    }

    pub fn handles(&self) -> &[SortableHandle<N>] {
        &self.handles
    }

    pub fn add_handle(&mut self, handle: SortableHandle<N>) {
        self.handles.push(handle);
    }

    pub fn remove_invalid_handles<F>(&mut self, is_valid_handle: F)
    where
        F: Fn(SortableHandle<N>) -> bool,
    {
        self.handles.retain(|&h| is_valid_handle(h));
    }

    pub fn sort_already_mostly_sorted_handles<F>(&mut self, is_less_than: F)
    where
        F: Fn(SortableHandle<N>, SortableHandle<N>) -> bool,
    {
        Self::insertion_sort_by(&mut self.handles, is_less_than);
    }

    fn insertion_sort_by<T, F>(seq: &mut [T], is_less_than: F)
    where
        T: Copy,
        F: Fn(T, T) -> bool,
    {
        for i in 1..seq.len() {
            for j in (1..=i).rev() {
                if is_less_than(seq[j - 1], seq[j]) {
                    break;
                }
                seq.swap(j - 1, j)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::simple_graph_elements::*;

    #[test]
    fn no_wall_overlaps() {
        let mut graph: NodeGraph<
            SimpleCircleNode,
            SimpleGraphEdge<SimpleCircleNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();
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
        let mut graph: NodeGraph<
            SimpleCircleNode,
            SimpleGraphEdge<SimpleCircleNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();
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
        let mut graph: NodeGraph<
            SimpleCircleNode,
            SimpleGraphEdge<SimpleCircleNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();
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
        let mut graph: NodeGraph<
            SimpleCircleNode,
            SimpleGraphEdge<SimpleCircleNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();
        let mut node_handles = SortableHandles::new();

        let node_handle0 = graph.add_node(SimpleCircleNode::new(
            Position::new(0.0, 0.0),
            Length::new(1.5),
        ));
        node_handles.add_handle(SortableHandle::GraphNode(node_handle0));
        let node_handle1 = graph.add_node(SimpleCircleNode::new(
            Position::new(2.0, 0.0),
            Length::new(2.0),
        ));
        node_handles.add_handle(SortableHandle::GraphNode(node_handle1));

        let overlaps = find_pair_overlaps(&mut graph, &mut node_handles);

        assert_eq!(overlaps.len(), 1);
        if let SortableHandle::GraphNode(handle0) = node_handles.handles()[0] {
            assert_eq!(
                overlaps[0].0,
                (handle0, Overlap::new(Displacement::new(-1.5, 0.0), 1.5))
            );
        }
        if let SortableHandle::GraphNode(handle1) = node_handles.handles()[1] {
            assert_eq!(
                overlaps[0].1,
                (handle1, Overlap::new(Displacement::new(1.5, 0.0), 1.5))
            );
        }
    }

    #[test]
    fn bonded_graph_pair_overlap_is_ignored() {
        let mut graph: NodeGraph<
            SimpleCircleNode,
            SimpleGraphEdge<SimpleCircleNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();
        let mut node_handles = SortableHandles::new();

        let node_handle0 = graph.add_node(SimpleCircleNode::new(
            Position::new(0.0, 0.0),
            Length::new(1.0),
        ));
        node_handles.add_handle(SortableHandle::GraphNode(node_handle0));
        let node_handle1 = graph.add_node(SimpleCircleNode::new(
            Position::new(1.5, 0.0),
            Length::new(1.0),
        ));
        node_handles.add_handle(SortableHandle::GraphNode(node_handle1));

        let edge = SimpleGraphEdge::new(&graph.nodes()[0], &graph.nodes()[1]);
        graph.add_edge(edge, 1, 0);

        let overlaps = find_pair_overlaps(&mut graph, &mut node_handles);

        assert!(overlaps.is_empty());
    }

    #[test]
    fn graph_pairs_overlap_after_movement() {
        let mut graph: NodeGraph<
            SimpleCircleNode,
            SimpleGraphEdge<SimpleCircleNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();
        let mut node_handles = SortableHandles::new();

        let node_handle0 = graph.add_node(SimpleCircleNode::new(
            Position::new(0.0, 0.0),
            Length::new(1.0),
        ));
        node_handles.add_handle(SortableHandle::GraphNode(node_handle0));
        let node_handle1 = graph.add_node(SimpleCircleNode::new(
            Position::new(3.0, 0.0),
            Length::new(1.0),
        ));
        node_handles.add_handle(SortableHandle::GraphNode(node_handle1));
        let node_handle2 = graph.add_node(SimpleCircleNode::new(
            Position::new(6.0, 0.0),
            Length::new(1.0),
        ));
        node_handles.add_handle(SortableHandle::GraphNode(node_handle2));

        graph.nodes_mut()[2].set_center(Position::new(1.5, 0.0));

        let overlaps = find_pair_overlaps(&mut graph, &mut node_handles);

        assert_eq!(overlaps.len(), 2);
        if let SortableHandle::GraphNode(handle0) = node_handles.handles()[0] {
            assert_eq!((overlaps[0].0).0, handle0);
        }
        if let SortableHandle::GraphNode(handle1) = node_handles.handles()[1] {
            assert_eq!((overlaps[0].1).0, handle1);
            assert_eq!((overlaps[1].0).0, handle1);
        }
        if let SortableHandle::GraphNode(handle2) = node_handles.handles()[2] {
            assert_eq!((overlaps[1].1).0, handle2);
        }
    }

    #[test]
    fn can_remove_invalid_handles() {
        let mut graph: NodeGraph<
            SimpleGraphNode,
            SimpleGraphEdge<SimpleGraphNode>,
            SimpleGraphMetaEdge,
        > = NodeGraph::new();
        let mut node_handles = SortableHandles::new();

        let node0_handle = graph.add_node(SimpleGraphNode::new(0));
        node_handles.add_handle(SortableHandle::GraphNode(node0_handle));
        let node1_handle = graph.add_node(SimpleGraphNode::new(1));
        node_handles.add_handle(SortableHandle::GraphNode(node1_handle));

        graph.remove_nodes(&vec![node0_handle]);
        node_handles.remove_invalid_handles(|h| match h {
            SortableHandle::GraphNode(h) => graph.is_valid_handle(h),
            SortableHandle::Cloud => false,
        });

        assert_eq!(node_handles.handles().len(), 1);
        if let SortableHandle::GraphNode(handle0) = node_handles.handles()[0] {
            assert_eq!(handle0, node0_handle);
        }
    }
}
