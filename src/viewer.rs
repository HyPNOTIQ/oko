use super::vulkan_wrapper;
use super::vulkan_wrapper::Allocator;
use super::vulkan_wrapper::Buffer;
use super::vulkan_wrapper::CommandPool;
use super::vulkan_wrapper::CreateSurface;
use super::vulkan_wrapper::Device;
use super::vulkan_wrapper::Fence;
use super::vulkan_wrapper::FrameBuffer;
use super::vulkan_wrapper::Image;
use super::vulkan_wrapper::ImageView;
use super::vulkan_wrapper::Instance;
use super::vulkan_wrapper::PhysicalDevice;
use super::vulkan_wrapper::Queue;
use super::vulkan_wrapper::RenderPass;
use super::vulkan_wrapper::Semaphore;
use super::vulkan_wrapper::ShaderModule;
use super::vulkan_wrapper::Surface;
use super::vulkan_wrapper::Swapchain;
use crate::slice_from_ref;
use anyhow::Result;
use ash::vk;
use gltf::Gltf;
use std::path::{Path, PathBuf};

pub enum Event {
	Stop,
}

pub struct LaunchConfig {
	pub input_file: PathBuf,
	pub scene_index: usize,
}

struct FrameResources<'a> {
	// pub index: usize,
	pub image_available_semaphore: Semaphore<'a>,
	pub render_done_semaphore: Semaphore<'a>,
	pub render_done_fence: Fence<'a>,
}

const SHADER_ENTRY_POINT: &std::ffi::CStr = cstr::cstr!("main");

#[repr(C)]
struct ViewProjectionUBO {
	mt: glm::TMat4<f32>,
}

pub fn run<SurfaceOwner: CreateSurface>(
	present_target: SurfaceOwner,
	config: LaunchConfig,
	rx: std::sync::mpsc::Receiver<Event>,
) -> Result<()> {
	let input_file = Path::new(&config.input_file);
	let gltf = Gltf::open(input_file)?;

	let surface_required_extension = present_target.required_extensions()?;

	// Instance
	let instance_extensions = [
		#[cfg(debug_assertions)]
		ash::extensions::ext::DebugUtils::name().as_ptr(),
	];

	let instance_extensions =
		[&instance_extensions[..], &surface_required_extension[..]].concat();

	let required_instance_version = vulkan_wrapper::make_api_version(1, 1, 0);
	let instance =
		Instance::new(&instance_extensions, required_instance_version)?;

	// Debug Messenger
	#[cfg(debug_assertions)]
	let _debug_messenger =
		super::vulkan_wrapper::debug_messanger::DebugMessenger::new(
			&instance,
			Some(vulkan_debug_callback),
		)?;

	//Surface
	let surface = Surface::new(present_target, &instance)?;

	// Physical device and main queue
	let (physical_device, graphics_queue_family_index) =
		find_suitable_physical_device(&instance, &surface).ok_or_else(
			|| anyhow::anyhow!("The suitable physical device is not found"),
		)?;

	// Device
	let queue_priorities = [1.0_f32];
	let queues = maplit::hashmap! {
			graphics_queue_family_index => &queue_priorities[..]
	};

	let device_extensions = [
		ash::extensions::khr::Swapchain::name().as_ptr(),
		ash::extensions::khr::TimelineSemaphore::name().as_ptr(),
	];

	let device =
		Device::new(&instance, physical_device, &device_extensions, &queues)?;

	// queue
	let queue = Queue::new(&device, graphics_queue_family_index);

	// allocator
	let allocator = Allocator::new(&instance, &device, physical_device, false)?;

	// Swapchain
	let swapchain = Swapchain::new(&instance, &device, &surface)?;
	let surface_extent = swapchain.extent();
	let swapchain_image_count = swapchain.image_count();

	// command pool
	let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
		.queue_family_index(graphics_queue_family_index);
	let command_pool = CommandPool::new(&device, &command_pool_create_info)?;

	// command buffers
	let command_buffers = command_pool.allocate_command_buffers(
		swapchain_image_count as _,
		vk::CommandBufferLevel::PRIMARY,
	)?;

	let vertex_buffers = create_vertex_buffers(
		&gltf,
		input_file,
		&command_pool,
		&allocator,
		&queue,
	)?;

	// vertex shader
	let _vertex_shader_module =
		ShaderModule::new(&device, &gen_shader_path("geometry.vert"))?;

	// fragment shader
	let _frag_shader_module =
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
	let multisample_state_info =
		vk::PipelineMultisampleStateCreateInfo::builder()
			.rasterization_samples(vk::SampleCountFlags::TYPE_1);

	// rasterization_info
	let rasterization_info =
		vk::PipelineRasterizationStateCreateInfo::builder()
			.polygon_mode(vk::PolygonMode::FILL)
			.cull_mode(vk::CullModeFlags::BACK)
			.front_face(vk::FrontFace::COUNTER_CLOCKWISE)
			.line_width(1.0);

	// color_blend_state
	let color_blend_attachments =
		[vk::PipelineColorBlendAttachmentState::builder()
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

	let graphics_pipeline_create_info =
		vk::GraphicsPipelineCreateInfo::builder()
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
			;

	// depth
	let depth_format_candidates = [
		vk::Format::D24_UNORM_S8_UINT,
		vk::Format::D32_SFLOAT,
		vk::Format::D32_SFLOAT_S8_UINT,
	];

	let depth_format_tiling = vk::ImageTiling::OPTIMAL;
	let depth_format_features =
		vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT;

	let depth_format = device
		.find_supported_format(
			&depth_format_candidates,
			depth_format_tiling,
			depth_format_features,
		)
		.ok_or_else(|| anyhow::anyhow!("Suitable depth format not found"))?;

	let depth_extent = vk::Extent3D::builder()
		.width(surface_extent.width)
		.height(surface_extent.height)
		.depth(1)
		.build();

	let depth_image_info = vk::ImageCreateInfo::builder()
		.image_type(vk::ImageType::TYPE_2D)
		.extent(depth_extent)
		.mip_levels(1)
		.array_layers(1)
		.format(depth_format)
		.tiling(depth_format_tiling)
		.initial_layout(vk::ImageLayout::UNDEFINED)
		.usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
		.samples(vk::SampleCountFlags::TYPE_1)
		.sharing_mode(vk::SharingMode::EXCLUSIVE);

	let depth_image =
		Image::new(&device, &allocator, &depth_image_info, "depth")?;

	let depth_image_subresource_range = vk::ImageSubresourceRange::builder()
		.level_count(1)
		.layer_count(1)
		.aspect_mask(vk::ImageAspectFlags::DEPTH)
		.build();

	let depth_image_view_info = vk::ImageViewCreateInfo::builder()
		.image(depth_image.handle())
		.view_type(vk::ImageViewType::TYPE_2D)
		.format(depth_format)
		.subresource_range(depth_image_subresource_range);

	let depth_image_view = ImageView::new(&device, &depth_image_view_info)?;

	// render pass
	let attachment_descriptions = [
		vk::AttachmentDescription::builder()
			.format(swapchain.format())
			.samples(vk::SampleCountFlags::TYPE_1)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(vk::AttachmentStoreOp::STORE)
			.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
			.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
			.build(),
		vk::AttachmentDescription::builder()
			.format(depth_format)
			.samples(vk::SampleCountFlags::TYPE_1)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(vk::AttachmentStoreOp::DONT_CARE)
			.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
			.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
			.build(),
	];

	let attachment_references = [vk::AttachmentReference::builder()
		.attachment(0)
		.layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
		.build()];

	let depth_attachment_reference = vk::AttachmentReference::builder()
		.attachment(1)
		.layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
		.build();

	let subpass_descriptions = [vk::SubpassDescription::builder()
		.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
		.color_attachments(&attachment_references)
		.depth_stencil_attachment(&depth_attachment_reference)
		.build()];

	let subpass_dependency = [vk::SubpassDependency::builder()
		.src_subpass(vk::SUBPASS_EXTERNAL)
		.dst_subpass(0)
		.src_stage_mask(
			vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
				| vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
		)
		.src_access_mask(vk::AccessFlags::empty())
		.dst_stage_mask(
			vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
				| vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
		)
		.dst_access_mask(
			vk::AccessFlags::COLOR_ATTACHMENT_WRITE
				| vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
		)
		.build()];

	let render_pass_create_info = vk::RenderPassCreateInfo::builder()
		.attachments(&attachment_descriptions)
		.subpasses(&subpass_descriptions)
		.dependencies(&subpass_dependency)
		.build();

	let render_pass = RenderPass::new(&device, &render_pass_create_info)?;

	// framebuffers
	let framebuffers = swapchain
		.image_views()
		.iter()
		.map(|image_view| {
			let attachments = [image_view.handle(), depth_image_view.handle()];

			let extent = surface_extent;

			let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
				.render_pass(render_pass.handle())
				.attachments(&attachments)
				.width(extent.width)
				.height(extent.height)
				.layers(1);

			FrameBuffer::new(&device, &frame_buffer_create_info)
		})
		.collect::<Result<Vec<_>>>()?;

	// record command buffers
	for (command_buffer, frame_buffer) in
		(&command_buffers).iter().zip(&framebuffers)
	{
		let begin_info = vk::CommandBufferBeginInfo::builder();

		command_buffer.begin(&begin_info)?;

		let render_area = ash::vk::Rect2D::builder()
			.extent(frame_buffer.extent())
			.build();

		let clear_color_value = vk::ClearColorValue {
			float32: [0.1, 0.1, 0.1, 1.0],
		};

		let clear_color_value = vk::ClearValue {
			color: clear_color_value,
		};

		let clear_depth_stencil_value = vk::ClearDepthStencilValue {
			depth: 1.0,
			stencil: 0,
		};

		let clear_depth_stencil_value = vk::ClearValue {
			depth_stencil: clear_depth_stencil_value,
		};

		let clear_values = [clear_color_value, clear_depth_stencil_value];

		let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
			.clear_values(&clear_values)
			.render_pass(render_pass.handle())
			.framebuffer(frame_buffer.handle())
			.render_area(render_area);

		command_buffer.begin_render_pass(
			&render_pass_begin_info,
			vk::SubpassContents::INLINE,
		);

		command_buffer.bind_vertex_buffers(
			vertex_buffers.handles(),
			vertex_buffers.offsets(),
		);

		command_buffer.end_render_pass();
		command_buffer.end()?;
	}

	let mut view_projection_buffers =
		std::iter::repeat_with(|| -> Result<_> {
			let buffer_create_info = vk::BufferCreateInfo::builder()
				.size(std::mem::size_of::<ViewProjectionUBO>() as _)
				.usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
				.sharing_mode(vk::SharingMode::EXCLUSIVE);

			let buffer = Buffer::new(
				&device,
				&allocator,
				&buffer_create_info,
				gpu_allocator::MemoryLocation::CpuToGpu,
				"",
			)?;

			Ok(Some(buffer))
		})
		.take(swapchain_image_count)
		.collect::<Result<Vec<_>>>()?;

	let frame_resources = std::iter::repeat_with(|| -> Result<_> {
		let frame_resources = FrameResources {
			image_available_semaphore: Semaphore::new(&device)?,
			render_done_semaphore: Semaphore::new(&device)?,
			render_done_fence: Fence::new(&device, true)?,
		};

		Ok(frame_resources)
	})
	.take(swapchain_image_count)
	.collect::<Result<Vec<_>>>()?;

	let mut iter = frame_resources.iter().cycle();

	let mut camera_dolly = 2.0_f32;
	let mut azimuth = 0.0_f32;
	let mut altitude = std::f32::consts::FRAC_PI_2;

	let mut current_index = 0_usize;
	loop {
		let mut stop = false;
		let frame_resources = iter.next().unwrap();

		for event in rx.try_iter() {
			match event {
				// Defer break to be sure all events processed
				Event::Stop => stop = true,
			}
		}

		let render_done_fence = &frame_resources.render_done_fence;
		render_done_fence.wait_max_timeout()?;
		render_done_fence.reset()?;

		let image_available_semaphore =
			&frame_resources.image_available_semaphore;
		let acquire_next_image_result = swapchain
			.acquire_next_image_max_timeout(
				image_available_semaphore,
				vk::Fence::null(),
			)?;

		// TODO: handle acruire result
		let next_image = acquire_next_image_result.0;

		let render_done_semaphore = &frame_resources.render_done_semaphore;

		let y = altitude.cos();
		let altitude_sin = altitude.sin();
		let x = azimuth.sin() * altitude_sin;
		let z = azimuth.cos() * altitude_sin;
		let camera_pos = glm::vec3::<f32>(x, y, z) * camera_dolly;

		let center = glm::vec3::<f32>(0.0, 0.0, 0.0);
		let up = glm::vec3::<f32>(0.0, 1.0, 0.0);

		let view = glm::look_at(&camera_pos, &center, &up);
		let fov = 90.0_f32;
		let fov = fov.to_radians();

		let width = surface_extent.width as f32;
		let height = surface_extent.height as f32;
		let near = 0.1_f32;
		let far = 10.0_f32;
		let projection =
			glm::perspective_fov_rh_zo(fov, width, height, near, far);

		let view_projection = ViewProjectionUBO {
			mt: projection * view,
		};

		let mut view_projection_beffer =
			view_projection_buffers[current_index].take().unwrap();

		view_projection_beffer.copy_into(&view_projection)?;
		view_projection_beffer.flush()?;
		view_projection_buffers[current_index].replace(view_projection_beffer);

		queue.submit(
			&command_buffers[next_image as usize],
			slice_from_ref(&image_available_semaphore.handle()),
			slice_from_ref(&vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT),
			slice_from_ref(&render_done_semaphore.handle()),
			render_done_fence,
		)?;

		// TODO: handle result
		let present_result = swapchain.present(
			&queue,
			next_image,
			slice_from_ref(&render_done_semaphore.handle()),
		)?;

		// log::info!("frame");

		current_index += 1;
		current_index %= swapchain_image_count;

		if stop {
			break;
		}
	}

	device.wait_idle()?;

	Ok(())
}

struct VertexDataBuffers<'a> {
	_buffers: Vec<Buffer<'a>>,
	handles: Vec<vk::Buffer>,
	offsets: Vec<vk::DeviceSize>,
}

impl<'a> VertexDataBuffers<'a> {
	pub fn new(buffers: Vec<Buffer<'a>>) -> Self {
		let (handles, offsets): (Vec<_>, Vec<_>) = buffers
			.iter()
			.map(|buffer| (buffer.handle(), buffer.offset() as vk::DeviceSize))
			.unzip();

		Self {
			_buffers: buffers,
			handles,
			offsets,
		}
	}

	pub fn handles(&self) -> &[vk::Buffer] {
		self.handles.as_slice()
	}

	pub fn offsets(&self) -> &[vk::DeviceSize] {
		self.offsets.as_slice()
	}
}

fn create_vertex_buffers<'a>(
	gltf: &Gltf,
	input_file: &Path,
	command_pool: &'a CommandPool,
	allocator: &'a Allocator,
	queue: &Queue,
) -> Result<VertexDataBuffers<'a>> {
	let transfer_command_buffer = command_pool
		.allocate_command_buffer(vk::CommandBufferLevel::PRIMARY)?;

	let device = command_pool.device();

	// buffers
	let staging_buffers = gltf
		.buffers()
		.map(|buffer| {
			let buffer_create_info = vk::BufferCreateInfo::builder()
				.size(buffer.length() as _)
				.usage(vk::BufferUsageFlags::TRANSFER_SRC)
				.sharing_mode(vk::SharingMode::EXCLUSIVE);

			let mut staging_buffer = Buffer::new(
				device,
				allocator,
				&buffer_create_info,
				gpu_allocator::MemoryLocation::CpuToGpu,
				"staging buffer",
			)?;

			let data = staging_buffer.mapped_slice_mut()?;

			match buffer.source() {
				gltf::buffer::Source::Uri(uri) => {
					use std::io::Read;

					let buffer_path = input_file.parent().unwrap().join(uri);
					let mut file = std::fs::File::open(buffer_path)?;
					file.read_exact(data)?;
				}
				gltf::buffer::Source::Bin => {
					//TODO: handle BIN source
					unimplemented!();
				}
			}

			staging_buffer.flush()?;

			Ok(staging_buffer)
		})
		.collect::<Result<Vec<_>>>()?;

	let begin_info = vk::CommandBufferBeginInfo::builder();
	transfer_command_buffer.begin(&begin_info)?;
	let vertex_buffers = staging_buffers
		.iter()
		.map(|stagin_buffer| -> Result<_> {
			let buffer_create_info = vk::BufferCreateInfo::builder()
				.size(stagin_buffer.size() as _)
				.usage(
					vk::BufferUsageFlags::INDEX_BUFFER
						| vk::BufferUsageFlags::VERTEX_BUFFER
						| vk::BufferUsageFlags::TRANSFER_DST,
				)
				.sharing_mode(vk::SharingMode::EXCLUSIVE)
				.build();

			let vertex_buffer = Buffer::new(
				device,
				allocator,
				&buffer_create_info,
				gpu_allocator::MemoryLocation::CpuToGpu,
				"vertex buffer",
			)?;

			transfer_command_buffer.copy_buffer(stagin_buffer, &vertex_buffer);

			Ok(vertex_buffer)
		})
		.collect::<Result<Vec<_>>>()?;

	transfer_command_buffer.end()?;
	let transfer_fence = Fence::new(&device, false)?;
	queue.submit(&transfer_command_buffer, &[], &[], &[], &transfer_fence)?;
	transfer_fence.wait_max_timeout()?;

	let vertex_buffers = VertexDataBuffers::new(vertex_buffers);
	Ok(vertex_buffers)
}

fn find_suitable_physical_device<'a, SurfaceOwner>(
	instance: &'a Instance,
	surface: &Surface<SurfaceOwner>,
) -> Option<(&'a PhysicalDevice, u32)> {
	instance
		.physical_devices()
		.iter()
		.find_map(|physical_device| {
			let queue_families_properties =
				&physical_device.queue_families_properties;

			let graphics_queue_index = queue_families_properties
				.iter()
				.enumerate()
				.find_map(|(queue_index, &queue_family_properties)| {
					let queue_index = queue_index as _;

					let surface_support = surface
						.physical_device_support(physical_device, queue_index);

					let graphic_support = queue_family_properties
						.queue_flags
						.contains(vk::QueueFlags::GRAPHICS);

					if graphic_support && surface_support {
						Some(queue_index)
					} else {
						None
					}
				});

			graphics_queue_index
				.map(|queue_index| (physical_device, queue_index))
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

	if let Ok(message) =
		std::ffi::CStr::from_ptr(callback_data.p_message).to_str()
	{
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
