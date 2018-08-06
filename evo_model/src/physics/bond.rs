use physics::ball::*;
use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bond {
    ball1_handle: NodeHandle,
    ball2_handle: NodeHandle,
}

impl Bond {
    pub fn new(ball1: &Ball, ball2: &Ball) -> Self {
        Bond {
            ball1_handle: ball1.handle(),
            ball2_handle: ball2.handle(),
        }
    }
}

impl GraphEdge for Bond {
    fn handle1(&self) -> NodeHandle {
        self.ball1_handle
    }

    fn handle1_mut(&mut self) -> &mut NodeHandle {
        &mut self.ball1_handle
    }

    fn handle2(&self) -> NodeHandle {
        self.ball2_handle
    }

    fn handle2_mut(&mut self) -> &mut NodeHandle {
        &mut self.ball2_handle
    }
}

pub fn calc_bond_forces<'a, C, E>(graph: &'a mut SortableGraph<C, E>, on_stretch: fn(&mut C, Force))
    where C: Circle + GraphNode, E: GraphEdge
{
    // TODO
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics::quantities::*;

    #[test]
    fn new_bond_has_correct_ball_handles() {
        let mut graph: SortableGraph<Ball, Bond> = SortableGraph::new();

        graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.0, 1.0), Velocity::new(1.0, 1.0)));
        graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.0, 1.0), Velocity::new(1.0, 1.0)));

        let ball1 = &graph.nodes()[0];
        let ball2 = &graph.nodes()[1];

        let bond = Bond::new(ball1, ball2);

        assert_eq!(ball1, graph.node(bond.handle1()));
        assert_eq!(ball2, graph.node(bond.handle2()));
    }
}
