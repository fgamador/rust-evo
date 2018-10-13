use environment::environment::*;
use environment::influences::*;
use physics::bond::*;
use physics::newtonian::NewtonianBody;
use physics::quantities::*;
use physics::shapes::Circle;
use physics::sortable_graph::*;

pub struct World<T>
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    ball_graph: SortableGraph<T, Bond, AngleGusset>,
    influences: Vec<Box<Influence<T>>>,
}

impl<T> World<T>
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        Self::with_influences_static(vec![
            Box::new(WallCollisions::new(min_corner, max_corner)),
            Box::new(PairCollisions::new()),
            Box::new(BondForces::new()),
            Box::new(BondAngleForces::new()),
        ])
    }

    pub fn with_influences_static(influences: Vec<Box<Influence<T>>>) -> Self {
        World {
            ball_graph: SortableGraph::new(),
            influences,
        }
    }

    pub fn with_influences(&mut self, influences: Vec<Box<Influence<T>>>) -> &mut Self {
        self.influences = influences;
        self
    }

    pub fn add_ball(&mut self, ball: T) {
        self.ball_graph.add_node(ball);
    }

    pub fn balls(&self) -> &[T] {
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
            self.exert_forces(subtick_duration);
            self.clear_influences();
        }
    }

    fn apply_influences(&mut self) {
        for influence in &self.influences {
            influence.apply(&mut self.ball_graph);
        }
    }

    fn exert_forces(&mut self, subtick_duration: Duration) {
        for ball in self.ball_graph.unsorted_nodes_mut() {
            ball.exert_forces(subtick_duration);
            ball.move_for(subtick_duration);
        }
    }

    fn clear_influences(&mut self) -> () {
        for ball in self.ball_graph.unsorted_nodes_mut() {
            ball.environment_mut().clear();
            ball.forces_mut().clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics::ball::Ball;
    use physics::overlap::Overlap;

    #[test]
    fn tick_moves_ball() {
        let mut world = World::with_influences_static(vec![]);
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(1.0, 1.0)));
        world.tick();
        let ball = &world.balls()[0];
        assert!(ball.position().x() > 0.0);
        assert!(ball.position().y() > 0.0);
    }

    #[test]
    fn tick_with_force_accelerates_ball() {
        let mut world = World::with_influences_static(vec![
            Box::new(UniversalForce::new(Force::new(1.0, 1.0)))
        ]);
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        world.tick();
        let ball = &world.balls()[0];
        assert!(ball.velocity().x() > 0.0);
        assert!(ball.velocity().y() > 0.0);
    }

    #[test]
    fn overlaps_do_not_persist() {
        let mut world = World::with_influences_static(vec![
            Box::new(UniversalOverlap::new(Overlap::new(Displacement::new(1.0, 1.0))))
        ]);
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        world.tick();
        let ball = &world.balls()[0];
        assert!(ball.environment().overlaps().is_empty());
    }

    #[test]
    fn forces_do_not_persist() {
        let mut world = World::with_influences_static(vec![
            Box::new(UniversalForce::new(Force::new(1.0, 1.0)))
        ]);
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        world.tick();
        let ball = &world.balls()[0];
        assert_eq!(Force::new(0.0, 0.0), ball.forces().net_force());
    }
}
