use physics::ball::*;
use physics::sortable_graph::*;
use physics::bond::*;
use physics::quantities::*;
use physics::overlap::*;

pub trait Influence {
    fn apply(&self, ball_graph: &mut SortableGraph<Ball, Bond, AngleGusset>);
}

#[derive(Debug)]
pub struct WallCollisions {
    walls: Walls,
}

impl WallCollisions {
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        WallCollisions {
            walls: Walls::new(min_corner, max_corner),
        }
    }
}

impl Influence for WallCollisions {
    fn apply(&self, ball_graph: &mut SortableGraph<Ball, Bond, AngleGusset>) {
        self.walls.find_overlaps(ball_graph.unsorted_nodes_mut(), |ball, overlap| {
            ball.environment_mut().add_overlap(overlap);
        });
    }
}

#[derive(Debug)]
pub struct PairCollisions {}

impl PairCollisions {
    pub fn new() -> Self {
        PairCollisions {}
    }
}

impl Influence for PairCollisions {
    fn apply(&self, ball_graph: &mut SortableGraph<Ball, Bond, AngleGusset>) {
        find_graph_pair_overlaps_outer(ball_graph, |ball, overlap| {
            ball.environment_mut().add_overlap(overlap);
        });
    }
}
