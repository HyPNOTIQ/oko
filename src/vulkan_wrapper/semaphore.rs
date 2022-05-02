use std::ops::Deref;

use {super::Device, anyhow::Result, ash::vk};

pub struct Semaphore<'a> {
	handle: vk::Semaphore,
	device: &'a Device<'a>,
}

impl<'a> Semaphore<'a> {
	pub fn new(device: &'a Device) -> Result<Self> {
		let create_info = vk::SemaphoreCreateInfo::builder();

		let handle = unsafe { device.inner().create_semaphore(&create_info, None)? };

		let semaphore = Self { handle, device };

		Ok(semaphore)
	}

	pub fn handle(&self) -> vk::Semaphore {
		self.handle
	}
}

impl<'a> Drop for Semaphore<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device.inner().destroy_semaphore(self.handle, None);
		}
	}
}
