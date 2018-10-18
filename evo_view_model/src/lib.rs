pub mod events;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Event {
    Rendered,
    Updated,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub min_corner: Point,
    pub max_corner: Point,
}

pub struct ViewModel {
    pub circles: Vec<Circle>,
}

impl ViewModel {
    pub fn new() -> Self {
        ViewModel {
            circles: vec![],
        }
    }
}

pub struct CoordinateTransform {
    input_window: Rectangle,
    output_window: Rectangle,
}

impl CoordinateTransform {
    pub fn new(input_window: Rectangle) -> Self {
        CoordinateTransform {
            input_window,
            output_window: input_window,
        }
    }

    pub fn set_output_window(&mut self, output_window: Rectangle) {
        self.output_window = output_window;
    }

    pub fn transform_x(&self, input_x: f64) -> f64 {
        input_x
    }

    pub fn transform_y(&self, input_y: f64) -> f64 {
        input_y
    }

    pub fn transform_length(&self, input_length: f64) -> f64 {
        input_length
    }
}

//pub struct CoordinateTransform {
//    input_window: Rectangle,
//    output_window: Rectangle,
//    scaling: f64,
//}
//
//impl CoordinateTransform {
//    pub fn new(input_window: Rectangle, output_window: Rectangle) -> Self {
//        let mut transform = CoordinateTransform {
//            input_window,
//            output_window,
//            scaling: 0.0,
//        };
//        transform.scaling = transform.calc_scale_x();
//        if transform.scaling != transform.calc_scale_y() {
//            panic!("Transform does not scale by the same factor in x ({}) and y ({})",
//                   transform.scaling, transform.calc_scale_y());
//        }
//        transform
//    }
//
//    fn calc_scale_x(&self) -> f64 {
//        let input_width = self.input_window.max_corner().x() - self.input_window.min_corner().x();
//        let output_width = self.output_window.max_corner().x() - self.output_window.min_corner().x();
//        output_width / input_width
//    }
//
//    fn calc_scale_y(&self) -> f64 {
//        let input_height = self.input_window.max_corner().y() - self.input_window.min_corner().y();
//        let output_height = self.output_window.max_corner().y() - self.output_window.min_corner().y();
//        output_height / input_height
//    }
//
//    pub fn transform_position(&self, input_position: Position) -> Position {
//        let x = self.transform_x(input_position.x());
//        let y = self.transform_y(input_position.y());
//        Position::new(x, y)
//    }
//
//    fn transform_x(&self, input_x: f64) -> f64 {
//        let input_delta_x = input_x - self.input_window.min_corner().x();
//        self.output_window.min_corner().x() + self.scaling * input_delta_x
//    }
//
//    fn transform_y(&self, input_y: f64) -> f64 {
//        let input_delta_y = input_y - self.input_window.min_corner().y();
//        self.output_window.min_corner().y() + self.scaling * input_delta_y
//    }
//
//    pub fn transform_length(&self, len: Length) -> Length {
//        len * self.scaling
//    }
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_coordinate_transform() {
        let window = rect((-10.0, -10.0), (10.0, 10.0));
        let mut subject = CoordinateTransform::new(window);
        subject.set_output_window(window);
        assert_eq!(1.0, subject.transform_x(1.0));
        assert_eq!(1.0, subject.transform_length(1.0));
    }

//    #[test]
    fn shift_coordinate_transform() {
        let input_window = rect((0.0, -20.0), (20.0, 0.0));
        let output_window = rect((-10.0, -10.0), (10.0, 10.0));
        let mut subject = CoordinateTransform::new(input_window);
        subject.set_output_window(output_window);
        assert_eq!(0.0, subject.transform_x(10.0));
        assert_eq!(0.0, subject.transform_y(-10.0));
        assert_eq!(1.0, subject.transform_length(1.0));
    }

//    #[test]
//    fn scale_coordinate_transform() {
//        let input_window = rect((-10.0, -10.0), (10.0, 10.0));
//        let output_window = rect((-20.0, -20.0), (20.0, 20.0));
//        let mut subject = CoordinateTransform::new(input_window);
//        subject.set_output_window(output_window);
//        assert_eq!(Position::new(2.0, -2.0), subject.transform_position(Position::new(1.0, -1.0)));
//        assert_eq!(2.0, subject.transform_length(1.0));
//    }

//    #[test]
//    #[should_panic]
//    fn unequal_scale_coordinate_transform() {
//        let input_window = rect((-10.0, -10.0), (10.0, 10.0));
//        let output_window = rect((-20.0, -21.0), (20.0, 21.0));
//        let mut subject = CoordinateTransform::new(input_window);
//        subject.set_output_window(output_window);
//    }

    fn rect(min_corner: (f64, f64), max_corner: (f64, f64)) -> Rectangle {
        Rectangle {
            min_corner: Point { x: min_corner.0, y: min_corner.1 },
            max_corner: Point { x: max_corner.0, y: max_corner.1 },
        }
    }
}
