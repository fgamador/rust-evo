extern crate evo_view_model;

pub mod environment;
pub mod physics;
pub mod world;

use environment::environment::WithLocalEnvironment;
use evo_view_model::ViewModel;
use physics::newtonian::NewtonianBody;
use physics::shapes::Circle;
use physics::sortable_graph::GraphNode;
use world::World;

pub fn tick<T>(world: &mut World<T>, view_model: &mut ViewModel)
    where T: Circle + GraphNode + NewtonianBody + WithLocalEnvironment
{
    world.tick();

    view_model.circles.clear();

    for ball in world.balls() {
        view_model.circles.push(evo_view_model::Circle {
            x: ball.center().x(),
            y: ball.center().y(),
            radius: ball.radius().value(),
        });
    }
}
