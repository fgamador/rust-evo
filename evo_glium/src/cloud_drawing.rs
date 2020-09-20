use glium::{implement_vertex, uniform, Surface};

#[derive(Clone, Copy)]
pub struct CloudSprite {
    pub center: [f32; 2],
    pub radius: f32,
    pub concentration: f32,
    pub color_index: u32,
}

implement_vertex!(CloudSprite, center, radius, concentration, color_index);

pub struct CloudDrawing {
    pub shader_program: glium::Program,
    pub indices: glium::index::NoIndices,
}

impl CloudDrawing {
    pub fn new(display: &glium::Display) -> Self {
        CloudDrawing {
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
        cloud_colors: [[f32; 4]; 8],
    ) where
        T: Copy,
    {
        let uniforms = uniform! {
            screen_transform: screen_transform,
            cloud_colors_0_3: [cloud_colors[0], cloud_colors[1], cloud_colors[2], cloud_colors[3]],
            cloud_colors_4_7: [cloud_colors[4], cloud_colors[5], cloud_colors[6], cloud_colors[7]],
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

        in vec2 center;
        in float radius;
        in float concentration;
        in uint color_index;

        out CloudSprite {
            vec2 center;
            float radius;
            float concentration;
            uint color_index;
        } cloud_out;

        void main() {
            cloud_out.center = center;
            cloud_out.radius = radius;
            cloud_out.concentration = concentration;
            cloud_out.color_index = color_index;
        }
    "#;

    const GEOMETRY_SHADER_SRC: &'static str = r#"
        #version 330 core

        uniform mat4 screen_transform;

        layout (points) in;
        layout (triangle_strip, max_vertices = 4) out;

        in CloudSprite {
            vec2 center;
            float radius;
            float concentration;
            uint color_index;
        } cloud_in[];

        out CloudPoint {
            vec2 offset;
            flat float radius;
            flat float concentration;
            flat uint color_index;
        } cloud_point_out;

        void emit_circle_bounding_box_corner(in vec2 center, in float radius, in vec2 corner) {
            vec2 offset = vec2(radius, radius) * corner;
            cloud_point_out.offset = offset;
            cloud_point_out.radius = radius;
            cloud_point_out.concentration = cloud_in[0].concentration;
            cloud_point_out.color_index = cloud_in[0].color_index;
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
            emit_circle_bounding_box(cloud_in[0].center, cloud_in[0].radius);
        }
    "#;

    const FRAGMENT_SHADER_SRC: &'static str = r#"
        #version 330 core

        uniform mat4 cloud_colors_0_3;
        uniform mat4 cloud_colors_4_7;

        in CloudPoint {
            vec2 offset;
            flat float radius;
            flat float concentration;
            flat uint color_index;
        } cloud_point_in;

        out vec4 color_out;

        float alpha_factor(in float radial_offset, in float cloud_radius, in float concentration) {
            return concentration * (1.0 - (radial_offset / cloud_radius));
        }

        void emit_color(in uint color_index, in float radial_offset, in float cloud_radius, in float concentration) {
            color_out = (color_index < 4u)
                ? cloud_colors_0_3[color_index]
                : cloud_colors_4_7[color_index - 4u];
            color_out.a *= alpha_factor(radial_offset, cloud_radius, concentration);
        }

        void main() {
            float radial_offset = length(cloud_point_in.offset);
            if (radial_offset <= cloud_point_in.radius) {
                emit_color(cloud_point_in.color_index, radial_offset, cloud_point_in.radius, cloud_point_in.concentration);
                return;
            }
            discard;
        }
    "#;
}
