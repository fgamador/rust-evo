extern crate evo_view_model;

use evo_view_model::ViewModel;

pub mod physics;
pub mod world;

pub fn tick(view_model: &mut ViewModel) {
    let circle = &mut view_model.circle;
    circle.x += 1.0;
    circle.y += 1.0;
    if circle.x > 100.0 {
        circle.x = -100.0;
        circle.y = -100.0;
    }
}
