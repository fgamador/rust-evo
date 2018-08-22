use physics::ball::*;
use physics::sortable_graph::*;
use physics::bond::*;
use physics::quantities::*;
use physics::overlap::*;

pub trait Influence {
    fn influence(&self, ball_graph: &mut SortableGraph<Ball, Bond, AngleGusset>);
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
    fn influence(&self, ball_graph: &mut SortableGraph<Ball, Bond, AngleGusset>) {
        self.walls.find_overlaps(ball_graph.unsorted_nodes_mut(), |ball, overlap| {
            ball.environment_mut().add_overlap(overlap);
        });
    }
}
