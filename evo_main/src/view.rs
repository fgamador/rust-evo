use evo_glium::GliumView;
use evo_model::physics::quantities::*;
use evo_model::world::World;
use evo_model::UserAction;

pub struct View {
    view: GliumView,
}

impl View {
    pub fn new(world_min_corner: Position, world_max_corner: Position) -> Self {
        View {
            view: GliumView::new(
                [world_min_corner.x() as f32, world_min_corner.y() as f32],
                [world_max_corner.x() as f32, world_max_corner.y() as f32],
            ),
        }
    }

    pub fn check_for_user_action(&mut self) -> Option<UserAction> {
        self.view.check_for_user_action()
    }

    pub fn wait_for_user_action(&mut self) -> UserAction {
        self.view.wait_for_user_action()
    }

    pub fn render(&mut self, world: &World) {
        self.view.render(world);
    }
}
