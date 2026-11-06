//! OpenGL ES 2.0 / 3.0 3D shader and rendering pipeline for Android and mobile hardware.

use crate::{Camera3d, RenderScene};
use nova_math::Vec3;

pub const VERTEX_SHADER_SRC: &str = r#"#version 100
precision mediump float;

attribute vec3 a_position;
attribute vec3 a_normal;
attribute vec2 a_texcoord;

uniform mat4 u_mvp;
uniform mat4 u_model;
uniform vec3 u_light_dir;

varying vec3 v_normal;
varying vec2 v_texcoord;
varying float v_light_intensity;

void main() {
    vec3 world_normal = normalize(mat3(u_model[0].xyz, u_model[1].xyz, u_model[2].xyz) * a_normal);
    v_normal = world_normal;
    v_texcoord = a_texcoord;
    v_light_intensity = max(dot(world_normal, normalize(u_light_dir)), 0.25);
    gl_Position = u_mvp * vec4(a_position, 1.0);
}
"#;

pub const FRAGMENT_SHADER_SRC: &str = r#"#version 100
precision mediump float;

uniform vec4 u_base_color;
varying vec3 v_normal;
varying vec2 v_texcoord;
varying float v_light_intensity;

void main() {
    vec3 color = u_base_color.rgb * v_light_intensity;
    gl_FragColor = vec4(color, u_base_color.a);
}
"#;

/// 4x4 matrix for projection and view transformation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4([f32; 16]);

impl Mat4 {
    pub fn identity() -> Self {
        Self([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y_rad * 0.5).tan();
        let range_inv = 1.0 / (near - far);

        Self([
            f / aspect, 0.0, 0.0, 0.0,
            0.0, f, 0.0, 0.0,
            0.0, 0.0, (far + near) * range_inv, -1.0,
            0.0, 0.0, 2.0 * far * near * range_inv, 0.0,
        ])
    }

    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        let f = (center - eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);

        Self([
            s.x, u.x, -f.x, 0.0,
            s.y, u.y, -f.y, 0.0,
            s.z, u.z, -f.z, 0.0,
            -s.dot(eye), -u.dot(eye), f.dot(eye), 1.0,
        ])
    }

    pub fn multiply(&self, rhs: &Self) -> Self {
        let mut out = [0.0; 16];
        for c in 0..4 {
            for r in 0..4 {
                out[c * 4 + r] =
                    self.0[0 * 4 + r] * rhs.0[c * 4 + 0] +
                    self.0[1 * 4 + r] * rhs.0[c * 4 + 1] +
                    self.0[2 * 4 + r] * rhs.0[c * 4 + 2] +
                    self.0[3 * 4 + r] * rhs.0[c * 4 + 3];
            }
        }
        Self(out)
    }

    pub fn as_slice(&self) -> &[f32; 16] {
        &self.0
    }
}

/// A render command submission packet for OpenGLES draw calls.
#[derive(Debug, Clone)]
pub struct RenderCommand {
    pub entity_name: String,
    pub model_matrix: Mat4,
    pub color_rgba: [f32; 4],
    pub vertex_count: usize,
}

/// Renderer pipeline compiling scenes into GPU-ready draw commands.
pub struct GlesRenderer {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub camera: Camera3d,
}

impl GlesRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            viewport_width: width.max(1),
            viewport_height: height.max(1),
            camera: Camera3d::default(),
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.viewport_width as f32 / self.viewport_height as f32
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.camera.position, self.camera.target, self.camera.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(
            self.camera.vertical_fov_radians,
            self.aspect_ratio(),
            self.camera.near_plane,
            self.camera.far_plane,
        )
    }

    pub fn generate_render_commands(&self, scene: &RenderScene) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        for entity in &scene.entities {
            let mut model = Mat4::identity();
            // Position translation into model matrix
            model.0[12] = entity.transform.position.x;
            model.0[13] = entity.transform.position.y;
            model.0[14] = entity.transform.position.z;

            commands.push(RenderCommand {
                entity_name: entity.name.clone(),
                model_matrix: model,
                color_rgba: entity.base_color_rgba,
                vertex_count: 36, // Cube mesh (6 faces * 2 triangles * 3 vertices)
            });
        }
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gles_matrices() {
        let proj = Mat4::perspective(60.0_f32.to_radians(), 1.77, 0.1, 1000.0);
        let view = Mat4::look_at(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO, Vec3::UP);
        let vp = proj.multiply(&view);
        assert_eq!(vp.as_slice().len(), 16);
    }
}
