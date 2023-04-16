use phantom_render_traits::GpuDevice;
use phantom_vulkan::VulkanGpuDevice;
use phantom_wgpu::WgpuRenderer;
use phantom_world::Viewport;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::error::Error;

#[derive(Debug, Copy, Clone)]
pub enum Backend {
	Dx11,
	Dx12,
	Metal,
	Vulkan,
	VulkanWgpu,
}

pub fn create_renderer<W: HasRawWindowHandle + HasRawDisplayHandle>(
	backend: &Backend,
	window_handle: &W,
	viewport: &Viewport,
) -> Result<Box<dyn GpuDevice>, Box<dyn Error>> {
	let backend = if let Backend::Vulkan = backend {
		Box::new(VulkanGpuDevice::new(&window_handle, viewport)?) as _
	} else {
		let backend = map_backend(backend);
		Box::new(WgpuRenderer::new(&window_handle, backend, viewport)?) as _
	};
	Ok(backend)
}

fn map_backend(backend: &Backend) -> wgpu::Backend {
	match backend {
		Backend::Dx11 => wgpu::Backend::Dx11,
		Backend::Dx12 => wgpu::Backend::Dx12,
		Backend::Metal => wgpu::Backend::Metal,
		_ => wgpu::Backend::Vulkan,
	}
}
