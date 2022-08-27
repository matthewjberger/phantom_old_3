use super::WorldShader;
use crate::backend::opengl::{shader::ShaderProgram, texture::Texture};
use phantom_dependencies::{anyhow::Result, nalgebra_glm as glm};
use phantom_world::{Material, World};

pub struct BlinnPhongShader {
    shader_program: ShaderProgram,
}

impl BlinnPhongShader {
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

impl WorldShader for BlinnPhongShader {
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
        textures: &[Texture],
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.shader_program.set_uniform_vec4(
            "material.baseColorFactor",
            material.base_color_factor.as_slice(),
        );

        for (index, descriptor) in ["Diffuse", "Normal"].iter().enumerate() {
            let texture_index = match *descriptor {
                "Diffuse" => material.color_texture_index,
                "Normal" => material.normal_texture_index,
                // TODO: Give this a proper error
                _ => panic!("Failed to find index for texture type!"),
            };
            let has_texture = texture_index > -1;

            self.shader_program
                .set_uniform_bool(&format!("material.has{}Texture", *descriptor), has_texture);

            self.shader_program
                .set_uniform_int(&format!("{}Texture", *descriptor), index as _);

            if has_texture {
                textures[texture_index as usize].bind(index as _);
            }
        }

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

struct Material {
    vec4 baseColorFactor;
}; 

uniform Material material;

uniform sampler2D DiffuseTexture;
uniform sampler2D NormalTexture;

uniform vec3 cameraPosition;

in vec3 Position;
in vec2 UV0;
in vec3 Normal;
in vec3 Color0;

out vec4 color;

vec4 srgb_to_linear(vec4 srgbIn)
{
    return vec4(pow(srgbIn.xyz,vec3(2.2)),srgbIn.w);
}

const float PI = 3.14159265359;

void main(void)
{
    vec3 N = getNormal();

    color = material.baseColorFactor;
    if (material.hasDiffuseTexture) {
        vec4 albedoMap = texture(DiffuseTexture, UV0);
        color = srgb_to_linear(albedoMap);
    }
    color *= vec4(Color0, 1.0);

}
vec3 getNormal()
{
    if (!material.hasNormalTexture) {
        return Normal;
    }
    vec3 tangentNormal = texture(NormalTexture, UV0).xyz * 2.0 - 1.0;
    vec3 Q1  = dFdx(Position);
    vec3 Q2  = dFdy(Position);
    vec2 st1 = dFdx(UV0);
    vec2 st2 = dFdy(UV0);
    vec3 N   = normalize(Normal);
    vec3 T  = normalize(Q1*st2.t - Q2*st1.t);
    vec3 B  = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);
    return normalize(TBN * tangentNormal);
}
"#;
