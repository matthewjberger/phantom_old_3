use phantom_dependencies::{
    image::{
        codecs::hdr::HdrDecoder, io::Reader as ImageReader, DynamicImage, GenericImageView,
        ImageBuffer, ImageError, Pixel, RgbImage,
    },
    nalgebra_glm as glm,
    serde::{Deserialize, Serialize},
};
use std::{io::BufReader, path::Path};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Failed to match the provided image format to a vulkan format!")]
    DetermineImageFormat,

    #[error("Failed to load image from file!")]
    LoadHdrFromFile(#[source] std::io::Error),

    #[error("Failed to load image from file!")]
    LoadImageFromFile(#[source] std::io::Error),

    #[error("Failed to decode image!")]
    DecodeImage(#[source] ImageError),

    #[error("Failed to decode HDR image!")]
    DecodeHdrImage(#[source] ImageError),

    #[error("Failed to map texture format to world texture format!")]
    MapFormat,

    #[error("Failed to create image buffer from raw pixel data!")]
    CreateImageBuffer,
}

type Result<T, E = TextureError> = std::result::Result<T, E>;

// FIXME: Add mip levels
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct Texture {
    pub pixels: Vec<u8>,
    pub format: Format,
    pub width: u32,
    pub height: u32,
    pub sampler: Sampler,
}

impl Texture {
    pub fn new(
        pixels: Vec<u8>,
        format: Format,
        width: u32,
        height: u32,
        sampler: Sampler,
    ) -> Result<Self> {
        let mut texture = Self {
            pixels,
            format,
            width,
            height,
            sampler,
        };
        texture.convert_24bit_formats()?;
        Ok(texture)
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let image = ImageReader::open(path)
            .map_err(TextureError::LoadImageFromFile)?
            .decode()
            .map_err(TextureError::DecodeImage)?;
        let (width, height) = image.dimensions();
        let format = Self::map_format(&image)?;
        let pixels = image.into_bytes();

        Self::new(pixels, format, width, height, Sampler::default())
    }

    pub fn map_format(image: &DynamicImage) -> Result<Format> {
        Ok(match image {
            DynamicImage::ImageRgb8(_) => Format::R8G8B8,
            DynamicImage::ImageRgba8(_) => Format::R8G8B8A8,
            DynamicImage::ImageRgb16(_) => Format::R16G16B16,
            DynamicImage::ImageRgba16(_) => Format::R16G16B16A16,
            DynamicImage::ImageRgba32F(_) => Format::R32G32B32A32F,
            _ => return Err(TextureError::MapFormat),
        })
    }

    fn convert_24bit_formats(&mut self) -> Result<()> {
        // 24-bit formats are unsupported, so they
        // need to have an alpha channel added to make them 32-bit
        let format = match self.format {
            Format::R8G8B8 => Format::R8G8B8A8,
            Format::B8G8R8 => Format::B8G8R8A8,
            _ => return Ok(()),
        };
        self.format = format;
        self.attach_alpha_channel()
    }

    fn attach_alpha_channel(&mut self) -> Result<()> {
        let image_buffer: RgbImage =
            ImageBuffer::from_raw(self.width, self.height, self.pixels.to_vec())
                .ok_or(TextureError::CreateImageBuffer)?;

        self.pixels = image_buffer
            .pixels()
            .flat_map(|pixel| pixel.to_rgba().channels().to_vec())
            .collect::<Vec<_>>();

        Ok(())
    }

    pub fn from_hdr(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(&path).map_err(TextureError::LoadHdrFromFile)?;
        let decoder =
            HdrDecoder::new(BufReader::new(file)).map_err(TextureError::DecodeHdrImage)?;
        let metadata = decoder.metadata();
        let decoded = decoder
            .read_image_hdr()
            .map_err(TextureError::DecodeHdrImage)?;
        let width = metadata.width as u32;
        let height = metadata.height as u32;
        let data = decoded
            .iter()
            .flat_map(|pixel| vec![pixel[0], pixel[1], pixel[2], 1.0])
            .collect::<Vec<_>>();
        let pixels =
            unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 4) }
                .to_vec();
        Ok(Self {
            pixels,
            format: Format::R32G32B32A32F,
            width,
            height,
            sampler: Sampler::default(),
        })
    }

    pub fn padded_bytes_per_row(&self, alignment: u32) -> u32 {
        let bytes_per_row = self.bytes_per_row();
        let padding = (alignment - bytes_per_row % alignment) % alignment;
        bytes_per_row + padding
    }

    pub fn bytes_per_row(&self) -> u32 {
        self.bytes_per_pixel() * self.width
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        match self.format {
            Format::R8 => 1,
            Format::R8G8 => 2,
            Format::R8G8B8 | Format::B8G8R8 => 3,
            Format::R8G8B8A8 | Format::B8G8R8A8 => 4,

            Format::R16 | Format::R16F => 2,
            Format::R16G16 | Format::R16G16F => 4,
            Format::R16G16B16 | Format::R16G16B16F => 6,
            Format::R16G16B16A16 | Format::R16G16B16A16F => 8,

            Format::R32 | Format::R32F => 4,
            Format::R32G32 | Format::R32G32F => 8,
            Format::R32G32B32 | Format::R32G32B32F => 12,
            Format::R32G32B32A32 | Format::R32G32B32A32F => 16,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub enum Format {
    R8,
    R8G8,
    R8G8B8,
    R8G8B8A8,
    B8G8R8,
    B8G8R8A8,
    R16,
    R16G16,
    R16G16B16,
    R16G16B16A16,
    R16F,
    R16G16F,
    R16G16B16F,
    R16G16B16A16F,
    R32,
    R32G32,
    R32G32B32,
    R32G32B32A32,
    R32F,
    R32G32F,
    R32G32B32F,
    R32G32B32A32F,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct Sampler {
    pub name: String,
    pub min_filter: Filter,
    pub mag_filter: Filter,
    pub wrap_s: WrappingMode,
    pub wrap_t: WrappingMode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub enum WrappingMode {
    ClampToEdge,
    MirroredRepeat,
    Repeat,
}

impl Default for WrappingMode {
    fn default() -> Self {
        Self::Repeat
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub enum Filter {
    Nearest,
    Linear,
}

impl Default for Filter {
    fn default() -> Self {
        Self::Nearest
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub struct Material {
    pub name: String,
    pub base_color_factor: glm::Vec4,
    pub emissive_factor: glm::Vec3,
    pub color_texture_index: i32,
    pub color_texture_set: i32,
    pub metallic_roughness_texture_index: i32,
    pub metallic_roughness_texture_set: i32, // B channel - metalness values. G channel - roughness values
    pub normal_texture_index: i32,
    pub normal_texture_set: i32,
    pub normal_texture_scale: f32,
    pub occlusion_texture_index: i32,
    pub occlusion_texture_set: i32, // R channel - occlusion values
    pub occlusion_strength: f32,
    pub emissive_texture_index: i32,
    pub emissive_texture_set: i32,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub is_unlit: bool,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "<Unnamed>".to_string(),
            base_color_factor: glm::vec4(1.0, 1.0, 1.0, 1.0),
            emissive_factor: glm::Vec3::identity(),
            color_texture_index: -1,
            color_texture_set: -1,
            metallic_roughness_texture_index: -1,
            metallic_roughness_texture_set: -1,
            normal_texture_index: -1,
            normal_texture_set: -1,
            normal_texture_scale: 1.0,
            occlusion_texture_index: -1,
            occlusion_texture_set: -1,
            occlusion_strength: 1.0,
            emissive_texture_index: -1,
            emissive_texture_set: -1,
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            is_unlit: false,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[serde(crate = "phantom_dependencies::serde")]
pub enum AlphaMode {
    Opaque = 1,
    Mask,
    Blend,
}

impl Default for AlphaMode {
    fn default() -> Self {
        Self::Opaque
    }
}
