use super::WorldShader;
use crate::backend::opengl::{shader::ShaderProgram, texture::Texture};
use phantom_dependencies::{anyhow::Result, nalgebra_glm as glm};
use phantom_world::{LightKind, Material, Transform, World};

#[derive(Default, Debug, Copy, Clone)]
pub struct Light {
    ambient: glm::Vec3,
    constant: f32,
    cutoff: f32,
    diffuse: glm::Vec3,
    direction: glm::Vec3,
    linear: f32,
    outer_cutoff: f32,
    position: glm::Vec3,
    quadratic: f32,
    specular: glm::Vec3,
    kind: i32,
}

impl Light {
    pub fn from_node(transform: &Transform, light: &phantom_world::BlinnPhongLight) -> Self {
        let mut inner_cone_cos: f32 = 0.0;
        let mut outer_cone_cos: f32 = 0.0;
        let kind = match light.kind {
            LightKind::Directional => 0,
            LightKind::Point => 1,
            LightKind::Spot {
                inner_cone_angle,
                outer_cone_angle,
            } => {
                inner_cone_cos = inner_cone_angle;
                outer_cone_cos = outer_cone_angle;
                2
            }
        };
        Self {
            ambient: light.ambient,
            constant: light.constant,
            diffuse: light.diffuse,
            direction: -1.0 * glm::quat_rotate_vec3(&transform.rotation, &glm::Vec3::z()),
            linear: light.linear,
            position: transform.translation,
            quadratic: light.quadratic,
            specular: light.specular,
            kind,
            cutoff: inner_cone_cos,
            outer_cutoff: outer_cone_cos,
        }
    }
}

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
    fn upload_lights(&self, world: &World) -> Result<()> {
        let world_lights = world
            .components::<phantom_world::BlinnPhongLight>()
            .unwrap()
            .iter()
            .map(|(transform, light)| Light::from_node(transform, light))
            .collect::<Vec<_>>();
        for (index, light) in world_lights.iter().enumerate() {
            let name = |key: &str| format!("lights[{}].{}", index, key);
            self.shader_program
                .set_uniform_vec3(&name("ambient"), light.ambient.as_slice());
            self.shader_program
                .set_uniform_float(&name("constant"), light.constant);
            self.shader_program
                .set_uniform_float(&name("cutoff"), light.cutoff);
            self.shader_program
                .set_uniform_vec3(&name("diffuse"), light.diffuse.as_slice());
            self.shader_program
                .set_uniform_vec3(&name("direction"), light.direction.as_slice());
            self.shader_program
                .set_uniform_float(&name("linear"), light.linear);
            self.shader_program
                .set_uniform_float(&name("outer_cutoff"), light.outer_cutoff);
            self.shader_program
                .set_uniform_vec3(&name("position"), light.position.as_slice());
            self.shader_program
                .set_uniform_float(&name("quadratic"), light.quadratic);
            self.shader_program
                .set_uniform_vec3(&name("specular"), light.specular.as_slice());
            self.shader_program
                .set_uniform_int(&name("kind"), light.kind);
        }
        self.shader_program
            .set_uniform_int("numberOfLights", world_lights.len() as _);
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
    bool hasDiffuseTexture;
    bool hasNormalTexture;
}; 

uniform Material material;

uniform sampler2D DiffuseTexture;
uniform sampler2D NormalTexture;

uniform vec3 cameraPosition;

struct Light {
    vec3 ambient;
    float constant;
    float cutoff;
    vec3 diffuse;
    vec3 direction;
    float linear;
    float outer_cutoff;
    vec3 position;
    float quadratic;
    vec3 specular;
    int kind;
};


#define MAX_NUMBER_OF_LIGHTS 4
uniform Light lights[MAX_NUMBER_OF_LIGHTS];
uniform int numberOfLights;

in vec3 Position;
in vec2 UV0;
in vec3 Normal;
in vec3 Color0;

out vec4 color;

vec4 srgb_to_linear(vec4 srgbIn)
{
    return vec4(pow(srgbIn.xyz,vec3(2.2)),srgbIn.w);
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
"#;
