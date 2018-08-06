use physics::ball::*;
use physics::sortable_graph::*;
use physics::bond::*;
use physics::newtonian::Body;
use physics::quantities::*;
use physics::overlap::*;

#[derive(Debug)]
pub struct World {
    ball_graph: SortableGraph<Ball, Bond>,
    walls: Walls,
}

impl World {
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        World {
            ball_graph: SortableGraph::new(),
            walls: Walls::new(min_corner, max_corner),
        }
    }

    pub fn add_ball(&mut self, ball: Ball) {
        self.ball_graph.add_node(ball);
    }

    pub fn add_bond(&mut self, bond: Bond) {
        self.ball_graph.add_edge(bond);
    }

    pub fn balls(&self) -> &[Ball] {
        &self.ball_graph.nodes()
    }

    pub fn tick(&mut self) {
        self.walls.find_overlaps(self.ball_graph.nodes_mut(), |ball, overlap| {
            ball.environment_mut().add_overlap(overlap);
        });

        find_graph_pair_overlaps(&mut self.ball_graph, |ball, overlap| {
            ball.environment_mut().add_overlap(overlap);
        });

        for ball in self.ball_graph.nodes_mut() {
            ball.add_overlap_forces();
        }

        calc_bond_forces(&mut self.ball_graph, |ball, force| {
            ball.forces_mut().add_force(force);
        });

        let tick_duration = Duration::new(1.0);
        for ball in self.ball_graph.nodes_mut() {
            ball.exert_forces(tick_duration);
            ball.move_for(tick_duration);
        }

        for ball in self.ball_graph.nodes_mut() {
            ball.environment_mut().clear();
            ball.forces_mut().clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_moves_ball() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(1.0, 1.0)));
        world.tick();
        let ball = &world.balls()[0];
        assert!(ball.position().x() > 0.0);
        assert!(ball.position().y() > 0.0);
    }

    #[test]
    fn overlaps_do_not_persist() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(9.5, 9.5), Velocity::new(0.0, 0.0)));
        world.tick();
        let ball = &world.balls()[0];
        assert!(ball.environment().overlaps().is_empty());
    }

    #[test]
    fn forces_do_not_persist() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(9.5, 9.5), Velocity::new(0.0, 0.0)));
        world.tick();
        let ball = &world.balls()[0];
        assert_eq!(Force::new(0.0, 0.0), ball.forces().net_force());
    }

    #[test]
    fn ball_bounces_off_walls() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(9.5, 9.5), Velocity::new(1.0, 1.0)));

        world.tick();

        let ball = &world.balls()[0];
        assert!(ball.velocity().x() < 1.0);
        assert!(ball.velocity().y() < 1.0);
    }

    #[test]
    fn balls_bounce_off_each_other() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(1.0, 1.0)));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.4, 1.4), Velocity::new(-1.0, -1.0)));

        world.tick();

        let ball1 = &world.balls()[0];
        assert!(ball1.velocity().x() < 1.0);
        assert!(ball1.velocity().y() < 1.0);
        let ball2 = &world.balls()[1];
        assert!(ball2.velocity().x() > -1.0);
        assert!(ball2.velocity().y() > -1.0);
    }

    //#[test]
    fn _bond_pulls_balls_together() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(-1.0, -1.0)));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(1.5, 1.5), Velocity::new(1.0, 1.0)));
        let bond = Bond::new(&world.balls()[0], &world.balls()[1]);
        world.add_bond(bond);

        world.tick();

        let ball1 = &world.balls()[0];
        let ball2 = &world.balls()[1];
        assert!(ball1.velocity().x() > -1.0);
        assert!(ball1.velocity().y() > -1.0);
        assert!(ball2.velocity().x() < 1.0);
        assert!(ball2.velocity().y() < 1.0);
    }
}
