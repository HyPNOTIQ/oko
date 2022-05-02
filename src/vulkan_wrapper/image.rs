use super::Allocator;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc};
use gpu_allocator::MemoryLocation;
use {super::Device, anyhow::Result, ash::vk};

pub struct Image<'a> {
	allocation: Option<Allocation>,
	allocator: &'a Allocator<'a>,
	handle: vk::Image,
	device: &'a Device<'a>,
}

impl<'a> Image<'a> {
	pub fn new(
		device: &'a Device,
		allocator: &'a Allocator,
		create_info: &vk::ImageCreateInfo,
		name: &str,
	) -> Result<Self> {
		let device_inner = device.inner();
		let handle = unsafe { device.inner().create_image(create_info, None)? };

		let image_memory_requirements =
			unsafe { device_inner.get_image_memory_requirements(handle) };

		let allocation_create_desc = AllocationCreateDesc {
			name,
			requirements: image_memory_requirements,
			location: MemoryLocation::GpuOnly,
			linear: true,
		};

		let allocation = allocator.allocate(&allocation_create_desc)?;

		unsafe {
			device_inner.bind_image_memory(
				handle,
				allocation.memory(),
				allocation.offset(),
			)?
		};

		let allocation = Some(allocation);

		let image = Self {
			handle,
			allocator,
			device,
			allocation,
		};

		Ok(image)
	}

	pub fn handle(&self) -> vk::Image {
		self.handle
	}
}

impl<'a> Drop for Image<'a> {
	fn drop(&mut self) {
		let allocation = self.allocation.take().unwrap();
		self.allocator.free(allocation).unwrap();

		unsafe {
			self.device.inner().destroy_image(self.handle, None);
		}
	}
}
