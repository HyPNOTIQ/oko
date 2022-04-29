use ash::vk;

pub trait Pipeline {
	const BIND_POINT: vk::PipelineBindPoint;
	fn bind_point(&self) -> vk::PipelineBindPoint;
	fn handle(&self) -> vk::Pipeline;
}
