#[cfg(all(feature = "winit", feature = "glium"))]
#[macro_use]
extern crate conrod;

#[cfg(all(feature = "winit", feature = "glium"))]
mod support;

pub fn main() {
    feature::main();
}

#[cfg(all(feature = "winit", feature = "glium"))]
mod feature {
    extern crate find_folder;

    use conrod;
    use conrod::backend::glium::glium;
    use conrod::backend::glium::glium::Surface;
    use support;

    // Generate a type that will produce a unique `widget::Id` for each widget.
    widget_ids! {
        struct Ids {
            canvas,
            circles[],
            moving_circle,
        }
    }

    pub fn main() {
        const WIDTH: u32 = 400;
        const HEIGHT: u32 = 400;

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Evo")
            .with_dimensions(WIDTH, HEIGHT);
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // A unique identifier for each widget.
        let mut ids = Ids::new(ui.widget_id_generator());

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used
        // for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        let mut moving_x = -150.0;
        let mut moving_y = -150.0;

        // Poll events from the window.
        let mut event_loop = support::EventLoop::new();
        'main: loop {

            // Handle all events.
            for event in event_loop.next(&mut events_loop) {

                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &display) {
                    ui.handle_event(event);
                    event_loop.needs_update();
                }

                if is_window_close(&event) {
                    break 'main;
                }
            }

            set_ui(ui.set_widgets(), &mut ids, moving_x, moving_y);

            // Render the `Ui` and then display it on the screen.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }

            moving_x += 1.0;
            moving_y += 1.0;
            event_loop.needs_update();
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

    fn set_ui(ref mut ui: conrod::UiCell, ids: &mut Ids, moving_x: f64, moving_y: f64) {
        use conrod::{Positionable, Widget};
        use conrod::color;
        use conrod::widget::{Canvas, Circle};

        // The background canvas upon which we'll place our widgets.
        Canvas::new().pad(80.0).set(ids.canvas, ui);

        Circle::fill_with(20.0, color::rgb(0.5, 1.0, 0.5))
            .x_y(moving_x, moving_y)
            .set(ids.moving_circle, ui);

        let mut walker = ids.circles.walk();
        let mut x = -100.0;
        let mut y = 100.0;
        for _i in 0..4 {
            let id = walker.next(&mut ids.circles, &mut ui.widget_id_generator());
            Circle::fill_with(20.0, color::rgb(0.5, 1.0, 0.5)).x_y(x, y).set(id, ui);
            x += 50.0;
            y -= 50.0;
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
