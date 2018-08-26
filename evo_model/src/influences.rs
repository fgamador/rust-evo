use physics::ball::*;
use physics::bond::*;
use physics::overlap::*;
use physics::quantities::*;
use physics::sortable_graph::*;

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
            ball.forces_mut().add_force(overlap.to_force());
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
        let overlaps = find_graph_pair_overlaps(ball_graph);
        for (handle, overlap) in overlaps {
            let ball = ball_graph.node_mut(handle);
            ball.environment_mut().add_overlap(overlap);
            ball.forces_mut().add_force(overlap.to_force());
        }
    }
}

#[derive(Debug)]
pub struct BondForces {}

impl BondForces {
    pub fn new() -> Self {
        BondForces {}
    }
}

impl Influence for BondForces {
    fn apply(&self, ball_graph: &mut SortableGraph<Ball, Bond, AngleGusset>) {
        calc_bond_forces(ball_graph, |ball, force| {
            ball.forces_mut().add_force(force);
        });
    }
}

#[derive(Debug)]
pub struct BondAngleForces {}

impl BondAngleForces {
    pub fn new() -> Self {
        BondAngleForces {}
    }
}

impl Influence for BondAngleForces {
    fn apply(&self, ball_graph: &mut SortableGraph<Ball, Bond, AngleGusset>) {
        calc_bond_angle_forces(ball_graph, |ball, force| {
            ball.forces_mut().add_force(force);
        });
    }
}

#[derive(Debug)]
pub struct UniversalOverlap {
    overlap: Overlap
}

impl UniversalOverlap {
    pub fn new(overlap: Overlap) -> Self {
        UniversalOverlap {
            overlap
        }
    }
}

impl Influence for UniversalOverlap {
    fn apply(&self, ball_graph: &mut SortableGraph<Ball, Bond, AngleGusset>) {
        for ball in ball_graph.unsorted_nodes_mut() {
            ball.environment_mut().add_overlap(self.overlap);
        }
    }
}

#[derive(Debug)]
pub struct UniversalForce {
    force: Force
}

impl UniversalForce {
    pub fn new(force: Force) -> Self {
        UniversalForce {
            force
        }
    }
}

impl Influence for UniversalForce {
    fn apply(&self, ball_graph: &mut SortableGraph<Ball, Bond, AngleGusset>) {
        for ball in ball_graph.unsorted_nodes_mut() {
            ball.forces_mut().add_force(self.force);
        }
    }
}
