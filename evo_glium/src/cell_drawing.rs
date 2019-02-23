use glium::Surface;

#[derive(Clone, Copy)]
pub struct Cell {
    pub center: [f32; 2],
    pub num_radii: u32,
    pub radii_0_3: [f32; 4],
    pub radii_4_7: [f32; 4],
    pub health_0_3: [f32; 4],
    pub health_4_7: [f32; 4],
}

implement_vertex!(Cell, center, num_radii, radii_0_3, radii_4_7);

pub struct CellDrawing {
    pub shader_program: glium::Program,
    pub indices: glium::index::NoIndices,
}

impl CellDrawing {
    pub fn new(display: &glium::Display) -> Self {
        CellDrawing {
            shader_program: glium::Program::from_source(
                display, Self::VERTEX_SHADER_SRC, Self::FRAGMENT_SHADER_SRC,
                Some(Self::GEOMETRY_SHADER_SRC)).unwrap(),
            indices: glium::index::NoIndices(glium::index::PrimitiveType::Points),
        }
    }

    pub fn draw<T>(&self, frame: &mut glium::Frame, vertex_buffer: &glium::VertexBuffer<T>,
                   screen_transform: [[f32; 4]; 4], layer_colors: [[f32; 4]; 8]) where T: Copy
    {
        let uniforms = uniform! {
            screen_transform: screen_transform,
            layer_colors_0_3: [layer_colors[0], layer_colors[1], layer_colors[2], layer_colors[3]],
            layer_colors_4_7: [layer_colors[4], layer_colors[5], layer_colors[6], layer_colors[7]],
        };
        frame.draw(vertex_buffer, &self.indices, &self.shader_program, &uniforms, &Default::default()).unwrap();
    }

    const VERTEX_SHADER_SRC: &'static str = r#"
        #version 330 core

        in vec2 center;
        in uint num_radii;
        in vec4 radii_0_3;
        in vec4 radii_4_7;

        out Cell {
            vec2 center;
            uint num_radii;
            float radii[8];
        } cell_out;

        void main() {
            cell_out.center = center;
            cell_out.num_radii = num_radii;
            cell_out.radii = float[](radii_0_3[0], radii_0_3[1], radii_0_3[2], radii_0_3[3],
                                     radii_4_7[0], radii_4_7[1], radii_4_7[2], radii_4_7[3]);
        }
    "#;

    const GEOMETRY_SHADER_SRC: &'static str = r#"
        #version 330 core

        uniform mat4 screen_transform;

        layout (points) in;
        layout (triangle_strip, max_vertices = 4) out;

        in Cell {
            vec2 center;
            uint num_radii;
            float radii[8];
        } cell_in[];

        out CellPoint {
            vec2 offset;
            flat uint num_radii;
            flat float radii[8];
        } cell_point_out;

        void emit_circle_bounding_box_corner(in vec2 center, in float radius, in vec2 corner) {
            vec2 offset = vec2(radius, radius) * corner;
            cell_point_out.offset = offset;
            cell_point_out.num_radii = cell_in[0].num_radii;
            cell_point_out.radii = cell_in[0].radii;
            gl_Position = screen_transform * vec4(center + offset, 0.0, 1.0);
            EmitVertex();
        }

        void emit_circle_bounding_box(in vec2 center, in float radius) {
            emit_circle_bounding_box_corner(center, radius, vec2(-1.0, -1.0));
            emit_circle_bounding_box_corner(center, radius, vec2(-1.0, 1.0));
            emit_circle_bounding_box_corner(center, radius, vec2(1.0, -1.0));
            emit_circle_bounding_box_corner(center, radius, vec2(1.0, 1.0));
            EndPrimitive();
        }

        void main() {
            uint num_radii = cell_in[0].num_radii;
            float radius = cell_in[0].radii[num_radii - 1u];
            emit_circle_bounding_box(cell_in[0].center, radius);
        }
    "#;

    const FRAGMENT_SHADER_SRC: &'static str = r#"
        #version 330 core

        uniform mat4 layer_colors_0_3;
        uniform mat4 layer_colors_4_7;

        in CellPoint {
            vec2 offset;
            flat uint num_radii;
            flat float radii[8];
        } cell_point_in;

        out vec4 color_out;

        void emit_color(in uint layer_index) {
            color_out = (layer_index < 4u)
                ? layer_colors_0_3[layer_index]
                : layer_colors_4_7[layer_index - 4u];
        }

        void main() {
            float dist = sqrt(dot(cell_point_in.offset, cell_point_in.offset));
            for (uint i = 0u; i < min(8u, cell_point_in.num_radii); ++i) {
                if (dist <= cell_point_in.radii[i]) {
                    emit_color(i);
                    return;
                }
            }
            discard;
        }
    "#;
}
