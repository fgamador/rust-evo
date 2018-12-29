pub mod events;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Event {
    Rendered,
    Updated,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    Green,
    White,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub min_corner: Point,
    pub max_corner: Point,
}

#[derive(Debug)]
pub struct Bullseye {
    pub center: Point,
    pub rings: Vec<BullseyeRing>,
}

impl Bullseye {
    pub fn new(center: Point) -> Self {
        Bullseye {
            center,
            rings: Vec::with_capacity(8),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BullseyeRing {
    pub outer_radius: f64,
    pub color: Color,
    pub health: f64,
}

pub struct ViewModel {
    pub bullseyes: Vec<Bullseye>
}

impl ViewModel {
    pub fn new() -> Self {
        ViewModel {
            bullseyes: vec![]
        }
    }
}

pub struct CoordinateTransform {
    input_window: Rectangle,
    output_window: Rectangle,
    scaling: f64,
}

impl CoordinateTransform {
    pub fn new(input_window: Rectangle) -> Self {
        CoordinateTransform {
            input_window,
            output_window: input_window,
            scaling: 1.0,
        }
    }

    pub fn output_window(&self) -> Rectangle {
        self.output_window
    }

    pub fn set_output_window(&mut self, output_window: Rectangle) {
        self.output_window = output_window;
        self.scaling = self.calc_scale_x();
        if self.scaling != self.calc_scale_y() {
            // TODO just reduce scale until the longer dimension fits the window
            panic!("Transform does not scale by the same factor in x ({}) and y ({})",
                   self.scaling, self.calc_scale_y());
        }
    }

    fn calc_scale_x(&self) -> f64 {
        let input_width = self.input_window.max_corner.x - self.input_window.min_corner.x;
        let output_width = self.output_window.max_corner.x - self.output_window.min_corner.x;
        output_width / input_width
    }

    fn calc_scale_y(&self) -> f64 {
        let input_width = self.input_window.max_corner.y - self.input_window.min_corner.y;
        let output_width = self.output_window.max_corner.y - self.output_window.min_corner.y;
        output_width / input_width
    }

    pub fn transform_x(&self, input_x: f64) -> f64 {
        let input_delta_x = input_x - self.input_window.min_corner.x;
        self.output_window.min_corner.x + self.scaling * input_delta_x
    }

    pub fn transform_y(&self, input_y: f64) -> f64 {
        let input_delta_y = input_y - self.input_window.min_corner.y;
        self.output_window.min_corner.y + self.scaling * input_delta_y
    }

    pub fn transform_length(&self, input_length: f64) -> f64 {
        input_length * self.scaling
    }
}

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

    #[test]
    fn shift_coordinate_transform() {
        let input_window = rect((0.0, -20.0), (20.0, 0.0));
        let output_window = rect((-10.0, -10.0), (10.0, 10.0));
        let mut subject = CoordinateTransform::new(input_window);
        subject.set_output_window(output_window);
        assert_eq!(0.0, subject.transform_x(10.0));
        assert_eq!(0.0, subject.transform_y(-10.0));
        assert_eq!(1.0, subject.transform_length(1.0));
    }

    #[test]
    fn scale_coordinate_transform() {
        let input_window = rect((-10.0, -10.0), (10.0, 10.0));
        let output_window = rect((-20.0, -20.0), (20.0, 20.0));
        let mut subject = CoordinateTransform::new(input_window);
        subject.set_output_window(output_window);
        assert_eq!(2.0, subject.transform_x(1.0));
        assert_eq!(-2.0, subject.transform_y(-1.0));
        assert_eq!(2.0, subject.transform_length(1.0));
    }

    #[test]
    #[should_panic]
    fn unequal_scale_coordinate_transform() {
        let input_window = rect((-10.0, -10.0), (10.0, 10.0));
        let output_window = rect((-20.0, -21.0), (20.0, 21.0));
        let mut subject = CoordinateTransform::new(input_window);
        subject.set_output_window(output_window);
    }

    fn rect(min_corner: (f64, f64), max_corner: (f64, f64)) -> Rectangle {
        Rectangle {
            min_corner: Point { x: min_corner.0, y: min_corner.1 },
            max_corner: Point { x: max_corner.0, y: max_corner.1 },
        }
    }
}
