use {
	super::{CommandBuffer, Device, Fence, Semaphore},
	crate::slice_from_ref,
	anyhow::Result,
	ash::vk,
};

pub struct Queue<'a> {
	handle: ash::vk::Queue,
	device: &'a Device<'a>,
	family_index: u32,
}

impl<'a> Queue<'a> {
	pub fn new(device: &'a Device, family_index: u32) -> Self {
		let index = 0;

		let handle =
			unsafe { device.inner().get_device_queue(family_index, index) };

		Self {
			handle,
			device,
			family_index,
		}
	}

	pub fn family_index(&self) -> u32 {
		self.family_index
	}

	pub fn handle(&self) -> vk::Queue {
		self.handle
	}

	pub fn submit(
		&self,
		command_buffer: &CommandBuffer,
		wait_semaphores: &[vk::Semaphore],
		wait_stages: &[vk::PipelineStageFlags],
		signal_semaphores: &[vk::Semaphore],
		fence: &Fence,
	) -> Result<()> {
		let command_buffer = command_buffer.handle();
		let command_buffer = &command_buffer;
		let command_buffer = slice_from_ref(command_buffer);

		let submit_info = vk::SubmitInfo::builder()
			.command_buffers(command_buffer)
			.wait_semaphores(wait_semaphores)
			.wait_dst_stage_mask(wait_stages)
			.signal_semaphores(signal_semaphores)
			.build();

		let submit_infos = slice_from_ref(&submit_info);

		unsafe {
			self.device.inner().queue_submit(
				self.handle,
				submit_infos,
				fence.handle(),
			)?;
		}

		Ok(())
	}
}
