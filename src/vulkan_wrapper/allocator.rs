use super::Device;
use super::Instance;
use super::PhysicalDevice;
use anyhow::{Ok, Result};

pub struct Allocator<'a> {
	allocator: gpu_allocator::vulkan::Allocator,
	_device: &'a Device<'a>,
}

impl<'a> Allocator<'a> {
	pub fn new(
		instance: &'a Instance,
		device: &'a Device,
		physical_device: &PhysicalDevice,
		buffer_device_address: bool,
	) -> Result<Self> {
		let allocator = gpu_allocator::vulkan::Allocator::new(
			&gpu_allocator::vulkan::AllocatorCreateDesc {
				instance: instance.inner().clone(),
				device: device.inner().clone(),
				physical_device: physical_device.handle,
				debug_settings: Default::default(),
				buffer_device_address,
			},
		)?;

		let allocator = Self {
			allocator,
			_device: device,
		};

		Ok(allocator)
	}
}
