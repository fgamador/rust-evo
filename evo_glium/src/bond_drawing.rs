use glium::{implement_vertex, uniform, Surface};

#[derive(Clone, Copy)]
pub struct BondSprite {
    pub end1: [f32; 2],
    pub end2: [f32; 2],
    pub radius1: f32,
    pub radius2: f32,
}

implement_vertex!(BondSprite, end1, end2, radius1, radius2);

pub struct BondDrawing {
    pub shader_program: glium::Program,
    pub indices: glium::index::NoIndices,
}

impl BondDrawing {
    pub fn new(display: &glium::Display) -> Self {
        BondDrawing {
            shader_program: glium::Program::from_source(
                display,
                Self::VERTEX_SHADER_SRC,
                Self::FRAGMENT_SHADER_SRC,
                Some(Self::GEOMETRY_SHADER_SRC),
            )
            .unwrap(),
            indices: glium::index::NoIndices(glium::index::PrimitiveType::Points),
        }
    }

    pub fn draw<T>(
        &self,
        frame: &mut glium::Frame,
        vertex_buffer: &glium::VertexBuffer<T>,
        screen_transform: [[f32; 4]; 4],
        bond_color: [f32; 4],
    ) where
        T: Copy,
    {
        let uniforms = uniform! {
            screen_transform: screen_transform,
            bond_color: bond_color,
        };
        let params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };
        frame
            .draw(
                vertex_buffer,
                &self.indices,
                &self.shader_program,
                &uniforms,
                &params,
            )
            .unwrap();
    }

    const VERTEX_SHADER_SRC: &'static str = r#"
        #version 330 core

        in vec2 end1;
        in vec2 end2;
        in float radius1;
        in float radius2;

        out BondSprite {
            vec2 end1;
            vec2 end2;
            float radius1;
            float radius2;
        } bond_out;

        void main() {
            bond_out.end1 = end1;
            bond_out.end2 = end2;
            bond_out.radius1 = radius1;
            bond_out.radius2 = radius2;
        }
    "#;

    const GEOMETRY_SHADER_SRC: &'static str = r#"
        #version 330 core

        uniform mat4 screen_transform;

        layout (points) in;
        layout (triangle_strip, max_vertices = 4) out;

        in BondSprite {
            vec2 end1;
            vec2 end2;
            float radius1;
            float radius2;
        } bond_in[];

        out BondPoint {
            vec3 todo;
        } bond_point_out;

        void emit_corner(in vec2 corner) {
            gl_Position = screen_transform * vec4(corner[0], corner[1], 0.0, 1.0);
            EmitVertex();
        }

        void main() {
            vec2 bond_vec = bond_in[0].end2 - bond_in[0].end1;
            vec2 bond_vec_unit = bond_vec / length(bond_vec);
            vec2 bond_vec_unit_perp1 = vec2(bond_vec_unit[1], -bond_vec_unit[0]);
            vec2 bond_vec_unit_perp2 = vec2(-bond_vec_unit[1], bond_vec_unit[0]);

            emit_corner(bond_in[0].end1 + (bond_in[0].radius1 / 3) * bond_vec_unit_perp1);
            emit_corner(bond_in[0].end1 + (bond_in[0].radius1 / 3) * bond_vec_unit_perp2);
            emit_corner(bond_in[0].end2 + (bond_in[0].radius2 / 3) * bond_vec_unit_perp1);
            emit_corner(bond_in[0].end2 + (bond_in[0].radius2 / 3) * bond_vec_unit_perp2);

            EndPrimitive();
        }
    "#;

    const FRAGMENT_SHADER_SRC: &'static str = r#"
        #version 330 core

        uniform vec4 bond_color;

        in BondPoint {
            vec3 todo;
        } bond_point_in;

        out vec4 color_out;

        void main() {
            color_out = bond_color;
        }
    "#;
}
