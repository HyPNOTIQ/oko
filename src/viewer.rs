use super::vulkan_wrapper;
use super::vulkan_wrapper::Allocator;
use super::vulkan_wrapper::CreateSurface;
use super::vulkan_wrapper::Device;
use super::vulkan_wrapper::GraphicsPipeline;
use super::vulkan_wrapper::Instance;
use super::vulkan_wrapper::PhysicalDevice;
use super::vulkan_wrapper::ShaderModule;
use super::vulkan_wrapper::Surface;
use super::vulkan_wrapper::Swapchain;
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
	#[cfg(debug_assertions)]
	let _debug_messenger = super::vulkan_wrapper::debug_messanger::DebugMessenger::new(
		&instance,
		Some(vulkan_debug_callback),
	)?;

	//Surface
	let surface = Surface::new(present_target, &instance)?;

	// Physical device
	let physical_device = find_suitable_physical_device(&instance, &surface);

	let (physical_device, graphics_queue_family_index) = physical_device
		.ok_or_else(|| anyhow::anyhow!("The suitable physical device is not found!"))?;

	// Device
	let device_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];

	// Device
	let queue_priorities = [1.0_f32];
	let queues = maplit::hashmap! {
			graphics_queue_family_index => &queue_priorities[..]
	};

	let device_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];
	let device = Device::new(&instance, physical_device, &device_extensions, &queues)?;

	// Allocator
	let allocator = Allocator::new(&instance, &device, physical_device)?;
	let mut allocator = RefCell::new(allocator);

	// Swapchain
	let swapchain = Swapchain::new(&instance, &device, &surface)?;
	let surface_extent = swapchain.extent();

	// vertex shader
	let vertex_shader_module =
		ShaderModule::new(&device, &gen_shader_path("geometry.vert"))?;

	// fragment shader
	let frag_shader_module =
		ShaderModule::new(&device, &gen_shader_path("geometry.frag"))?;

	// viewport state
	let height = surface_extent.height as f32;
	let view_ports = [vk::Viewport::builder()
		.width(surface_extent.width as _)
		.height(-height)
		.y(height)
		.max_depth(1.0)
		.build()];

	let scissors = [vk::Rect2D::builder().extent(surface_extent).build()];

	let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
		.viewports(&view_ports)
		.scissors(&scissors);

	// multisample_state
	let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
		.rasterization_samples(vk::SampleCountFlags::TYPE_1);

	// rasterization_info
	let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
		.polygon_mode(vk::PolygonMode::FILL)
		.cull_mode(vk::CullModeFlags::BACK)
		.front_face(vk::FrontFace::COUNTER_CLOCKWISE)
		.line_width(1.0);

	// color_blend_state
	let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
		.color_write_mask(
			vk::ColorComponentFlags::R
				| vk::ColorComponentFlags::G
				| vk::ColorComponentFlags::B
				| vk::ColorComponentFlags::A,
		)
		.blend_enable(false)
		.build()];

	let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
		.attachments(&color_blend_attachments);

	let graphics_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
		// .stages(&shader_stage_create_infos)
		// .input_assembly_state(&vertex_input_assembly_state_info)
		// .vertex_input_state(&vertex_input_state_info)
		.rasterization_state(&rasterization_info)
		.color_blend_state(&color_blend_state)
		.multisample_state(&multisample_state_info)
		.viewport_state(&viewport_state_info)
		// .render_pass(render_pass.handle())
		// .layout(pipeline_layout.handle())
		// .depth_stencil_state(&depth_stencil_state_info)
		.build();

	// let pipeline = GraphicsPipeline::new(&device, &graphics_pipeline_create_info)?;

	// Geometry pipelines
	// let geometry_pipelines = {
	// 	// let mut geometry_pipelines;

	// };

	loop {
		let mut stop = false;

		for event in rx.try_iter() {
			match event {
				// Defer break to be sure all events processed
				Event::Stop => stop = true,
			}
		}

		if stop {
			break;
		}
	}

	device.wait_idle()?;

	Ok(())
}

fn find_suitable_physical_device<'a, SurfaceOwner>(
	instance: &'a Instance,
	surface: &Surface<SurfaceOwner>,
) -> Option<(&'a PhysicalDevice, u32)> {
	instance
		.physical_devices()
		.iter()
		.find_map(|physical_device| {
			let queue_families_properties = &physical_device.queue_families_properties;

			let graphics_queue_index = queue_families_properties
				.iter()
				.enumerate()
				.find_map(|(queue_index, &queue_family_properties)| {
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
				});

			graphics_queue_index.map(|queue_index| (physical_device, queue_index))
		})
}

fn gen_shader_path(name: &str) -> PathBuf {
	PathBuf::from("gen")
		.join(if cfg!(debug_assertions) {
			"debug"
		} else {
			"release"
		})
		.join("shaders")
		.join(name)
}

#[cfg(debug_assertions)]
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
