mod allocator;
#[cfg(debug_assertions)]
pub mod debug_messanger;
mod device;
mod graphics_pipeline;
mod image_view;
mod instance;
mod pipeline;
mod shader_module;
mod surface;
mod swapchain;

pub use allocator::Allocator;
pub use device::Device;
pub use graphics_pipeline::GraphicsPipeline;
pub use image_view::ImageView;
pub use instance::Instance;
pub use instance::PhysicalDevice;
pub use pipeline::Pipeline;
pub use shader_module::ShaderModule;
pub use surface::CreateSurface;
pub use surface::Surface;
pub use surface::SurfaceExtent;
pub use swapchain::Swapchain;

pub type ExtensionName = *const std::os::raw::c_char;

pub fn make_api_version(major: u32, minor: u32, patch: u32) -> u32 {
	ash::vk::make_api_version(0, major, minor, patch)
}
