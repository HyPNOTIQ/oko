use std::cell::RefCell;
use std::rc::Rc;

use super::Device;
use super::Instance;
use super::PhysicalDevice;
use anyhow::{Ok, Result};

pub struct Allocator<'a> {
	allocator: Rc<RefCell<gpu_allocator::vulkan::Allocator>>,
	_device: &'a Device<'a>,
}

impl<'a> Allocator<'a> {
	pub fn new(
		instance: &Instance,
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

		let allocator = Rc::new(RefCell::new(allocator));

		let allocator = Self {
			allocator,
			_device: device,
		};

		Ok(allocator)
	}

	pub fn allocate(
		&self,
		desc: &gpu_allocator::vulkan::AllocationCreateDesc,
	) -> Result<gpu_allocator::vulkan::Allocation> {
		let allocation = self.allocator.borrow_mut().allocate(desc)?;

		Ok(allocation)
	}

	pub fn free(
		&self,
		allocation: gpu_allocator::vulkan::Allocation,
	) -> Result<()> {
		self.allocator.borrow_mut().free(allocation)?;

		Ok(())
	}
}
