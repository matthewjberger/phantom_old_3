use super::{
    graphics::{BlendFunction, Graphics},
    shader::ShaderProgram,
};
use phantom_dependencies::{
    anyhow::Result,
    gl::{
        self,
        types::{GLuint, GLvoid},
    },
    nalgebra_glm as glm,
};

const VERTEX_SHADER_SOURCE: &'static str = &r#"
#version 460 core

layout (location=0) out vec2 uv;

layout(std140, binding = 0) uniform PerFrameData
{
	mat4 view;
	mat4 proj;
	vec4 cameraPos;
};

struct Vertex
{
	float p[3];
	float n[3];
	float tc[2];
};

layout(std430, binding = 1) restrict readonly buffer Vertices
{
	Vertex in_Vertices[];
};

layout(std430, binding = 2) restrict readonly buffer Matrices
{
	mat4 in_ModelMatrices[];
};

// extents of grid in world coordinates
float gridSize = 100.0;

// size of one cell
float gridCellSize = 0.025;

// color of thin lines
vec4 gridColorThin = vec4(0.5, 0.5, 0.5, 1.0);

// color of thick lines (every tenth line)
vec4 gridColorThick = vec4(0.0, 0.0, 0.0, 1.0);

// minimum number of pixels between cell lines before LOD switch should occur. 
const float gridMinPixelsBetweenCells = 2.0;

const vec3 pos[4] = vec3[4](
	vec3(-1.0, 0.0, -1.0),
	vec3( 1.0, 0.0, -1.0),
	vec3( 1.0, 0.0,  1.0),
	vec3(-1.0, 0.0,  1.0)
);

const int indices[6] = int[6](
	0, 1, 2, 2, 3, 0
);

void main()
{
	mat4 MVP = proj * view;

	int idx = indices[gl_VertexID];
	vec3 position = pos[idx] * gridSize;

	gl_Position = MVP * vec4(position, 1.0);
	uv = position.xz;
}
"#;

const FRAGMENT_SHADER_SOURCE: &'static str = &r#"
#version 460 core

layout (location=0) in vec2 uv;
layout (location=0) out vec4 out_FragColor;

layout(std140, binding = 0) uniform PerFrameData
{
	mat4 view;
	mat4 proj;
	vec4 cameraPos;
};

struct Vertex
{
	float p[3];
	float n[3];
	float tc[2];
};

layout(std430, binding = 1) restrict readonly buffer Vertices
{
	Vertex in_Vertices[];
};

layout(std430, binding = 2) restrict readonly buffer Matrices
{
	mat4 in_ModelMatrices[];
};

// extents of grid in world coordinates
float gridSize = 100.0;

// size of one cell
float gridCellSize = 0.025;

// color of thin lines
vec4 gridColorThin = vec4(0.5, 0.5, 0.5, 1.0);

// color of thick lines (every tenth line)
vec4 gridColorThick = vec4(0.0, 0.0, 0.0, 1.0);

// minimum number of pixels between cell lines before LOD switch should occur. 
const float gridMinPixelsBetweenCells = 2.0;

const vec3 pos[4] = vec3[4](
	vec3(-1.0, 0.0, -1.0),
	vec3( 1.0, 0.0, -1.0),
	vec3( 1.0, 0.0,  1.0),
	vec3(-1.0, 0.0,  1.0)
);

const int indices[6] = int[6](
	0, 1, 2, 2, 3, 0
);

float log10(float x)
{
	return log(x) / log(10.0);
}

float satf(float x)
{
	return clamp(x, 0.0, 1.0);
}

vec2 satv(vec2 x)
{
	return clamp(x, vec2(0.0), vec2(1.0));
}

float max2(vec2 v)
{
	return max(v.x, v.y);
}

vec4 gridColor(vec2 uv)
{
	vec2 dudv = vec2(
		length(vec2(dFdx(uv.x), dFdy(uv.x))),
		length(vec2(dFdx(uv.y), dFdy(uv.y)))
	);

	float lodLevel = max(0.0, log10((length(dudv) * gridMinPixelsBetweenCells) / gridCellSize) + 1.0);
	float lodFade = fract(lodLevel);

	// cell sizes for lod0, lod1 and lod2
	float lod0 = gridCellSize * pow(10.0, floor(lodLevel));
	float lod1 = lod0 * 10.0;
	float lod2 = lod1 * 10.0;

	// each anti-aliased line covers up to 4 pixels
	dudv *= 4.0;

	// calculate absolute distances to cell line centers for each lod and pick max X/Y to get coverage alpha value
	float lod0a = max2( vec2(1.0) - abs(satv(mod(uv, lod0) / dudv) * 2.0 - vec2(1.0)) );
	float lod1a = max2( vec2(1.0) - abs(satv(mod(uv, lod1) / dudv) * 2.0 - vec2(1.0)) );
	float lod2a = max2( vec2(1.0) - abs(satv(mod(uv, lod2) / dudv) * 2.0 - vec2(1.0)) );

	// blend between falloff colors to handle LOD transition
	vec4 c = lod2a > 0.0 ? gridColorThick : lod1a > 0.0 ? mix(gridColorThick, gridColorThin, lodFade) : gridColorThin;

	// calculate opacity falloff based on distance to grid extents
	float opacityFalloff = (1.0 - satf(length(uv) / gridSize));

	// blend between LOD level alphas and scale with opacity falloff
	c.a *= (lod2a > 0.0 ? lod2a : lod1a > 0.0 ? lod1a : (lod0a * (1.0-lodFade))) * opacityFalloff;

	return c;
}

void main()
{
	out_FragColor = gridColor(uv);
};
"#;

struct FrameData {
    view: glm::Mat4,
    projection: glm::Mat4,
    camera_position: glm::Vec4,
}

pub struct GridShader {
    shader_program: ShaderProgram,
    data_buffer: GLuint,
    vao: GLuint,
}

impl GridShader {
    pub fn new() -> Result<Self> {
        let mut shader_program = ShaderProgram::new();
        shader_program
            .vertex_shader_source(VERTEX_SHADER_SOURCE)?
            .fragment_shader_source(FRAGMENT_SHADER_SOURCE)?
            .link();
        let size = std::mem::size_of::<FrameData>();
        let data_buffer = unsafe {
            let mut data_buffer: GLuint = 0;
            gl::CreateBuffers(1, &mut data_buffer);
            gl::NamedBufferStorage(
                data_buffer,
                size as _,
                std::ptr::null(),
                gl::DYNAMIC_STORAGE_BIT,
            );
            gl::BindBufferRange(gl::UNIFORM_BUFFER, 0, data_buffer, 0, size as _);
            data_buffer
        };

        let mut vao: GLuint = 0;
        unsafe {
            gl::CreateVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        };

        Ok(Self {
            shader_program,
            data_buffer,
            vao,
        })
    }

    pub fn update(&self, view: glm::Mat4, projection: glm::Mat4, camera_position: glm::Vec3) {
        let data = FrameData {
            view,
            projection,
            camera_position: glm::vec4(
                camera_position.x,
                camera_position.y,
                camera_position.z,
                1.0,
            ),
        };
        unsafe {
            gl::NamedBufferSubData(
                self.data_buffer,
                0,
                std::mem::size_of::<FrameData>() as _,
                [data].as_ptr() as *const GLvoid,
            );
        }
    }

    pub fn render(&self) {
        self.shader_program.use_program();
        Graphics::enable_blending(
            BlendFunction::SourceAlpha,
            BlendFunction::OneMinusSourceAlpha,
        );
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArraysInstancedBaseInstance(gl::TRIANGLES, 0, 6, 1, 0);
        }
    }
}

impl Drop for GridShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.data_buffer);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
