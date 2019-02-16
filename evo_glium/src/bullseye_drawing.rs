use glium::Surface;

#[derive(Clone, Copy)]
pub struct Bullseye {
    pub center: [f32; 2],
    pub radius: f32,
}

implement_vertex!(Bullseye, center, radius);

pub struct BullseyeDrawing {
    pub shader_program: glium::Program,
    pub indices: glium::index::NoIndices,
}

impl BullseyeDrawing {
    pub fn new(display: &glium::Display) -> Self {
        BullseyeDrawing {
            shader_program: glium::Program::from_source(
                display, Self::VERTEX_SHADER_SRC, Self::FRAGMENT_SHADER_SRC,
                Some(Self::GEOMETRY_SHADER_SRC)).unwrap(),
            indices: glium::index::NoIndices(glium::index::PrimitiveType::Points),
        }
    }

    pub fn draw<T>(&self, frame: &mut glium::Frame, vertex_buffer: &glium::VertexBuffer<T>, screen_transform: [[f32; 4]; 4]) where T: Copy {
        let uniforms = uniform! {
            circle_color: [0.85_f32, 0.1_f32, 0.1_f32, 1.0_f32],
            screen_transform: screen_transform
        };
        frame.draw(vertex_buffer, &self.indices, &self.shader_program, &uniforms, &Default::default()).unwrap();
    }

    const VERTEX_SHADER_SRC: &'static str = r#"
        #version 330 core

        in vec2 center;
        in float radius;

        out Circle {
            vec2 center;
            float radius;
        } circle_out;

        void main() {
            circle_out.center = center;
            circle_out.radius = radius;
        }
    "#;

    const GEOMETRY_SHADER_SRC: &'static str = r#"
        #version 330 core

        uniform mat4 screen_transform;

        layout (points) in;
        layout (triangle_strip, max_vertices = 4) out;

        in Circle {
            vec2 center;
            float radius;
        } circle_in[];

        out CirclePoint {
            vec2 offset;
            float radius;
        } circle_point_out;

        void emit_circle_bounding_box_corner(in vec2 center, in float radius, in vec2 corner) {
            vec2 offset = vec2(radius, radius) * corner;
            circle_point_out.offset = offset;
            gl_Position = screen_transform * vec4(center + offset, 0.0, 1.0);
            EmitVertex();
        }

        void emit_circle_bounding_box(in vec2 center, in float radius) {
            circle_point_out.radius = radius;
            emit_circle_bounding_box_corner(center, radius, vec2(-1.0, -1.0));
            emit_circle_bounding_box_corner(center, radius, vec2(-1.0, 1.0));
            emit_circle_bounding_box_corner(center, radius, vec2(1.0, -1.0));
            emit_circle_bounding_box_corner(center, radius, vec2(1.0, 1.0));
            EndPrimitive();
        }

        void main() {
            emit_circle_bounding_box(circle_in[0].center, circle_in[0].radius);
        }
    "#;

    const FRAGMENT_SHADER_SRC: &'static str = r#"
        #version 330 core

        uniform vec4 circle_color;

        in CirclePoint {
            vec2 offset;
            float radius;
        } circle_point_in;

        out vec4 color;

        void main() {
            float dist = sqrt(dot(circle_point_in.offset, circle_point_in.offset));
            if (dist <= circle_point_in.radius / 2.0)
                color = circle_color / 2.0;
            else if (dist <= circle_point_in.radius)
                color = circle_color;
            else
                discard;
        }
    "#;
}
