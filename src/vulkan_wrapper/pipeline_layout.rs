use {super::Device, anyhow::Result, ash::vk};

pub struct PipelineLayout<'a> {
	handle: vk::PipelineLayout,
	device: &'a Device<'a>,
}

impl<'a> PipelineLayout<'a> {
	pub fn new(
		device: &'a Device,
		create_info: &vk::PipelineLayoutCreateInfo,
	) -> Result<Self> {
		let handle = unsafe {
			device.inner().create_pipeline_layout(&create_info, None)?
		};

		let pipeline_layout = Self { handle, device };

		Ok(pipeline_layout)
	}

	pub fn handle(&self) -> vk::PipelineLayout {
		self.handle
	}
}

impl<'a> Drop for PipelineLayout<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device
				.inner()
				.destroy_pipeline_layout(self.handle, None);
		}
	}
}
