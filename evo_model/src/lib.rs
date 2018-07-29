extern crate evo_view_model;

pub mod physics;
pub mod world;

use evo_view_model::ViewModel;
use physics::shapes::Circle;
use world::World;

pub fn tick(world: &mut World, view_model: &mut ViewModel) {
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
