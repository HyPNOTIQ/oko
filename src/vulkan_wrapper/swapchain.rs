use {
	super::{Device, ImageView, Instance, Surface, SurfaceExtent},
	anyhow::Result,
	ash::vk,
	ash::vk::{
		Extent2D, Format, PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
	},
};

pub struct Swapchain<'a, T> {
	format: Format,
	extent: Extent2D,
	handle: vk::SwapchainKHR,
	image_views: Vec<ImageView<'a>>,
	extension: ash::extensions::khr::Swapchain,
	_device: &'a Device<'a>,
	_surface: &'a Surface<'a, T>,
}

impl<'a, T> Swapchain<'a, T> {
	pub fn new(
		instance: &'a Instance,
		device: &'a Device,
		surface: &'a Surface<T>,
	) -> Result<Self>
	where
		T: SurfaceExtent,
	{
		let extension =
			ash::extensions::khr::Swapchain::new(instance.inner(), device.inner());

		let physical_device = device.physical_device();
		let surface_capabilities = surface.capabilities(physical_device)?;
		let min_images_count = Self::get_min_images_count(&surface_capabilities);
		let surface_present_modes = surface.present_modes(physical_device)?;
		let surface_formats = surface.formats(physical_device)?;
		let surface_format = Self::get_surface_format(&surface_formats);
		let surface_format = surface_format.unwrap();
		let extent = Self::get_extent(&surface_capabilities, surface.extent());
		let present_mode = Self::get_present_mode(&surface_present_modes);
		let format = surface_format.format;
		let color_space = surface_format.color_space;

		let create_info = ash::vk::SwapchainCreateInfoKHR::builder()
			.surface(surface.handle())
			.min_image_count(min_images_count)
			.image_format(format)
			.image_color_space(color_space)
			.image_extent(extent)
			.present_mode(present_mode)
			.image_array_layers(1)
			.image_usage(ash::vk::ImageUsageFlags::COLOR_ATTACHMENT)
			.image_sharing_mode(ash::vk::SharingMode::EXCLUSIVE)
			.pre_transform(surface_capabilities.current_transform)
			.composite_alpha(ash::vk::CompositeAlphaFlagsKHR::OPAQUE)
			.clipped(true);

		let handle = unsafe { extension.create_swapchain(&create_info, None)? };

		let images = unsafe { extension.get_swapchain_images(handle)? };

		let image_views = images
			.iter()
			.map(|&image| {
				let component_mapping = vk::ComponentMapping::builder().build();

				let image_subresource_range = vk::ImageSubresourceRange::builder()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1)
					.build();

				let create_info = vk::ImageViewCreateInfo::builder()
					.image(image)
					.view_type(vk::ImageViewType::TYPE_2D)
					.format(format)
					.components(component_mapping)
					.subresource_range(image_subresource_range);

				ImageView::new(device, &create_info)
			})
			.collect::<Result<Vec<_>>>()?;

		let swapchain = Self {
			format,
			extent,
			handle,
			image_views,
			extension,
			_device: device,
			_surface: surface,
		};

		Ok(swapchain)
	}

	pub fn extent(&self) -> Extent2D {
		self.extent
	}

	pub fn format(&self) -> Format {
		self.format
	}

	pub fn image_views(&self) -> &Vec<ImageView> {
		&self.image_views
	}

	pub fn image_count(&self) -> usize {
		self.image_views.len()
	}

	pub fn acquire_next_image(
		&self,
		semaphore: vk::Semaphore,
		fence: vk::Fence,
		timeout: u64,
	) -> ash::prelude::VkResult<(u32, bool)> {
		unsafe {
			self.extension
				.acquire_next_image(self.handle, timeout, semaphore, fence)
		}
	}

	fn get_min_images_count(capabilities: &vk::SurfaceCapabilitiesKHR) -> u32 {
		let desired_min_images_count = capabilities.min_image_count + 1;

		let max_image_count = capabilities.max_image_count;
		if max_image_count != 0 {
			std::cmp::min(desired_min_images_count, max_image_count)
		} else {
			desired_min_images_count
		}
	}

	fn get_surface_format(formats: &[SurfaceFormatKHR]) -> Option<SurfaceFormatKHR> {
		let desired_format = ash::vk::Format::B8G8R8A8_UNORM;
		let desired_color_space = ash::vk::ColorSpaceKHR::SRGB_NONLINEAR;

		let format = formats.iter().copied().find(|format| {
			format.format == desired_format && format.color_space == desired_color_space
		});

		if format.is_none() {
			formats.first().copied()
		} else {
			format
		}
	}

	fn get_extent(
		capabilities: &SurfaceCapabilitiesKHR,
		surface_extent: Extent2D,
	) -> Extent2D {
		let current_extent = capabilities.current_extent;

		let special_value = std::u32::MAX;

		let special_value = current_extent.width == special_value
			&& current_extent.height == special_value;

		if special_value {
			let min_image_extent = capabilities.min_image_extent;
			let max_image_extent = capabilities.max_image_extent;

			let width = surface_extent
				.width
				.clamp(min_image_extent.width, max_image_extent.width);

			let height = surface_extent
				.height
				.clamp(min_image_extent.height, max_image_extent.height);

			Extent2D { width, height }
		} else {
			current_extent
		}
	}

	fn get_present_mode(present_modes: &[ash::vk::PresentModeKHR]) -> PresentModeKHR {
		let find_present_mode =
			|present_mode| present_modes.contains(present_mode).then(|| *present_mode);

		if let Some(present_mode) = find_present_mode(&PresentModeKHR::MAILBOX) {
			present_mode
		} else if let Some(present_mode) = find_present_mode(&PresentModeKHR::IMMEDIATE) {
			present_mode
		} else {
			PresentModeKHR::FIFO
		}
	}
}

impl<'a, T> Drop for Swapchain<'a, T> {
	fn drop(&mut self) {
		unsafe {
			self.extension.destroy_swapchain(self.handle, None);
		}
	}
}
