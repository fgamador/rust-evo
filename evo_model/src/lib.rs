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
//    bullseye.rings.push(to_bullseye_ring(cell, evo_view_model::Color::White));
//    bullseye.rings[1].outer_radius /= 2.0;
//    bullseye.rings.push(to_bullseye_ring(cell, evo_view_model::Color::Green));
//    bullseye.rings[2].outer_radius /= 4.0;
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
    use evo_view_model::Point;

    #[test]
    fn tick_creates_view_model_bullseye_for_each_cell() {
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
    fn tick_populates_view_model_bullseye_from_cell() {
        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.add_cell(Ball::new(Length::new(5.0), Mass::new(1.0),
                                 Position::new(2.0, -3.0), Velocity::new(0.0, 0.0)));
        let mut view_model = ViewModel::new();

        tick(&mut world, &mut view_model);

        let bullseye = &view_model.bullseyes[0];
        assert_eq!(1, bullseye.rings.len());

        let cell = &world.cells()[0];
        let ring = bullseye.rings[0];
        assert_eq!(ring.outer_radius, cell.radius().value());
        assert_eq!(ring.color, evo_view_model::Color::Green);
    }

    #[test]
    fn tick_clears_view_model_bullseyes_before_populating_them() {
        let mut view_model = ViewModel::new();
        view_model.bullseyes.push(evo_view_model::Bullseye::new(Point { x: 0.0, y: 0.0 }));

        let mut world = World::new(Position::new(0.0, 0.0), Position::new(0.0, 0.0));
        world.add_cell(Ball::new(Length::new(1.0), Mass::new(1.0),
                                 Position::new(0.0, 0.0), Velocity::new(0.0, 0.0)));

        tick(&mut world, &mut view_model);

        assert_eq!(1, view_model.bullseyes.len());
    }
}
