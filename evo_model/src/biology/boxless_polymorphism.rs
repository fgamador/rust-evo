use std::f32::consts::PI;

pub trait Shapeness {
    fn area(&self) -> f32;
    fn resize(&mut self, factor: f32);
}

pub struct Rectangle {
    width: f32,
    height: f32,
}

impl Shapeness for Rectangle {
    fn area(&self) -> f32 {
        self.width * self.height
    }

    fn resize(&mut self, factor: f32) {
        self.width *= factor;
        self.height *= factor;
    }
}

pub struct Circle {
    radius: f32,
}

impl Shapeness for Circle {
    fn area(&self) -> f32 {
        PI * self.radius * self.radius
    }

    fn resize(&mut self, factor: f32) {
        self.radius *= factor;
    }
}

pub enum Shape {
    Rectangle(Rectangle),
    Circle(Circle),
}

impl Shape {
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self::Rectangle(Rectangle { width, height })
    }

    pub fn circle(radius: f32) -> Self {
        Self::Circle(Circle { radius })
    }

    pub fn shapeness(&self) -> &dyn Shapeness {
        match self {
            Self::Rectangle(shape) => shape,
            Self::Circle(shape) => shape,
        }
    }

    pub fn mut_shapeness(&mut self) -> &mut dyn Shapeness {
        match self {
            Self::Rectangle(shape) => shape,
            Self::Circle(shape) => shape,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangle_has_correct_area() {
        let subject = Shape::rectangle(2.0, 3.0);
        assert_eq!(subject.shapeness().area(), 6.0);
    }

    #[test]
    fn resized_circle_has_correct_area() {
        let mut subject = Shape::circle(1.0);
        subject.mut_shapeness().resize(2.0);
        assert_eq!(subject.shapeness().area(), 4.0 * PI);
    }
}
