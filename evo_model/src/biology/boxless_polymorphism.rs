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

pub enum ShapeEnum {
    RectangleItem(Rectangle),
    CircleItem(Circle),
}

pub struct Shape {
    shape_enum: ShapeEnum,
}

impl Shape {
    pub fn rectangle(width: f32, height: f32) -> Self {
        Shape {
            shape_enum: ShapeEnum::RectangleItem(Rectangle { width, height }),
        }
    }

    pub fn circle(radius: f32) -> Self {
        Shape {
            shape_enum: ShapeEnum::CircleItem(Circle { radius }),
        }
    }
}

impl Shapeness for Shape {
    fn area(&self) -> f32 {
        match &self.shape_enum {
            ShapeEnum::RectangleItem(shape) => shape.area(),
            ShapeEnum::CircleItem(shape) => shape.area(),
        }
    }

    fn resize(&mut self, factor: f32) {
        match &mut self.shape_enum {
            ShapeEnum::RectangleItem(shape) => shape.resize(factor),
            ShapeEnum::CircleItem(shape) => shape.resize(factor),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangle_has_correct_area() {
        let subject = Shape::rectangle(2.0, 3.0);
        assert_eq!(subject.area(), 6.0);
    }

    #[test]
    fn resized_circle_has_correct_area() {
        let mut subject = Shape::circle(1.0);
        subject.resize(2.0);
        assert_eq!(subject.area(), 4.0 * PI);
    }
}
