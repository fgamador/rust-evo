use evo_model::world::World;
use evo_view_model::ViewModel;
use mvvm::*;

pub fn init_and_run(world: World) {
    let view = View::new(world.min_corner(), world.max_corner());
    let mvvm = MVVM(Model::new(world), view, ViewModel::new());
    run(mvvm);
}

fn run(mut mvvm: MVVM) {
    let mut done = false;
    while !done {
        let MVVM(ref mut model, ref mut view, ref mut view_model) = mvvm;
        model.tick(view_model);
        done = !view.render(&model.world, view_model);
    }
}
