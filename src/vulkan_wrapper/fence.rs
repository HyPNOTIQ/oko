use {
	super::Device,
	anyhow::Result,
	ash::vk::{self, FenceCreateFlags},
};

pub struct Fence<'a> {
	handle: vk::Fence,
	device: &'a Device<'a>,
}

impl<'a> Fence<'a> {
	pub fn new(device: &'a Device, signaled: bool) -> Result<Self> {
		let signaled = FenceCreateFlags::SIGNALED.as_raw() * signaled as vk::Flags;
		let signaled = FenceCreateFlags::from_raw(signaled);
		let flags = signaled;

		let create_info = vk::FenceCreateInfo::builder().flags(flags);

		let handle = unsafe { device.inner().create_fence(&create_info, None)? };

		let fence = Self { handle, device };

		Ok(fence)
	}

	pub fn wait(&self, timeout: u64) -> Result<()> {
		let fences = core::slice::from_ref(&self.handle);
		let wait_all = true;

		unsafe {
			self.device
				.inner()
				.wait_for_fences(&fences, wait_all, timeout)?
		}

		Ok(())
	}

	pub fn reset(&self) -> Result<()> {
		let fences = core::slice::from_ref(&self.handle);

		unsafe { self.device.inner().reset_fences(&fences)? }

		Ok(())
	}

	pub fn handle(&self) -> vk::Fence {
		self.handle
	}
}

impl<'a> Drop for Fence<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device.inner().destroy_fence(self.handle, None);
		}
	}
}
