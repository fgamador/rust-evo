use physics::ball::*;
use physics::quantities::*;
use physics::shapes::*;
use physics::sortable_graph::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bond {
    handle1: NodeHandle,
    handle2: NodeHandle,
}

impl Bond {
    pub fn new(ball1: &GraphNode, ball2: &GraphNode) -> Self {
        Bond {
            handle1: ball1.handle(),
            handle2: ball2.handle(),
        }
    }
}

impl GraphEdge for Bond {
    fn handle1(&self) -> NodeHandle {
        self.handle1
    }

    fn handle1_mut(&mut self) -> &mut NodeHandle {
        &mut self.handle1
    }

    fn handle2(&self) -> NodeHandle {
        self.handle2
    }

    fn handle2_mut(&mut self) -> &mut NodeHandle {
        &mut self.handle2
    }
}

pub fn calc_bond_forces<'a, C, E>(graph: &'a mut SortableGraph<C, E>, on_force: fn(&mut C, Force))
    where C: Circle + GraphNode, E: GraphEdge
{
    // TODO
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics::quantities::*;

    #[test]
    fn bond_calculates_strain() {
        let ball1 = Ball::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(0.0, 0.0), Velocity::new(0.0, 0.0));
        let ball2 = Ball::new(Length::new(1.0), Mass::new(1.0),
                              Position::new(1.0, 1.0), Velocity::new(1.0, 1.0));

        let bond = Bond::new(&ball1, &ball2);

        // TODO
    }

    // TODO redundant
    #[test]
    fn new_bond_has_correct_ball_handles() {
        let mut graph: SortableGraph<Ball, Bond> = SortableGraph::new();

        let h1 = graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                          Position::new(1.0, 1.0), Velocity::new(1.0, 1.0)));
        let h2 = graph.add_node(Ball::new(Length::new(1.0), Mass::new(1.0),
                                          Position::new(1.0, 1.0), Velocity::new(1.0, 1.0)));

        let bond = Bond::new(graph.node(h1), graph.node(h2));
        graph.add_edge(bond);

        let ball1 = &graph.nodes()[0];
        let ball2 = &graph.nodes()[1];
        assert_eq!(ball1, graph.node(bond.handle1()));
        assert_eq!(ball2, graph.node(bond.handle2()));
    }
}
