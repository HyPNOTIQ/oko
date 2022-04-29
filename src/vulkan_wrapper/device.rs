use {
	super::{ExtensionName, Instance, PhysicalDevice},
	anyhow::Result,
	ash::vk,
	std::collections::HashMap,
};

pub struct Device<'a> {
	inner: ash::Device,
	physical_device: &'a PhysicalDevice,
	instance: &'a Instance,
}

impl<'a> Device<'a> {
	pub fn new(
		instance: &'a Instance,
		physical_device: &'a PhysicalDevice,
		extensions: &[ExtensionName],
		queues: &HashMap<u32, &[f32]>,
	) -> Result<Self> {
		let device_queue_create_info = queues
			.iter()
			.map(|(&queue_family_index, queue_priorities)| {
				vk::DeviceQueueCreateInfo::builder()
					.queue_family_index(queue_family_index)
					.queue_priorities(queue_priorities)
					.build()
			})
			.collect::<Vec<_>>();

		let device_create_info = vk::DeviceCreateInfo::builder()
			.queue_create_infos(&device_queue_create_info)
			.enabled_extension_names(extensions);

		let inner = unsafe {
			instance.inner().create_device(
				physical_device.handle,
				&device_create_info,
				None,
			)?
		};

		let device = Self {
			instance,
			inner,
			physical_device,
		};

		Ok(device)
	}

	pub fn inner(&self) -> &ash::Device {
		&self.inner
	}

	pub fn physical_device(&self) -> &PhysicalDevice {
		self.physical_device
	}

	pub fn wait_idle(&self) -> Result<()> {
		unsafe { self.inner.device_wait_idle()? }

		Ok(())
	}

	pub fn update_descriptor_sets(
		&self,
		descriptor_writes: &[vk::WriteDescriptorSet],
		descriptor_copies: &[vk::CopyDescriptorSet],
	) {
		unsafe {
			self.inner
				.update_descriptor_sets(descriptor_writes, descriptor_copies);
		}
	}

	pub fn find_supported_format(
		&self,
		candidates: &[vk::Format],
		tiling: vk::ImageTiling,
		features: vk::FormatFeatureFlags,
	) -> Option<vk::Format> {
		candidates.iter().find_map(|&format| {
			let format_properties = unsafe {
				// TODO: add caching of format properties
				self.instance.inner().get_physical_device_format_properties(
					self.physical_device.handle,
					format,
				)
			};

			use vk::ImageTiling;

			let optimal_features =
				format_properties.optimal_tiling_features.contains(features);
			let linear_features =
				format_properties.linear_tiling_features.contains(features);

			(tiling == ImageTiling::OPTIMAL && optimal_features
				|| tiling == ImageTiling::LINEAR && linear_features)
				.then(|| format)
		})
	}
}

impl<'a> Drop for Device<'a> {
	fn drop(&mut self) {
		// self.allocator.destroy();

		unsafe {
			self.inner.destroy_device(None);
		}
	}
}
