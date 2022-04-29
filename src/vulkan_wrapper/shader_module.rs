use {super::Device, anyhow::Result, ash::vk};

pub struct ShaderModule<'a> {
	handle: vk::ShaderModule,
	device: &'a Device<'a>,
}

impl<'a> ShaderModule<'a> {
	pub fn new(device: &'a Device, path: &std::path::Path) -> Result<Self> {
		let mut file = std::fs::File::open(&path)?;

		let code = ash::util::read_spv(&mut file)?;

		let info = ash::vk::ShaderModuleCreateInfo::builder().code(&code);

		let handle = unsafe { device.inner().create_shader_module(&info, None)? };

		let shader_module = Self { handle, device };

		Ok(shader_module)
	}

	pub fn handle(&self) -> vk::ShaderModule {
		self.handle
	}
}

impl<'a> Drop for ShaderModule<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device.inner().destroy_shader_module(self.handle, None);
		}
	}
}
