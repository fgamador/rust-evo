#[cfg(all(feature = "winit", feature = "glium"))]
#[macro_use]
extern crate conrod;
extern crate evo_view_model;

#[cfg(all(feature = "winit", feature = "glium"))]
mod support;

#[cfg(all(feature = "winit", feature = "glium"))]
pub mod feature {
    extern crate find_folder;

    use evo_view_model;
    use evo_view_model::ViewModel;
    use evo_view_model::CoordinateTransform;

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

    pub struct ConrodView {
        display: glium::Display,
        events_loop: glium::glutin::EventsLoop,
        renderer: conrod::backend::glium::Renderer,
        ui: conrod::Ui,
        ids: Ids,
        image_map: conrod::image::Map<glium::texture::Texture2d>,
        event_loop: support::EventLoop,
        transform: CoordinateTransform,
    }

    impl ConrodView {
        pub fn new(mut transform: CoordinateTransform) -> Self {
            let (width, height) = Self::calc_width_and_height(transform.output_window());
            let window = glium::glutin::WindowBuilder::new()
                .with_title("Evo")
                .with_dimensions(width, height);
            let context = glium::glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_multisampling(4);
            let events_loop = glium::glutin::EventsLoop::new();
            let display = glium::Display::new(window, context, &events_loop).unwrap();
            let renderer = conrod::backend::glium::Renderer::new(&display).unwrap();
            let mut ui = conrod::UiBuilder::new([width as f64, height as f64]).build();
            let ids = Ids::new(ui.widget_id_generator());

            transform.set_output_window(Self::create_output_window(width, height));

            ConrodView {
                display,
                events_loop,
                renderer,
                ui,
                image_map: conrod::image::Map::<glium::texture::Texture2d>::new(),
                ids,
                event_loop: support::EventLoop::new(),
                transform,
            }
        }

        fn calc_width_and_height(rect: evo_view_model::Rectangle) -> (u32, u32) {
            let width = (rect.max_corner.x - rect.min_corner.x).floor() as u32;
            let height = (rect.max_corner.y - rect.min_corner.y).floor() as u32;
            (width, height)
        }

        fn create_output_window(width: u32, height: u32) -> evo_view_model::Rectangle {
            evo_view_model::Rectangle {
                min_corner: evo_view_model::Point {
                    x: -(width as f64) / 2.0,
                    y: -(height as f64) / 2.0,
                },
                max_corner: evo_view_model::Point {
                    x: (width as f64) / 2.0,
                    y: (height as f64) / 2.0,
                },
            }
        }

        pub fn once(&mut self, view_model: &ViewModel) -> bool {
            if !self.handle_events() {
                return false;
            }

            self.define_ui(view_model);
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

        fn define_ui(&mut self, view_model: &ViewModel) {
            use conrod::{Positionable, Widget};
            use conrod::widget::{Canvas, Circle};

            let mut ui = self.ui.set_widgets();

            Canvas::new().pad(80.0).set(self.ids.canvas, &mut ui);

            let mut walker = self.ids.circles.walk();
            for onion in &view_model.onions {
                for circle in &onion.concentric_circles {
                    let id = walker.next(&mut self.ids.circles, &mut ui.widget_id_generator());
                    Circle::fill_with(self.transform.transform_length(circle.radius),
                                      Self::lookup_rgb_color(circle.color))
                        .x(self.transform.transform_x(circle.center.x))
                        .y(self.transform.transform_y(circle.center.y))
                        .set(id, &mut ui);
                }
            }
        }

        fn lookup_rgb_color(color: evo_view_model::Color) -> conrod::color::Color {
            conrod::color::rgb(0.5, 1.0, 0.5)
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
}
