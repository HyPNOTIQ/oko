use {
	super::vulkan_wrapper::{CreateSurface, ExtensionName, Instance, SurfaceExtent},
	anyhow::Result,
	ash::vk::Extent2D,
};

pub struct PresentTarget {
	handle: winit::window::Window,
}

impl PresentTarget {
	pub fn new(handle: winit::window::Window) -> Self {
		Self { handle }
	}

	fn extent(&self) -> Extent2D {
		let physical_size = self.handle.inner_size();

		let width = physical_size.width;
		let height = physical_size.height;

		Extent2D::builder().width(width).height(height).build()
	}
}

impl SurfaceExtent for PresentTarget {
	fn extent(&self) -> Extent2D {
		self.extent()
	}
}

impl CreateSurface for PresentTarget {
	fn create_surface(&self, instance: &Instance) -> Result<ash::vk::SurfaceKHR> {
		let surface = unsafe {
			ash_window::create_surface(
				instance.entry(),
				instance.inner(),
				&self.handle,
				None,
			)?
		};

		Ok(surface)
	}

	fn required_extensions(&self) -> Result<Vec<ExtensionName>> {
		let required_extensions =
			ash_window::enumerate_required_extensions(&self.handle)?;

		let required_extensions = required_extensions
			.iter()
			.map(|extension| extension.as_ptr())
			.collect::<Vec<_>>();

		Ok(required_extensions)
	}
}
