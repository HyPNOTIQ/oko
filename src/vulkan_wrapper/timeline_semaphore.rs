use crate::slice_from_ref;
use {super::Device, anyhow::Result, ash::vk};

pub struct TimelineSemaphore<'a> {
	handle: vk::Semaphore,
	device: &'a Device<'a>,
}

impl<'a> TimelineSemaphore<'a> {
	pub fn new(device: &'a Device, initial_value: u64) -> Result<Self> {
		let mut type_create_info = vk::SemaphoreTypeCreateInfo::builder()
			.initial_value(initial_value)
			.semaphore_type(vk::SemaphoreType::TIMELINE);

		let create_info =
			vk::SemaphoreCreateInfo::builder().push_next(&mut type_create_info);

		let handle =
			unsafe { device.inner().create_semaphore(&create_info, None)? };

		let semaphore = Self { handle, device };

		Ok(semaphore)
	}

	pub fn handle(&self) -> vk::Semaphore {
		self.handle
	}

	pub fn signal(&self, value: u64) -> Result<()> {
		let info = vk::SemaphoreSignalInfo::builder()
			.semaphore(self.handle)
			.value(value);

		unsafe { self.device.inner().signal_semaphore(&info)? };

		Ok(())
	}

	pub fn wait_max_timeout(&self, value: u64) -> Result<()> {
		self.wait(value, u64::MAX)?;

		Ok(())
	}

	pub fn wait(&self, value: u64, timeout: u64) -> Result<()> {
		let semaphores = slice_from_ref(&self.handle);
		let values = slice_from_ref(&value);

		let info = vk::SemaphoreWaitInfo::builder()
			.values(values)
			.semaphores(semaphores);

		unsafe { self.device.inner().wait_semaphores(&info, timeout)? };

		Ok(())
	}
}

impl<'a> Drop for TimelineSemaphore<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device.inner().destroy_semaphore(self.handle, None);
		}
	}
}
