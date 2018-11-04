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
        add_circle(view_model, ball);
    }
}

fn add_circle<T>(view_model: &mut ViewModel, ball: &T)
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    view_model.circles.push(evo_view_model::Circle {
        center: evo_view_model::Point {
            x: ball.center().x(),
            y: ball.center().y(),
        },
        radius: ball.radius().value(),
    });
}
