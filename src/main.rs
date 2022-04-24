use anyhow::Result;
use winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};

fn main() -> Result<()> {
	let pkg_name = env!("CARGO_PKG_NAME");

	let logical_window_size = LogicalSize::new(800, 600);
	let event_loop = EventLoop::<()>::with_user_event();
	let event_loop_proxy = event_loop.create_proxy();

	let window = WindowBuilder::new()
		.with_title(pkg_name)
		.with_inner_size(logical_window_size)
		.with_resizable(false)
		.build(&event_loop)?;

	event_loop.run(move |event, _, control_flow| {});
}
