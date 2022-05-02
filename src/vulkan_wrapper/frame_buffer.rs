use {super::Device, anyhow::Result, ash::vk};

pub struct FrameBuffer<'a> {
	handle: vk::Framebuffer,
	extent: vk::Extent2D,
	device: &'a Device<'a>,
}

impl<'a> FrameBuffer<'a> {
	pub fn new(
		device: &'a Device,
		info: &vk::FramebufferCreateInfo,
	) -> Result<Self> {
		let extent = vk::Extent2D {
			width: info.width,
			height: info.height,
		};

		let handle = unsafe { device.inner().create_framebuffer(info, None)? };

		let frame_buffer = Self {
			extent,
			handle,
			device,
		};

		Ok(frame_buffer)
	}

	pub fn handle(&self) -> vk::Framebuffer {
		self.handle
	}

	pub fn extent(&self) -> vk::Extent2D {
		self.extent
	}
}

impl<'a> Drop for FrameBuffer<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device.inner().destroy_framebuffer(self.handle, None);
		}
	}
}
