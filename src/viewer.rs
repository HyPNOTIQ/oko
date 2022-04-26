use super::vulkan_wrapper;
use super::vulkan_wrapper::Allocator;
use super::vulkan_wrapper::CreateSurface;
use super::vulkan_wrapper::DebugMessenger;
use super::vulkan_wrapper::Device;
use super::vulkan_wrapper::Instance;
use super::vulkan_wrapper::PhysicalDevice;
use super::vulkan_wrapper::Surface;
// use super::vulkan_wrapper::Swapchain;
use anyhow::Result;
use ash::vk;
use gltf::Gltf;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

pub enum Event {
	Stop,
}

pub struct LaunchConfig {
	pub input_file: PathBuf,
	pub scene_index: usize,
}

pub fn run<SurfaceOwner: CreateSurface>(
	present_target: SurfaceOwner,
	config: LaunchConfig,
	rx: Receiver<Event>,
) -> Result<()> {
	let input_file = Path::new(&config.input_file);
	let gltf = Gltf::open(input_file)?;

	let surface_required_extension = present_target.required_extensions()?;

	// Instance
	let instance_extensions = [ash::extensions::ext::DebugUtils::name().as_ptr()];

	let instance_extensions =
		[&instance_extensions[..], &surface_required_extension[..]].concat();

	let required_instance_version = vulkan_wrapper::make_api_version(1, 2, 0);
	let instance = Instance::new(&instance_extensions, required_instance_version)?;

	// Debug Messenger
	let _debug_messenger = DebugMessenger::new(&instance, Some(vulkan_debug_callback))?;

	//Surface
	let surface = Surface::new(present_target, &instance)?;

	// Physical device
	let physical_device = find_suitable_physical_device(&instance, &surface);

	let (physical_device, graphics_queue_family_index) = physical_device.ok_or(
		anyhow::anyhow!("The suitable physical device is not found!"),
	)?;

	// Device
	let device_extensions = [
		ash::extensions::khr::Swapchain::name().as_ptr(),
		// ash::extensions::khr::TimelineSemaphore::name().as_ptr(),
	];

	// Device
	let queue_priorities = [1.0_f32];
	let queues = maplit::hashmap! {
			graphics_queue_family_index => &queue_priorities[..]
	};

	let device_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];
	let device = Device::new(&instance, physical_device, &device_extensions, &queues)?;

	// Allocator
	let allocator = Allocator::new(&instance, &device, &physical_device)?;
	let mut allocator = RefCell::new(allocator);

	// Swapchain
	// let swapchain = Swapchain::<SurfaceOwner>::new(&instance, &device, &surface)?;

	loop {
		let mut stop = false;

		for event in rx.try_iter() {
			match event {
				Event::Stop => stop = true,
			}
		}

		if stop {
			break;
		}
	}

	Ok(())
}

fn find_suitable_physical_device<'a, SurfaceOwner>(
	instance: &'a Instance,
	surface: &Surface<SurfaceOwner>,
) -> Option<(&'a PhysicalDevice, u32)> {
	let physical_devices = instance.physical_devices();

	let is_physical_device_suitable = |physical_device: &'a PhysicalDevice| {
		let queue_families_properties = physical_device.queue_families_properties();

		let is_queue_suitable = |(queue_index, &queue_family_properties): (
			usize,
			&vk::QueueFamilyProperties,
		)| {
			let queue_index = queue_index as _;

			let surface_support =
				surface.physical_device_support(physical_device, queue_index);

			let graphic_support = queue_family_properties
				.queue_flags
				.contains(vk::QueueFlags::GRAPHICS);

			if graphic_support && surface_support {
				Some(queue_index)
			} else {
				None
			}
		};

		let graphics_queue_index = queue_families_properties
			.iter()
			.enumerate()
			.find_map(is_queue_suitable);

		if let Some(queue_index) = graphics_queue_index {
			Some((physical_device, queue_index))
		} else {
			None
		}
	};

	physical_devices
		.iter()
		.find_map(is_physical_device_suitable)
}

unsafe extern "system" fn vulkan_debug_callback(
	severity_flags: vk::DebugUtilsMessageSeverityFlagsEXT,
	message_type_flags: vk::DebugUtilsMessageTypeFlagsEXT,
	callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
	_: *mut std::os::raw::c_void,
) -> vk::Bool32 {
	use ::log::{error, info, trace, warn};

	use vk::DebugUtilsMessageSeverityFlagsEXT;
	use vk::DebugUtilsMessageTypeFlagsEXT;

	let callback_data = *callback_data;

	if let Ok(message) = std::ffi::CStr::from_ptr(callback_data.p_message).to_str() {
		let message_type = match message_type_flags {
			DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
			DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
			DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
			_ => "[Unknown]",
		};

		let message = format!("{} {}", message_type, message);

		match severity_flags {
			DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("{}", message),
			DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("{}", message),
			DebugUtilsMessageSeverityFlagsEXT::INFO => info!("{}", message),
			_ => trace!("{}", message),
		};
	} else {
		error!("Vulkan debug callback: unable to get message data!");
	}

	vk::FALSE
}
