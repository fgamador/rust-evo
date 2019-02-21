use glium::Surface;

#[derive(Clone, Copy, Debug)]
pub struct World {
    pub corners: [f32; 4],
    pub top_color: [f32; 3],
    pub bottom_color: [f32; 3],
}

implement_vertex!(World, corners, top_color, bottom_color);

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

        in vec4 corners;
        in vec3 top_color;
        in vec3 bottom_color;

        out World {
            vec4 corners;
            vec3 top_color;
            vec3 bottom_color;
        } world_out;

        void main() {
            world_out.corners = corners;
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
            vec4 corners;
            vec3 top_color;
            vec3 bottom_color;
        } world_in[];

        out BackgroundPoint {
            vec3 color;
        } point_out;

        void emit_corner(in vec4 corners, in int x_index, in int y_index, in vec3 color) {
            point_out.color = color;
            gl_Position = screen_transform * vec4(corners[x_index], corners[y_index], 0.0, 1.0);
            EmitVertex();
        }

        void emit_quad(in vec4 corners, in vec3 top_color, in vec3 bottom_color) {
            emit_corner(corners, 0, 1, bottom_color);
            emit_corner(corners, 0, 3, top_color);
            emit_corner(corners, 2, 1, bottom_color);
            emit_corner(corners, 2, 3, top_color);
            EndPrimitive();
        }

        void main() {
            emit_quad(world_in[0].corners, world_in[0].top_color, world_in[0].bottom_color);
        }
    "#;

    const FRAGMENT_SHADER_SRC: &'static str = r#"
        #version 330 core

        in BackgroundPoint {
            vec3 color;
        } point_in;

        out vec4 color_out;

        void main() {
            color_out = vec4(point_in.color, 1.0);
        }
    "#;
}
