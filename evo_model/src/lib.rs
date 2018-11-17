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

pub fn tick<C>(world: &mut World<C>, view_model: &mut ViewModel)
    where C: Circle + GraphNode + HasLocalEnvironment + NewtonianBody + Onion
{
    world.tick();

    view_model.bullseyes.clear();

    for cell in world.cells() {
        view_model.bullseyes.push(to_bullseye(cell));
    }
}

fn to_bullseye<C>(cell: &C) -> evo_view_model::Bullseye
    where C: Circle + Onion
{
    let center = evo_view_model::Point {
        x: cell.center().x(),
        y: cell.center().y(),
    };
    let mut bullseye = evo_view_model::Bullseye::new(center);
    bullseye.rings.push(to_bullseye_ring(cell, evo_view_model::Color::Green));
//    onion.rings.push(to_bullseye_ring(cell, evo_view_model::Color::White));
//    onion.rings[1].radius /= 2.0;
//    onion.rings.push(to_bullseye_ring(cell, evo_view_model::Color::Green));
//    onion.rings[2].radius /= 4.0;
    bullseye
}

fn to_bullseye_ring(circle: &Circle, color: evo_view_model::Color) -> evo_view_model::BullseyeRing {
    evo_view_model::BullseyeRing {
        outer_radius: circle.radius().value(),
        color,
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

        assert_eq!(2, view_model.bullseyes.len());
    }

    #[test]
    fn tick_populates_view_model_onion_from_ball() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.add_cell(Ball::new(Length::new(5.0), Mass::new(1.0),
                                 Position::new(2.0, -3.0), Velocity::new(0.0, 0.0)));
        let mut view_model = ViewModel::new();

        tick(&mut world, &mut view_model);

        let onion = &view_model.bullseyes[0];
        assert_eq!(1, onion.rings.len());

        let ball = &world.cells()[0];
        let circle = onion.rings[0];
        assert_eq!(circle.color, evo_view_model::Color::Green);
        assert_eq!(circle.center.x, ball.center().x());
        assert_eq!(circle.center.y, ball.center().y());
        assert_eq!(circle.outer_radius, ball.radius().value());
    }

    #[test]
    fn tick_clears_view_model_onions_before_populating_them() {
        let mut view_model = ViewModel::new();
        view_model.bullseyes.push(evo_view_model::Bullseye::new());

        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.add_cell(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));

        tick(&mut world, &mut view_model);

        assert_eq!(1, view_model.bullseyes.len());
    }
}
