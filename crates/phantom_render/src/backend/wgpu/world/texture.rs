use phantom_dependencies::{
    log,
    thiserror::Error,
    wgpu::{self, TextureFormat},
};
use phantom_world::{Filter, Format, WrappingMode};

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("The specified texture format is not supported: `{0:?}`")]
    UnsupportedTextureFormat(Format),
}

type Result<T, E = TextureError> = std::result::Result<T, E>;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float; // 1.

    pub fn from_world_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        world_texture: &phantom_world::Texture,
        label: &str,
    ) -> Result<Self> {
        let size = wgpu::Extent3d {
            width: world_texture.width,
            height: world_texture.height,
            depth_or_array_layers: 1,
        };

        let format = Self::map_texture_format(world_texture.format)?;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &world_texture.pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(world_texture.bytes_per_row()),
                rows_per_image: std::num::NonZeroU32::new(world_texture.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("WorldTextureView"),
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let sampler = device.create_sampler(&Self::map_sampler(&world_texture.sampler));

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    fn map_texture_format(texture_format: Format) -> Result<TextureFormat> {
        log::info!("Texture format: {:#?}", texture_format);
        let format = match texture_format {
            Format::B8G8R8A8 => TextureFormat::Bgra8Unorm,
            Format::R16 => TextureFormat::R16Uint,
            Format::R16F => TextureFormat::R16Float,
            Format::R16G16 => TextureFormat::Rg16Uint,
            Format::R16G16B16A16 => TextureFormat::Rgba16Uint,
            Format::R16G16B16A16F => TextureFormat::Rgba16Float,
            Format::R16G16F => TextureFormat::Rg16Float,
            Format::R32 => TextureFormat::R32Uint,
            Format::R32F => TextureFormat::R32Float,
            Format::R32G32 => TextureFormat::Rg32Uint,
            Format::R32G32B32A32 => TextureFormat::Rgba32Uint,
            Format::R32G32B32A32F => TextureFormat::Rgba32Float,
            Format::R32G32F => TextureFormat::Rg32Float,
            Format::R8 => TextureFormat::R8Uint,
            Format::R8G8 => TextureFormat::Rg8Uint,
            Format::R8G8B8A8 => TextureFormat::Rgba8Unorm,
            format => return Err(TextureError::UnsupportedTextureFormat(format)),
        };
        Ok(format)
    }

    fn map_sampler(sampler: &phantom_world::Sampler) -> wgpu::SamplerDescriptor<'static> {
        let min_filter = match sampler.min_filter {
            Filter::Linear => wgpu::FilterMode::Linear,
            Filter::Nearest => wgpu::FilterMode::Nearest,
        };

        let mipmap_filter = match sampler.min_filter {
            Filter::Linear => wgpu::FilterMode::Linear,
            Filter::Nearest => wgpu::FilterMode::Nearest,
        };

        let mag_filter = match sampler.mag_filter {
            Filter::Nearest => wgpu::FilterMode::Nearest,
            Filter::Linear => wgpu::FilterMode::Linear,
        };

        let address_mode_u = match sampler.wrap_s {
            WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            WrappingMode::Repeat => wgpu::AddressMode::Repeat,
        };

        let address_mode_v = match sampler.wrap_t {
            WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
            WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
            WrappingMode::Repeat => wgpu::AddressMode::Repeat,
        };

        let address_mode_w = wgpu::AddressMode::Repeat;

        wgpu::SamplerDescriptor {
            address_mode_u,
            address_mode_v,
            address_mode_w,
            mag_filter,
            min_filter,
            mipmap_filter,
            ..Default::default()
        }
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}
