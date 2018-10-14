use evo_conrod;
use evo_model;
use evo_model::environment::environment::*;
use evo_model::physics::newtonian::NewtonianBody;
use evo_model::physics::quantities::*;
use evo_model::physics::shapes::*;
use evo_model::physics::sortable_graph::*;
use evo_model::world::World;
use evo_view_model::ViewModel;
use std::thread;
use std::time::{Duration, Instant};

pub struct MVVM<T>(pub Model<T>, pub View, pub ViewModel)
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment;

pub struct Model<T>
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    world: World<T>,
}

impl<T> Model<T>
    where T: Circle + GraphNode + NewtonianBody + HasLocalEnvironment
{
    pub fn new(world: World<T>) -> Self {
        Model {
            world
        }
    }

    pub fn tick(&mut self, view_model: &mut ViewModel) {
        evo_model::tick(&mut self.world, view_model);
    }
}

pub struct View {
    view: evo_conrod::feature::View,
    next_tick: Instant,
}

impl View {
    pub fn new() -> Self {
        View {
            view: evo_conrod::feature::View::new(),
            next_tick: Instant::now(),
        }
    }

    pub fn render(&mut self, view_model: &mut ViewModel) -> bool {
        self.await_next_tick();
        self.view.once(view_model)
    }

    fn await_next_tick(&mut self) {
        let now = Instant::now();
        if now < self.next_tick {
            thread::sleep(self.next_tick - now);
        }
        self.next_tick += Duration::from_millis(16);
    }
}

pub struct CoordinateTransform {
    input_window: Rectangle,
    output_window: Rectangle,
}

impl CoordinateTransform {
    pub fn new(input_window: Rectangle, output_window: Rectangle) -> Self {
        CoordinateTransform {
            input_window,
            output_window,
        }
    }

    pub fn transform_position(&self, input_position: Position) -> Position {
        let x = self.transform_x(input_position.x());
        let y = self.transform_y(input_position.y());
        Position::new(x, y)
    }

    fn transform_x(&self, input_x: f64) -> f64 {
        let input_delta_x = input_x - self.input_window.min_corner().x();
        self.output_window.min_corner().x() + input_delta_x
    }

    fn transform_y(&self, input_y: f64) -> f64 {
        let input_delta_y = input_y - self.input_window.min_corner().y();
        self.output_window.min_corner().y() + input_delta_y
    }

    pub fn transform_length(&self, len: Length) -> Length {
        len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_coordinate_transform() {
        let window = Rectangle::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        let subject = CoordinateTransform::new(window, window);
        assert_eq!(Position::new(1.0, 1.0), subject.transform_position(Position::new(1.0, 1.0)));
        assert_eq!(Length::new(1.0), subject.transform_length(Length::new(1.0)));
    }

    #[test]
    fn shift_coordinate_transform() {
        let input_window = Rectangle::new(Position::new(0.0, -20.0), Position::new(20.0, 0.0));
        let output_window = Rectangle::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        let subject = CoordinateTransform::new(input_window, output_window);
        assert_eq!(Position::new(0.0, 0.0), subject.transform_position(Position::new(10.0, -10.0)));
        assert_eq!(Length::new(1.0), subject.transform_length(Length::new(1.0)));
    }

    //#[test]
    fn scale_coordinate_transform() {
        let input_window = Rectangle::new(Position::new(-10.0, -10.0), Position::new(10.0, 10.0));
        let output_window = Rectangle::new(Position::new(-20.0, -20.0), Position::new(20.0, 20.0));
        let subject = CoordinateTransform::new(input_window, output_window);
        assert_eq!(Position::new(2.0, -2.0), subject.transform_position(Position::new(1.0, -1.0)));
        assert_eq!(Length::new(2.0), subject.transform_length(Length::new(1.0)));
    }
}
