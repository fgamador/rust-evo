#[cfg(all(feature = "winit", feature = "glium"))]
#[macro_use]
extern crate conrod;
extern crate evo_view_model;

#[cfg(all(feature = "winit", feature = "glium"))]
mod support;

pub fn main() {
    let mut view = feature::View::new();
    view.main();
}

#[cfg(all(feature = "winit", feature = "glium"))]
pub mod feature {
    extern crate find_folder;

    use evo_view_model::ViewModel;

    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::Surface;
    use support;

    // Generate a type that will produce a unique `widget::Id` for each widget.
    widget_ids! {
        struct Ids {
            canvas,
            circles[],
        }
    }

    pub struct View {
        display: glium::Display,
        events_loop: glium::glutin::EventsLoop,
        renderer: conrod::backend::glium::Renderer,
        ui: conrod::Ui,
        ids: Ids,
        image_map: conrod::image::Map<glium::texture::Texture2d>,
        event_loop: support::EventLoop,
        transform: CoordinateTransform,
    }

    impl View {
        const WIDTH: u32 = 400;
        const HEIGHT: u32 = 400;

        pub fn new() -> Self {
            let window = glium::glutin::WindowBuilder::new()
                .with_title("Evo")
                .with_dimensions(Self::WIDTH, Self::HEIGHT);
            let context = glium::glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_multisampling(4);
            let events_loop = glium::glutin::EventsLoop::new();
            let display = glium::Display::new(window, context, &events_loop).unwrap();
            let renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
            let mut ui = conrod::UiBuilder::new([Self::WIDTH as f64, Self::HEIGHT as f64]).build();
            let ids = Ids::new(ui.widget_id_generator());

            View {
                display,
                events_loop,
                renderer,
                ui,
                image_map: conrod::image::Map::<glium::texture::Texture2d>::new(),
                ids,
                event_loop: support::EventLoop::new(),
                transform: CoordinateTransform::new(),
            }
        }

        pub fn main(&mut self) {
            //while self.once() {}
        }

        pub fn once(&mut self, view_model: &ViewModel) -> bool {
            if !self.handle_events() {
                return false;
            }

            self.set_ui(view_model);

            self.render_and_display_ui();

            true
        }

        fn handle_events(&mut self) -> bool {
            self.event_loop.needs_update();
            for event in self.event_loop.next(&mut self.events_loop) {
                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &self.display) {
                    self.ui.handle_event(event);
                    self.event_loop.needs_update();
                }

                if is_window_close(&event) {
                    return false;
                }
            }
            true
        }

        fn set_ui(&mut self, view_model: &ViewModel) {
            use conrod::{Positionable, Widget};
            use conrod::color;
            use conrod::widget::{Canvas, Circle};

            let mut ui = self.ui.set_widgets();

            Canvas::new().pad(80.0).set(self.ids.canvas, &mut ui);

            let mut walker = self.ids.circles.walk();
            for circle in &view_model.circles {
                let id = walker.next(&mut self.ids.circles, &mut ui.widget_id_generator());
                Circle::fill_with(self.transform.transform_length(circle.radius),
                                  color::rgb(0.5, 1.0, 0.5))
                    .x(self.transform.transform_x(circle.x))
                    .y(self.transform.transform_y(circle.y))
                    .set(id, &mut ui);
            }
        }

        fn render_and_display_ui(&mut self) {
            if let Some(primitives) = self.ui.draw_if_changed() {
                self.renderer.fill(&self.display, primitives, &self.image_map);
                let mut target = self.display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                self.renderer.draw(&self.display, &mut target, &self.image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }

    fn is_window_close(event: &glium::glutin::Event) -> bool {
        match event {
            glium::glutin::Event::WindowEvent { event, .. } => match event {
                // Break from the loop upon `Escape`.
                glium::glutin::WindowEvent::Closed |
                glium::glutin::WindowEvent::KeyboardInput {
                    input: glium::glutin::KeyboardInput {
                        virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                } => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub struct CoordinateTransform {}

    impl CoordinateTransform {
        pub fn new() -> Self {
            CoordinateTransform {}
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
}

#[cfg(not(all(feature = "winit", feature = "glium")))]
mod feature {
    pub fn main() {
        println!("This example requires the `winit` and `glium` features. \
                 Try running `cargo run --release --features=\"winit glium\" --example <example_name>`");
    }
}
