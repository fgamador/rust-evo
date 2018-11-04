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
    min_corner: Position,
    max_corner: Position,
    ball_graph: SortableGraph<T, Bond, AngleGusset>,
    influences: Vec<Box<Influence<T>>>,
}

impl<T> World<T>
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    pub fn new(min_corner: Position, max_corner: Position) -> Self {
        World {
            min_corner,
            max_corner,
            ball_graph: SortableGraph::new(),
            influences: vec![],
        }
    }

    pub fn new2(min_corner: Position, max_corner: Position) -> Self {
        let world = World::new(min_corner, max_corner);
        world.with_standard_influences()
    }

    pub fn with_standard_influences(self) -> Self {
        self.with_perimeter_walls().with_influences(vec![
            Box::new(PairCollisions::new()),
            Box::new(BondForces::new()),
            Box::new(BondAngleForces::new()),
        ])
    }

    pub fn with_perimeter_walls(self) -> Self {
        let world_min_corner = self.min_corner();
        let world_max_corner = self.max_corner();
        self.with_influence(Box::new(WallCollisions::new(world_min_corner, world_max_corner)))
    }

    pub fn with_influence(mut self, influence: Box<Influence<T>>) -> Self {
        self.influences.push(influence);
        self
    }

    pub fn with_influences(mut self, mut influences: Vec<Box<Influence<T>>>) -> Self {
        self.influences.append(&mut influences);
        self
    }

    pub fn min_corner(&self) -> Position {
        self.min_corner
    }

    pub fn max_corner(&self) -> Position {
        self.max_corner
    }

    pub fn with_ball(mut self, ball: T) -> Self {
        self.add_ball(ball);
        self
    }

    pub fn with_balls(mut self, balls: Vec<T>) -> Self {
        for ball in balls {
            self.add_ball(ball);
        }
        self
    }

    pub fn add_ball(&mut self, ball: T) {
        self.ball_graph.add_node(ball);
    }

    pub fn balls(&self) -> &[T] {
        &self.ball_graph.unsorted_nodes()
    }

    pub fn with_bonds(mut self, index_pairs: Vec<(usize, usize)>) -> Self {
        for pair in index_pairs {
            let bond = Bond::new(&self.balls()[pair.0], &self.balls()[pair.1]);
            self.add_bond(bond);
        }
        self
    }

    pub fn add_bond(&mut self, bond: Bond) {
        self.ball_graph.add_edge(bond);
    }

    pub fn bonds(&self) -> &[Bond] {
        &self.ball_graph.edges()
    }

    pub fn with_angle_gussets(mut self, index_pairs_with_angles: Vec<(usize, usize, f64)>) -> Self {
        for tuple in index_pairs_with_angles {
            let gusset = AngleGusset::new(&self.bonds()[tuple.0], &self.bonds()[tuple.1], Angle::from_radians(tuple.2));
            self.add_angle_gusset(gusset);
        }
        self
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
        let mut world = create_world(vec![]);
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(1.0, 1.0)));
        world.tick();
        let ball = &world.balls()[0];
        assert!(ball.position().x() > 0.0);
        assert!(ball.position().y() > 0.0);
    }

    #[test]
    fn tick_with_force_accelerates_ball() {
        let mut world = create_world(vec![
            Box::new(SimpleForceInfluence::new(Box::new(ConstantForce::new(Force::new(1.0, 1.0)))))
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
        let mut world = create_world(vec![
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
        let mut world = create_world(vec![
            Box::new(SimpleForceInfluence::new(Box::new(ConstantForce::new(Force::new(1.0, 1.0)))))
        ]);
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        world.tick();
        let ball = &world.balls()[0];
        assert_eq!(Force::new(0.0, 0.0), ball.forces().net_force());
    }

    fn create_world(influences: Vec<Box<Influence<Ball>>>) -> World<Ball> {
        let world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.with_influences(influences)
    }
}
