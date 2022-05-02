use super::Buffer;
use super::Pipeline;
use {super::Device, anyhow::Result, ash::vk};

pub struct CommandPool<'a> {
	handle: vk::CommandPool,
	device: &'a Device<'a>,
}

impl<'a> CommandPool<'a> {
	pub fn new(
		device: &'a Device,
		create_info: &vk::CommandPoolCreateInfo,
	) -> Result<Self> {
		let handle =
			unsafe { device.inner().create_command_pool(create_info, None)? };

		let command_pool = Self { handle, device };

		Ok(command_pool)
	}

	fn allocate_command_buffers_inner(
		&self,
		count: u32,
		level: vk::CommandBufferLevel,
	) -> Result<Vec<vk::CommandBuffer>> {
		let info = vk::CommandBufferAllocateInfo::builder()
			.command_pool(self.handle)
			.level(level)
			.command_buffer_count(count);

		let command_buffers =
			unsafe { self.device.inner().allocate_command_buffers(&info)? };

		Ok(command_buffers)
	}

	fn construct_command_buffer(
		&self,
		command_buffer_handle: vk::CommandBuffer,
	) -> CommandBuffer {
		CommandBuffer {
			device: self.device,
			_command_pool: self,
			handle: command_buffer_handle,
		}
	}

	pub fn allocate_command_buffers(
		&self,
		count: u32,
		level: vk::CommandBufferLevel,
	) -> Result<Vec<CommandBuffer>> {
		let command_buffers =
			self.allocate_command_buffers_inner(count, level)?;

		let command_buffers = command_buffers
			.iter()
			.map(|command_buffer| {
				self.construct_command_buffer(*command_buffer)
			})
			.collect::<Vec<_>>();

		Ok(command_buffers)
	}

	pub fn device(&self) -> &Device {
		self.device
	}

	pub fn allocate_command_buffer(
		&self,
		level: vk::CommandBufferLevel,
	) -> Result<CommandBuffer> {
		let command_buffers = self.allocate_command_buffers_inner(1, level)?;
		Ok(self.construct_command_buffer(*command_buffers.first().unwrap()))
	}

	pub fn handle(&self) -> vk::CommandPool {
		self.handle
	}

	pub fn reset(&self) -> Result<()> {
		unsafe {
			self.device.inner().reset_command_pool(
				self.handle,
				vk::CommandPoolResetFlags::empty(),
			)?
		};

		Ok(())
	}
}

impl<'a> Drop for CommandPool<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device.inner().destroy_command_pool(self.handle, None);
		}
	}
}
pub struct CommandBuffer<'a> {
	handle: vk::CommandBuffer,
	device: &'a Device<'a>,
	_command_pool: &'a CommandPool<'a>,
}

impl<'a> CommandBuffer<'a> {
	pub fn begin(&self, info: &vk::CommandBufferBeginInfo) -> Result<()> {
		unsafe {
			self.device
				.inner()
				.begin_command_buffer(self.handle, info)?
		};

		Ok(())
	}

	pub fn draw(
		&self,
		vertex_count: u32,
		instance_count: u32,
		first_vertex: u32,
		first_instance: u32,
	) {
		unsafe {
			self.device.inner().cmd_draw(
				self.handle,
				vertex_count,
				instance_count,
				first_vertex,
				first_instance,
			)
		}
	}

	pub fn end(&self) -> Result<()> {
		unsafe { self.device.inner().end_command_buffer(self.handle)? }

		Ok(())
	}

	pub fn bind_vertex_buffers(
		&self,
		buffers: &[vk::Buffer],
		offsets: &[vk::DeviceSize],
	) {
		unsafe {
			self.device.inner().cmd_bind_vertex_buffers(
				self.handle,
				0,
				buffers,
				offsets,
			);
		}
	}

	pub fn bind_pipeline<T: Pipeline>(&self, pipeline: &T) {
		unsafe {
			self.device.inner().cmd_bind_pipeline(
				self.handle,
				T::BIND_POINT,
				pipeline.handle(),
			);
		}
	}

	pub fn copy_buffer(&self, src_buffer: &Buffer, dst_buffer: &Buffer) {
		debug_assert!(src_buffer.size() <= dst_buffer.size());

		let regions = [vk::BufferCopy::builder()
			.src_offset(src_buffer.offset() as _)
			.dst_offset(dst_buffer.offset() as _)
			.size(src_buffer.size() as _)
			.build()];

		unsafe {
			self.device.inner().cmd_copy_buffer(
				self.handle,
				src_buffer.handle(),
				dst_buffer.handle(),
				&regions,
			)
		}
	}

	pub fn begin_render_pass(
		&self,
		info: &vk::RenderPassBeginInfo,
		contents: vk::SubpassContents,
	) {
		unsafe {
			self.device.inner().cmd_begin_render_pass(
				self.handle,
				info,
				contents,
			)
		}
	}

	pub fn end_render_pass(&self) {
		unsafe { self.device.inner().cmd_end_render_pass(self.handle) }
	}

	pub fn bind_descriptor_sets(
		&self,
		bind_poit: vk::PipelineBindPoint,
		layout: vk::PipelineLayout,
		first_set: u32,
		descriptor_sets: &[vk::DescriptorSet],
		dynamic_offsets: &[u32],
	) {
		unsafe {
			self.device.inner().cmd_bind_descriptor_sets(
				self.handle,
				bind_poit,
				layout,
				first_set,
				descriptor_sets,
				dynamic_offsets,
			)
		}
	}

	pub fn handle(&self) -> vk::CommandBuffer {
		self.handle
	}
}
