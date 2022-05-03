use {super::Device, anyhow::Result, ash::vk};

pub struct DescriptorPool<'a> {
	handle: vk::DescriptorPool,
	device: &'a Device<'a>,
}

impl<'a> DescriptorPool<'a> {
	pub fn new(
		device: &'a Device,
		create_info: &vk::DescriptorPoolCreateInfo,
	) -> Result<Self> {
		let handle = unsafe {
			device.inner().create_descriptor_pool(create_info, None)?
		};

		let descriptor_pool = Self { handle, device };

		Ok(descriptor_pool)
	}

	pub fn allocate_descriptor_sets(
		&self,
		set_layouts: &'a [vk::DescriptorSetLayout],
	) -> Result<Vec<vk::DescriptorSet>> {
		let info = vk::DescriptorSetAllocateInfo::builder()
			.descriptor_pool(self.handle)
			.set_layouts(set_layouts);

		let descriptor_sets =
			unsafe { self.device.inner().allocate_descriptor_sets(&info)? };

		Ok(descriptor_sets)
	}
}

impl<'a> Drop for DescriptorPool<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device
				.inner()
				.destroy_descriptor_pool(self.handle, None)
		}
	}
}
