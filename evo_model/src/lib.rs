extern crate evo_view_model;
#[macro_use]
extern crate evo_model_derive;

pub mod biology;
pub mod environment;
pub mod physics;
pub mod world;

use environment::environment::HasLocalEnvironment;
use evo_view_model::ViewModel;
use physics::newtonian::NewtonianBody;
use physics::shapes::Circle;
use physics::sortable_graph::GraphNode;
use world::World;

pub fn tick<T>(world: &mut World<T>, view_model: &mut ViewModel)
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    world.tick();

    view_model.circles.clear();

    for ball in world.balls() {
        view_model.circles.push(to_circle(ball));
    }
}

fn to_circle<T>(ball: &T) -> evo_view_model::Circle
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    evo_view_model::Circle {
        center: evo_view_model::Point {
            x: ball.center().x(),
            y: ball.center().y(),
        },
        radius: ball.radius().value(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics::ball::Ball;
    use physics::quantities::*;

    #[test]
    fn tick_creates_view_model_circle_for_each_ball() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        let mut view_model = ViewModel::new();

        tick(&mut world, &mut view_model);

        assert_eq!(2, view_model.circles.len());
    }

    #[test]
    fn tick_populates_view_model_circle_from_ball() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.add_ball(Ball::new(Length::new(5.0), Mass::new(1.0),
                                 Position::new(2.0, -3.0), Velocity::new(0.0, 0.0)));
        let mut view_model = ViewModel::new();

        tick(&mut world, &mut view_model);

        let ball = &world.balls()[0];
        let circle = view_model.circles[0];
        assert_eq!(circle.center.x, ball.center().x());
        assert_eq!(circle.center.y, ball.center().y());
        assert_eq!(circle.radius, ball.radius().value());
    }

    #[test]
    fn tick_clears_view_model_circles_before_populating_them() {
        let mut view_model = ViewModel::new();
        view_model.circles.push(evo_view_model::Circle {
            center: evo_view_model::Point { x: 0.0, y: 0.0 },
            radius: 1.0,
        });

        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.add_ball(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));

        tick(&mut world, &mut view_model);

        assert_eq!(1, view_model.circles.len());
    }
}
