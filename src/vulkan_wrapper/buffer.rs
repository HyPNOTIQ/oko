use super::Allocator;
use crate::slice_from_ref;
use gpu_allocator::{vulkan::Allocation, vulkan::AllocationCreateDesc};
use {super::Device, anyhow::Result, ash::vk};

pub struct Buffer<'a> {
	size: usize,
	allocation: Option<Allocation>,
	allocator: &'a Allocator<'a>,
	handle: vk::Buffer,
	device: &'a Device<'a>,
}

impl<'a> Buffer<'a> {
	pub fn new(
		device: &'a Device,
		allocator: &'a Allocator,
		create_info: &vk::BufferCreateInfo,
		location: gpu_allocator::MemoryLocation,
		name: &str,
	) -> Result<Self> {
		let device_inner = device.inner();
		let handle =
			unsafe { device.inner().create_buffer(create_info, None)? };

		let image_memory_requirements =
			unsafe { device_inner.get_buffer_memory_requirements(handle) };

		let allocation_create_desc = AllocationCreateDesc {
			name,
			requirements: image_memory_requirements,
			location,
			linear: true,
		};

		let allocation = allocator.allocate(&allocation_create_desc)?;

		unsafe {
			device_inner.bind_buffer_memory(
				handle,
				allocation.memory(),
				allocation.offset(),
			)?
		};

		let allocation = Some(allocation);

		let image = Self {
			size: create_info.size as _,
			handle,
			allocator,
			device,
			allocation,
		};

		Ok(image)
	}

	pub fn offset(&self) -> usize {
		self.allocation.as_ref().unwrap().offset() as _
	}

	pub fn device_memory(&self) -> vk::DeviceMemory {
		unsafe { self.allocation.as_ref().unwrap().memory() }
	}

	pub fn allocation_size(&self) -> usize {
		self.allocation.as_ref().unwrap().size() as _
	}

	pub fn size(&self) -> usize {
		self.size
	}

	pub fn handle(&self) -> vk::Buffer {
		self.handle
	}

	pub fn mapped_slice_mut(&mut self) -> Result<&mut [u8]> {
		let mapped_slice = self
			.allocation
			.as_mut()
			.unwrap()
			.mapped_slice_mut()
			.ok_or_else(|| {
				anyhow::anyhow!(
					"Attempt to access non mapped memory,
				 i.e. memory is not host visible"
				)
			})?;

		Ok(&mut mapped_slice[0..self.size])
	}

	pub fn copy_into<T: Sized>(&mut self, data: &T) -> Result<()> {
		let slice = unsafe {
			std::slice::from_raw_parts(
				(data as *const T) as *const u8,
				std::mem::size_of::<T>(),
			)
		};

		self.mapped_slice_mut()?.copy_from_slice(slice);

		Ok(())
	}

	pub fn flush(&self) -> Result<()> {
		let ranges = vk::MappedMemoryRange::builder()
			.memory(self.device_memory())
			.offset(self.offset() as _)
			.size(self.size() as _)
			.build();

		unsafe {
			self.device
				.inner()
				.flush_mapped_memory_ranges(slice_from_ref(&ranges))?
		};

		Ok(())
	}

	pub fn copy_into_n_flush<T: Sized>(&mut self, data: &T) -> Result<()> {
		self.copy_into(data)?;
		self.flush()?;

		Ok(())
	}
}

impl<'a> Drop for Buffer<'a> {
	fn drop(&mut self) {
		self.allocator
			.free(self.allocation.take().unwrap())
			.unwrap();

		unsafe {
			self.device.inner().destroy_buffer(self.handle, None);
		}
	}
}
