extern crate evo_view_model;

pub mod physics;
pub mod world;

use evo_view_model::ViewModel;
use physics::newtonian::Body;
use world::World;

pub fn tick(world: &mut World, view_model: &mut ViewModel) {
    world.tick();

    let ball = &world.balls()[0];
    let circle = &mut view_model.circle;
    circle.x = ball.position().x();
    circle.y = ball.position().y();
}
