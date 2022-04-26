use {
	super::Device,
	anyhow::Result,
	ash::vk,
};

pub struct ImageView<'a> {
	handle: vk::ImageView,
	device: &'a Device<'a>,
}

impl<'a> ImageView<'a> {
	pub fn new(
		device: &'a Device,
		create_info: &vk::ImageViewCreateInfo,
	) -> Result<Self> {
		let handle = unsafe { device.inner().create_image_view(create_info, None)? };

		let image_view = Self { handle, device };

		Ok(image_view)
	}

	pub fn handle(&self) -> vk::ImageView {
		self.handle
	}
}

impl<'a> Drop for ImageView<'a> {
	fn drop(&mut self) {
		unsafe { self.device.inner().destroy_image_view(self.handle, None) }
	}
}
