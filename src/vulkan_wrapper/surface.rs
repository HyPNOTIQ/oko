use {
	super::{ExtensionName, Instance, PhysicalDevice},
	anyhow::Result,
	ash::{
		extensions::khr,
		vk,
		vk::{Extent2D, SurfaceKHR},
	},
};

pub struct Surface<'a, SurfaceOwner> {
	handle: SurfaceKHR,
	extension: khr::Surface,
	_instance: &'a Instance,
	owner: SurfaceOwner,
}

impl<'a, T> Surface<'a, T> {
	pub fn new(owner: T, instance: &'a Instance) -> Result<Self>
	where
		T: CreateSurface,
	{
		let extension = khr::Surface::new(instance.entry(), instance.inner());

		let handle = owner.create_surface(instance)?;

		let surface = Self {
			extension,
			handle,
			_instance: instance,
			owner,
		};

		Ok(surface)
	}

	pub fn handle(&self) -> SurfaceKHR {
		self.handle
	}

	pub fn extent(&self) -> Extent2D
	where
		T: SurfaceExtent,
	{
		self.owner.extent()
	}

	pub fn physical_device_support(
		&self,
		physical_device: &PhysicalDevice,
		queue_index: u32,
	) -> bool {
		let support = unsafe {
			self.extension.get_physical_device_surface_support(
				physical_device.handle,
				queue_index,
				self.handle,
			)
		};

		support.unwrap_or(false)
	}

	pub fn capabilities(
		&self,
		physical_device: &PhysicalDevice,
	) -> Result<vk::SurfaceCapabilitiesKHR> {
		let capabilities = unsafe {
			self.extension.get_physical_device_surface_capabilities(
				physical_device.handle,
				self.handle,
			)?
		};

		Ok(capabilities)
	}

	pub fn formats(
		&self,
		physical_device: &PhysicalDevice,
	) -> Result<Vec<vk::SurfaceFormatKHR>> {
		let formats = unsafe {
			self.extension.get_physical_device_surface_formats(
				physical_device.handle,
				self.handle,
			)?
		};

		Ok(formats)
	}

	pub fn present_modes(
		&self,
		physical_device: &PhysicalDevice,
	) -> Result<Vec<vk::PresentModeKHR>> {
		let present_modes = unsafe {
			self.extension.get_physical_device_surface_present_modes(
				physical_device.handle,
				self.handle,
			)?
		};

		Ok(present_modes)
	}
}

impl<'a, T> Drop for Surface<'a, T> {
	fn drop(&mut self) {
		unsafe {
			self.extension.destroy_surface(self.handle, None);
		}
	}
}

pub trait SurfaceExtent {
	fn extent(&self) -> Extent2D;
}

pub trait CreateSurface: SurfaceExtent {
	fn create_surface(&self, instance: &Instance) -> Result<SurfaceKHR>;
	fn required_extensions(&self) -> Result<Vec<ExtensionName>>;
}
