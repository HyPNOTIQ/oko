use ash::vk;
use {
	super::{Device, Pipeline},
	anyhow::Result,
};

pub struct GraphicsPipeline<'a> {
	handle: vk::Pipeline,
	device: &'a Device<'a>,
}

impl<'a> GraphicsPipeline<'a> {
	pub fn new(
		device: &'a Device,
		create_info: &vk::GraphicsPipelineCreateInfo,
	) -> Result<Self> {
		let pipelines = unsafe {
			device.inner().create_graphics_pipelines(
				vk::PipelineCache::null(),
				core::slice::from_ref(create_info),
				None,
			)
		};

		let handle = match pipelines {
			Ok(mut pipelines) => pipelines.pop().unwrap(),
			Err((_, err)) => return Err(err.into()),
		};

		let graphics_pipeline = Self { handle, device };

		Ok(graphics_pipeline)
	}
}

impl<'a> Drop for GraphicsPipeline<'a> {
	fn drop(&mut self) {
		unsafe {
			self.device.inner().destroy_pipeline(self.handle, None);
		}
	}
}

impl<'a> Pipeline for GraphicsPipeline<'a> {
	const BIND_POINT: vk::PipelineBindPoint = vk::PipelineBindPoint::GRAPHICS;

	fn handle(&self) -> vk::Pipeline {
		self.handle
	}

	fn bind_point(&self) -> vk::PipelineBindPoint {
		Self::BIND_POINT
	}
}
