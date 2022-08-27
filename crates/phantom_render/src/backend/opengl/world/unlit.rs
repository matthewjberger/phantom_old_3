use super::WorldShader;
use crate::backend::opengl::{shader::ShaderProgram, texture::Texture};
use phantom_dependencies::{anyhow::Result, nalgebra_glm as glm};
use phantom_world::{Material, World};

pub struct UnlitShader {
    shader_program: ShaderProgram,
}

impl UnlitShader {
    pub fn new() -> Result<Self> {
        let mut shader_program = ShaderProgram::new();
        shader_program
            .vertex_shader_source(VERTEX_SHADER_SOURCE)?
            .fragment_shader_source(FRAGMENT_SHADER_SOURCE)?
            .link();
        Ok(Self { shader_program })
    }

    fn update_uniforms(&self, world: &World, aspect_ratio: f32) -> Result<()> {
        let (projection, view) = world.active_camera_matrices(aspect_ratio).unwrap();
        let camera_entity = world.active_camera().unwrap();
        let camera_transform = world.entity_global_transform(camera_entity).unwrap();
        self.shader_program
            .set_uniform_vec3("cameraPosition", camera_transform.translation.as_slice());
        self.shader_program
            .set_uniform_matrix4x4("projection", projection.as_slice());
        self.shader_program
            .set_uniform_matrix4x4("view", view.as_slice());
        Ok(())
    }
}

impl WorldShader for UnlitShader {
    fn use_program(&self) {
        self.shader_program.use_program();
    }

    fn update(&self, world: &World, aspect_ratio: f32) -> Result<(), Box<dyn std::error::Error>> {
        self.update_uniforms(world, aspect_ratio)?;
        Ok(())
    }

    fn update_model_matrix(
        &self,
        model_matrix: glm::Mat4,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.shader_program
            .set_uniform_matrix4x4("model", model_matrix.as_slice());
        Ok(())
    }

    fn update_material(
        &self,
        material: &Material,
        _textures: &[Texture],
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.shader_program.set_uniform_vec4(
            "material.baseColorFactor",
            material.base_color_factor.as_slice(),
        );
        Ok(())
    }
}

const VERTEX_SHADER_SOURCE: &'static str = &r#"
#version 450 core

layout (location = 0) in vec3 inPosition;
layout (location = 1) in vec3 inNormal;
layout (location = 2) in vec2 inUV0;
layout (location = 3) in vec2 inUV1;
layout (location = 4) in vec4 inJoint0;
layout (location = 5) in vec4 inWeight0;
layout (location = 6) in vec3 inColor0;

uniform mat4 view;
uniform mat4 projection;
uniform mat4 model;

out vec3 Position;
out vec2 UV0;
out vec3 Normal;
out vec3 Color0;

void main()
{
   Position = vec3(model * vec4(inPosition, 1.0));
   gl_Position = projection * view * vec4(Position, 1.0);
   UV0 = inUV0;
   Normal = mat3(model) * inNormal;
   Color0 = inColor0;
}
"#;

const FRAGMENT_SHADER_SOURCE: &'static str = &r#"
#version 450 core

uniform vec3 cameraPosition;

struct Material {
    vec4 baseColorFactor;
};

uniform Material material;

in vec3 Position;
in vec2 UV0;
in vec3 Normal;
in vec3 Color0;

out vec4 color;

vec3 getNormal();


void main(void)
{
    color = material.baseColorFactor;
    color *= vec4(Color0, 1.0);

}
"#;
