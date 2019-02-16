use glium::Surface;

#[derive(Clone, Copy)]
pub struct World {
    pub dimensions: [f32; 2],
    pub top_color: [f32; 3],
    pub bottom_color: [f32; 3],
}

implement_vertex!(World, dimensions, top_color, bottom_color);

pub struct BackgroundDrawing {
    pub shader_program: glium::Program,
    pub indices: glium::index::NoIndices,
}

impl BackgroundDrawing {
    pub fn new(display: &glium::Display) -> Self {
        BackgroundDrawing {
            shader_program: glium::Program::from_source(
                display, Self::VERTEX_SHADER_SRC, Self::FRAGMENT_SHADER_SRC,
                Some(Self::GEOMETRY_SHADER_SRC)).unwrap(),
            indices: glium::index::NoIndices(glium::index::PrimitiveType::Points),
        }
    }

    pub fn draw<T>(&self, frame: &mut glium::Frame, vertex_buffer: &glium::VertexBuffer<T>, screen_transform: [[f32; 4]; 4]) where T: Copy {
        let uniforms = uniform! {
            screen_transform: screen_transform
        };
        frame.draw(vertex_buffer, &self.indices, &self.shader_program, &uniforms, &Default::default()).unwrap();
    }

    const VERTEX_SHADER_SRC: &'static str = r#"
        #version 330 core

        in vec2 dimensions;
        in vec3 top_color;
        in vec3 bottom_color;

        out World {
            vec2 dimensions;
            vec3 top_color;
            vec3 bottom_color;
        } world_out;

        void main() {
            world_out.dimensions = dimensions;
            world_out.top_color = top_color;
            world_out.bottom_color = bottom_color;
        }
    "#;

    const GEOMETRY_SHADER_SRC: &'static str = r#"
        #version 330 core

        uniform mat4 screen_transform;

        layout (points) in;
        layout (triangle_strip, max_vertices = 4) out;

        in World {
            vec2 dimensions;
            vec3 top_color;
            vec3 bottom_color;
        } world_in[];

        out BackgroundPoint {
            vec3 color;
        } point_out;

        void emit_corner(in vec2 dimensions, in vec2 multiplier, in vec3 color) {
            point_out.color = color;
            gl_Position = screen_transform * vec4(dimensions * multiplier, 0.0, 1.0);
            EmitVertex();
        }

        void emit_quad(in vec2 dimensions, in vec3 top_color, in vec3 bottom_color) {
            emit_corner(dimensions, vec2(-0.5, -0.5), bottom_color);
            emit_corner(dimensions, vec2(-0.5, 0.5), top_color);
            emit_corner(dimensions, vec2(0.5, -0.5), bottom_color);
            emit_corner(dimensions, vec2(0.5, 0.5), top_color);
            EndPrimitive();
        }

        void main() {
            emit_quad(world_in[0].dimensions, world_in[0].top_color, world_in[0].bottom_color);
        }
    "#;

    const FRAGMENT_SHADER_SRC: &'static str = r#"
        #version 330 core

        in BackgroundPoint {
            vec3 color;
        } point_in;

        out vec4 fragment_color;

        void main() {
            fragment_color = vec4(point_in.color, 1.0);
        }
    "#;
}
