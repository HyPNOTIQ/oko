mod allocator;
mod debug_messanger;
mod device;
mod image_view;
mod instance;
mod physical_device;
mod surface;
// mod swapchain;

pub use allocator::Allocator;
pub use debug_messanger::DebugMessenger;
pub use device::Device;
pub use image_view::ImageView;
pub use instance::Instance;
pub use physical_device::PhysicalDevice;
pub use surface::CreateSurface;
pub use surface::Surface;
pub use surface::SurfaceExtent;
// pub use swapchain::Swapchain;

pub type ExtensionName = *const std::os::raw::c_char;

pub fn make_api_version(major: u32, minor: u32, patch: u32) -> u32 {
	ash::vk::make_api_version(0, major, minor, patch)
}
