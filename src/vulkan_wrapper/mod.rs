mod allocator;
mod command_pool;
#[cfg(debug_assertions)]
pub mod debug_messanger;
mod device;
mod fence;
mod graphics_pipeline;
mod image_view;
mod instance;
mod pipeline;
mod semaphore;
mod shader_module;
mod surface;
mod swapchain;
mod timeline_semaphore;

pub use allocator::Allocator;
pub use command_pool::CommandPool;
pub use device::Device;
pub use fence::Fence;
pub use graphics_pipeline::GraphicsPipeline;
pub use image_view::ImageView;
pub use instance::Instance;
pub use instance::PhysicalDevice;
pub use pipeline::Pipeline;
pub use semaphore::Semaphore;
pub use shader_module::ShaderModule;
pub use surface::CreateSurface;
pub use surface::Surface;
pub use surface::SurfaceExtent;
pub use swapchain::Swapchain;
pub use timeline_semaphore::TimelineSemaphore;

pub type ExtensionName = *const std::os::raw::c_char;

pub fn make_api_version(major: u32, minor: u32, patch: u32) -> u32 {
	ash::vk::make_api_version(0, major, minor, patch)
}
