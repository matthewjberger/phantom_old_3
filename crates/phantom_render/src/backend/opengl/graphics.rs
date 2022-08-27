use phantom_dependencies::{
    gl::{self, types::GLuint},
    nalgebra_glm as glm,
};

pub enum CullMode {
    Front,
    Back,
    FrontAndBack,
}

impl From<CullMode> for GLuint {
    fn from(cull_mode: CullMode) -> Self {
        match cull_mode {
            CullMode::Front => gl::FRONT,
            CullMode::Back => gl::BACK,
            CullMode::FrontAndBack => gl::FRONT_AND_BACK,
        }
    }
}

pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

impl From<FrontFace> for GLuint {
    fn from(front_face: FrontFace) -> Self {
        match front_face {
            FrontFace::Clockwise => gl::CW,
            FrontFace::CounterClockwise => gl::CCW,
        }
    }
}

pub enum DepthTestFunction {
    Never,
    Always,
    LessThan,
    GreaterThan,
    LessThanOrEqualTo,
    GreaterThanOrEqualTo,
    EqualTo,
    NotEqualTo,
}

impl From<DepthTestFunction> for GLuint {
    fn from(depth_test_function: DepthTestFunction) -> Self {
        match depth_test_function {
            DepthTestFunction::Never => gl::NEVER,
            DepthTestFunction::Always => gl::ALWAYS,
            DepthTestFunction::LessThan => gl::LESS,
            DepthTestFunction::GreaterThan => gl::GREATER,
            DepthTestFunction::LessThanOrEqualTo => gl::LEQUAL,
            DepthTestFunction::GreaterThanOrEqualTo => gl::GEQUAL,
            DepthTestFunction::EqualTo => gl::EQUAL,
            DepthTestFunction::NotEqualTo => gl::NOTEQUAL,
        }
    }
}

/// OpenGL Blend Functions
///
/// Blending in OpenGL happens with the following equation:
/// C_result = C_source * F_source + C_destination * F_destination
///
/// C_source is the source color vector. This is the color output of the fragment shader.
/// C_destination is the destination color vector. This is the color vector that is currently stored in the color buffer.
/// F_source is the source factor value. Sets the impact of the alpha value on the source color.
/// F_destination is the destination factor value. Sets the impact of the alpha value on the destination color.
pub enum BlendFunction {
    /// Factor is equal to zero
    Zero,

    /// Factor is equal to 1
    One,

    /// Factor is equal to 1 minus the source color vector: 1−C¯source.
    OneMinusSourceColor,

    /// Factor is equal to the destination color vector C¯destination
    DestinationColor,

    /// Factor is equal to 1 minus the destination color vector: 1−C¯destination.
    OneMinusDestinationColor,

    /// Factor is equal to the alpha component of the source color vector C¯source.
    SourceAlpha,

    /// Factor is equal to 1−alpha of the source color vector C¯source.
    OneMinusSourceAlpha,

    /// Factor is equal to the alpha component of the destination color vector C¯destination.
    DestinationAlpha,

    /// Factor is equal to 1−alpha of the destination color vector C¯destination.
    OneMinusDestinationAlpha,

    /// Factor is equal to the constant color vector C¯constant.
    ConstantColor,

    /// Factor is equal to 1 - the constant color vector C¯constant.
    OneMinusConstantColor,

    /// Factor is equal to the alpha component of the constant color vector C¯constant.
    ConstantAlpha,

    /// Factor is equal to 1−alpha of the constant color vector C¯constant.
    OneMinusConstantAlpha,
}

impl From<BlendFunction> for GLuint {
    fn from(blend_function: BlendFunction) -> Self {
        match blend_function {
            BlendFunction::Zero => gl::ZERO,
            BlendFunction::One => gl::ONE,
            BlendFunction::OneMinusSourceColor => gl::ONE_MINUS_SRC_ALPHA,
            BlendFunction::DestinationColor => gl::DST_COLOR,
            BlendFunction::OneMinusDestinationColor => gl::ONE_MINUS_DST_COLOR,
            BlendFunction::SourceAlpha => gl::SRC_ALPHA,
            BlendFunction::OneMinusSourceAlpha => gl::ONE_MINUS_SRC_ALPHA,
            BlendFunction::DestinationAlpha => gl::DST_ALPHA,
            BlendFunction::OneMinusDestinationAlpha => gl::ONE_MINUS_DST_ALPHA,
            BlendFunction::ConstantColor => gl::CONSTANT_COLOR,
            BlendFunction::OneMinusConstantColor => gl::ONE_MINUS_CONSTANT_COLOR,
            BlendFunction::ConstantAlpha => gl::CONSTANT_ALPHA,
            BlendFunction::OneMinusConstantAlpha => gl::ONE_MINUS_CONSTANT_ALPHA,
        }
    }
}

pub struct Graphics;

impl Graphics {
    pub fn enable_culling(mode: CullMode, front_face: FrontFace) {
        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(mode.into());
            gl::FrontFace(front_face.into());
        }
    }

    pub fn disable_culling() {
        unsafe {
            gl::Disable(gl::CULL_FACE);
        }
    }

    pub fn enable_depth_testing(depth_function: DepthTestFunction) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(depth_function.into());
        }
    }

    pub fn disable_depth_testing() {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
        }
    }

    pub fn enable_blending(source_function: BlendFunction, destination_function: BlendFunction) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(source_function.into(), destination_function.into());
        }
    }

    pub fn disable_blending() {
        unsafe {
            gl::Disable(gl::BLEND);
        }
    }

    pub fn clear_buffers() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }

    pub fn clear_color(color: &glm::Vec3) {
        unsafe {
            gl::ClearColor(color.x, color.y, color.z, 1.0);
        }
    }
}
