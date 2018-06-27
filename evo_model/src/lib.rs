extern crate evo_view_model;

use evo_view_model::ViewModel;

pub mod physics;

pub fn tick(view_model: &mut ViewModel) {
    view_model.circle.x += 1.0;
    view_model.circle.y += 1.0;
}
