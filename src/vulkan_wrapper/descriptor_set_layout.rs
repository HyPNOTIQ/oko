use {super::Device, anyhow::Result, ash::vk};

pub struct DescriptorSetLayout<'a> {
	handle: vk::DescriptorSetLayout,
	device: &'a Device<'a>,
}

impl<'a> DescriptorSetLayout<'a> {
	pub fn new(
		device: &'a Device,
		bindings: &[vk::DescriptorSetLayoutBinding],
	) -> Result<Self> {
		let create_info =
			vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

		let handle = unsafe {
			device
				.inner()
				.create_descriptor_set_layout(&create_info, None)?
		};

		let descriptor_set_layout = Self { handle, device };

		Ok(descriptor_set_layout)
	}

	pub fn handle(&self) -> vk::DescriptorSetLayout {
		self.handle
	}
}

impl<'a> Drop for DescriptorSetLayout<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device
				.inner()
				.destroy_descriptor_set_layout(self.handle, None);
		}
	}
}
