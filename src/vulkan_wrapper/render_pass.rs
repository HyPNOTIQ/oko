use {super::Device, anyhow::Result, ash::vk};

pub struct RenderPass<'a> {
	handle: vk::RenderPass,
	device: &'a Device<'a>,
}

impl<'a> RenderPass<'a> {
	pub fn new(
		device: &'a Device,
		create_info: &vk::RenderPassCreateInfo,
	) -> Result<Self> {
		let handle =
			unsafe { device.inner().create_render_pass(create_info, None)? };

		let render_pass = Self { handle, device };

		Ok(render_pass)
	}

	pub fn handle(&self) -> vk::RenderPass {
		self.handle
	}
}

impl<'a> Drop for RenderPass<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device.inner().destroy_render_pass(self.handle, None);
		}
	}
}
