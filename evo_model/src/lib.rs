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
use physics::shapes::*;
use physics::sortable_graph::GraphNode;
use world::World;

pub fn tick<T>(world: &mut World<T>, view_model: &mut ViewModel)
    where T: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion
{
    world.tick();

    view_model.onions.clear();

    for cell in world.cells() {
        view_model.onions.push(to_onion(cell));
    }
}

fn to_onion<T>(cell: &T) -> evo_view_model::Onion
    where T: Circle + Onion
{
    let mut onion = evo_view_model::Onion::new();
    onion.concentric_circles.push(to_view_model_circle(cell, evo_view_model::Color::Green));
//    onion.concentric_circles.push(to_view_model_circle(cell, evo_view_model::Color::White));
//    onion.concentric_circles[1].radius /= 2.0;
//    onion.concentric_circles.push(to_view_model_circle(cell, evo_view_model::Color::Green));
//    onion.concentric_circles[2].radius /= 4.0;
    onion
}

fn to_view_model_circle(circle: &Circle, color: evo_view_model::Color) -> evo_view_model::Circle {
    evo_view_model::Circle {
        color,
        center: evo_view_model::Point {
            x: circle.center().x(),
            y: circle.center().y(),
        },
        radius: circle.radius().value(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use physics::ball::Ball;
    use physics::quantities::*;

    #[test]
    fn tick_creates_view_model_onion_for_each_ball() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.add_cell(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        world.add_cell(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));
        let mut view_model = ViewModel::new();

        tick(&mut world, &mut view_model);

        assert_eq!(2, view_model.onions.len());
    }

    #[test]
    fn tick_populates_view_model_onion_from_ball() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.add_cell(Ball::new(Length::new(5.0), Mass::new(1.0),
                                 Position::new(2.0, -3.0), Velocity::new(0.0, 0.0)));
        let mut view_model = ViewModel::new();

        tick(&mut world, &mut view_model);

        let onion = &view_model.onions[0];
        assert_eq!(1, onion.concentric_circles.len());

        let ball = &world.cells()[0];
        let circle = onion.concentric_circles[0];
        assert_eq!(circle.color, evo_view_model::Color::Green);
        assert_eq!(circle.center.x, ball.center().x());
        assert_eq!(circle.center.y, ball.center().y());
        assert_eq!(circle.radius, ball.radius().value());
    }

    #[test]
    fn tick_clears_view_model_onions_before_populating_them() {
        let mut view_model = ViewModel::new();
        view_model.onions.push(evo_view_model::Onion::new());

        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.add_cell(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));

        tick(&mut world, &mut view_model);

        assert_eq!(1, view_model.onions.len());
    }
}
