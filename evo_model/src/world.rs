use influences::*;
use physics::ball::*;
use physics::sortable_graph::*;
use physics::bond::*;
use physics::newtonian::Body;
use physics::quantities::*;

pub struct World {
    ball_graph: SortableGraph<Ball, Bond, AngleGusset>,
    influences: Vec<Box<Influence>>,
}

impl World {
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        World {
            ball_graph: SortableGraph::new(),
            influences: vec![
                Box::new(WallCollisions::new(min_corner, max_corner)),
                Box::new(PairCollisions::new()),
                Box::new(OverlapForces::new()),
            ],
        }
    }

    pub fn add_ball(&mut self, ball: Ball) {
        self.ball_graph.add_node(ball);
    }

    pub fn balls(&self) -> &[Ball] {
        &self.ball_graph.unsorted_nodes()
    }

    pub fn add_bond(&mut self, bond: Bond) {
        self.ball_graph.add_edge(bond);
    }

    pub fn bonds(&self) -> &[Bond] {
        &self.ball_graph.edges()
    }

    pub fn add_angle_gusset(&mut self, gusset: AngleGusset) {
        self.ball_graph.add_meta_edge(gusset);
    }

    pub fn tick(&mut self) {
        let tick_duration = Duration::new(1.0);
        let subticks_per_tick = 2;
        let subtick_duration = tick_duration / (subticks_per_tick as f64);

        for _subtick in 0..subticks_per_tick {
            self.apply_influences();
            self.apply_forces(subtick_duration);
            self.forget_forces();
        }
    }

    fn apply_influences(&mut self) {
        for influence in &self.influences {
            influence.apply(&mut self.ball_graph);
        }
        self.add_bond_forces();
        self.add_bond_angle_forces();
    }

    fn add_bond_forces(&mut self) {
        calc_bond_forces(&mut self.ball_graph, |ball, force| {
            ball.forces_mut().add_force(force);
        });
    }

    fn add_bond_angle_forces(&mut self) {
        calc_bond_angle_forces(&mut self.ball_graph, |ball, force| {
            ball.forces_mut().add_force(force);
        });
    }

    fn apply_forces(&mut self, subtick_duration: Duration) {
        for ball in self.ball_graph.unsorted_nodes_mut() {
            ball.exert_forces(subtick_duration);
            ball.move_for(subtick_duration);
        }
    }

    fn forget_forces(&mut self) -> () {
        for ball in self.ball_graph.unsorted_nodes_mut() {
            ball.environment_mut().clear();
            ball.forces_mut().clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

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

    #[test]
    fn bond_pulls_balls_together() {
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

    #[test]
    fn angle_gusset_exerts_force() {
        let mut world = World::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.1, 2.0), Velocity::new(0.0, 0.0)));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, -2.0), Velocity::new(0.0, 0.0)));

        let bond = Bond::new(&world.balls()[0], &world.balls()[1]);
        world.add_bond(bond);
        let bond = Bond::new(&world.balls()[1], &world.balls()[2]);
        world.add_bond(bond);

        let gusset = AngleGusset::new(&world.bonds()[0], &world.bonds()[1], Angle::from_radians(PI));
        world.add_angle_gusset(gusset);

        world.tick();

        let ball3 = &world.balls()[2];
        assert!(ball3.velocity().x() < 0.0);
    }
}
