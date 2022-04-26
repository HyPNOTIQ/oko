mod viewer;
mod vulkan_wrapper;
mod wsi;

use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use std::{
	path::PathBuf,
	thread::{self},
};
use viewer::LaunchConfig;
use winit::{
	dpi::LogicalSize,
	event::{Event, WindowEvent},
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};
use wsi::PresentTarget;

const DEFAULT_SCENE_INDEX: usize = 0;

#[derive(Debug)]
pub enum CustomEvent {
	Err(anyhow::Error),
}

fn main() -> Result<()> {
	env_logger::init();
	let pkg_name = env!("CARGO_PKG_NAME");

	let args = Command::new(pkg_name)
		.arg(Arg::new("FILE").index(1).help("glTF file path"))
		.arg(
			Arg::new("INDEX")
				.long("index")
				.takes_value(true)
				.help("glTF scene index"),
		)
		.arg(
			Arg::new("RENDERDOC")
				.long("renderdoc")
				.takes_value(false)
				.help("enables RenderDoc"),
		)
		.get_matches();

	let input_file = args
		.value_of("FILE")
		.ok_or_else(|| anyhow!("No input file"))?;

	let input_file = PathBuf::from(input_file);

	let scene_index = if let Some(scene_index) = args.value_of("INDEX") {
		scene_index.parse::<usize>()? //todo
	} else {
		DEFAULT_SCENE_INDEX
	};

	let logical_window_size = LogicalSize::new(800, 600);
	let event_loop = EventLoop::<CustomEvent>::with_user_event();
	let event_loop_proxy = event_loop.create_proxy();

	let window = WindowBuilder::new()
		.with_title(pkg_name)
		.with_inner_size(logical_window_size)
		.with_resizable(false)
		.build(&event_loop)?;

	let present_target = PresentTarget::new(window);

	let (tx, rx) = std::sync::mpsc::channel();

	let viewer_thread = move || {
		let current_thread = thread::current();
		let thread_name = current_thread.name().unwrap_or_default();

		log::info!("{} thread started!", thread_name);

		let config = LaunchConfig {
			input_file,
			scene_index,
		};

		if let Err(error) = viewer::run(present_target, config, rx) {
			event_loop_proxy
				.send_event(CustomEvent::Err(error))
				.unwrap();
		}
	};

	let viewer_thread = thread::Builder::new()
		.name("viewer".into())
		.spawn(viewer_thread)?;

	let mut viewer_thread = Some(viewer_thread);

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Wait;

		let mut exit = || {
			*control_flow = ControlFlow::Exit;
			viewer_thread.take().unwrap().join().unwrap();
		};

		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => {
					log::info!("Shut down");
					tx.send(viewer::Event::Stop).unwrap();
					exit();
				}
				_ => (),
			},
			Event::UserEvent(event) => match event {
				CustomEvent::Err(error) => {
					log::error!("{}", error);
					exit();
				}
			},
			_ => (),
		};
	});
}
