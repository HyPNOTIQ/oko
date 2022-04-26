use {
	anyhow::{anyhow, bail, Result},
	hassle_rs::wrapper::Dxc,
	std::{fs, path::Path},
};

enum SourceType {
	Hlsl,
	Glsl,
}

enum ShaderType {
	Vertex,
	Fragment,
}

const VERTEX_EXTENSION: &str = "vert";
const FRAGMENT_EXTENSION: &str = "frag";

const HLSL_EXTENSION: &str = "hlsl";
const GLSL_EXTENSION: &str = "glsl";

const GEN_FOLDER: &str = "gen";

fn main() -> Result<()> {
	println!("cargo:rerun-if-changed=shaders");

	// DXC
	let dxc = Dxc::new(None)?;
	let dxc_compiler = dxc.create_compiler()?;
	let dxc_library = dxc.create_library()?;

	// -spirv => Generate SPIR-V code
	// -WX => Treat warnings as errors
	// -Od => Disable optimizations
	// -Zi => Enable debug information. Cannot be used together with -Zs
	// "dxc --help" for more info
	let dxc_args = if cfg!(debug_assertions) {
		["-spirv", "-WX", "-Od", "-Zi"].as_slice()
	} else {
		["-spirv", "-WX"].as_slice()
	};

	// shaderc
	let shaderc_compiler = shaderc::Compiler::new()
		.ok_or_else(|| anyhow!("shaderc compiler initialization is failed!"))?;

	let mut shaderc_options = shaderc::CompileOptions::new()
		.ok_or_else(|| anyhow!("shaderc compiler options initialization is failed!"))?;

	let shaderc_optimization_level = if cfg!(debug_assertions) {
		shaderc::OptimizationLevel::Zero
	} else {
		shaderc::OptimizationLevel::Performance
	};

	shaderc_options.set_optimization_level(shaderc_optimization_level);
	shaderc_options.set_warnings_as_errors();

	#[cfg(debug_assertions)]
	shaderc_options.set_generate_debug_info();

	fs::create_dir_all(GEN_FOLDER)?;
	fs::remove_dir_all(GEN_FOLDER)?;
	for entry in walkdir::WalkDir::new("shaders") {
		let entry = entry?;
		let file_type = entry.file_type();

		let path = entry.path();

		// todo https://doc.rust-lang.org/std/primitive.bool.html#method.then_some
		let shader = if file_type.is_file() { Some(()) } else { None }
			.and_then(|()| path.extension())
			.and_then(|extension| extension.to_str())
			.and_then(|extension| match extension {
				HLSL_EXTENSION => Some(SourceType::Hlsl),
				GLSL_EXTENSION => Some(SourceType::Glsl),
				_ => None,
			})
			.and_then(|source_type| {
				path.file_stem()
					.and_then(|file_stem| Path::new(file_stem).extension())
					.and_then(|extension| extension.to_str())
					.and_then(|extension| match extension {
						VERTEX_EXTENSION => {
							let shader = (source_type, ShaderType::Vertex);
							Some(shader)
						}
						FRAGMENT_EXTENSION => {
							let shader = (source_type, ShaderType::Fragment);
							Some(shader)
						}
						_ => None,
					})
			});

		if let Some(shader) = shader {
			let (source_type, shader_type) = shader;

			let write_result = |data: &[u8]| -> Result<()> {
				let out_file = Path::new(GEN_FOLDER).join(path.with_extension("spv"));
				let folder = out_file.parent().ok_or_else(|| anyhow!(""))?;

				fs::create_dir_all(&folder)?;
				fs::write(&out_file, data)?;

				Ok(())
			};

			let source_name = path.to_str().ok_or_else(|| anyhow!(""))?;

			match source_type {
				SourceType::Hlsl => {
					let data = fs::read(path)?;
					let target_profile = match shader_type {
						ShaderType::Vertex => "vs_6_0",
						ShaderType::Fragment => "ps_6_0",
					};

					let result = dxc_compiler.compile(
						&dxc_library.create_blob_with_encoding(&data)?,
						source_name,
						"main",
						target_profile,
						dxc_args,
						None,
						&[],
					);

					match result {
						Err(result) => {
							let error_buffer = result.0.get_error_buffer()?;
							let error = dxc_library.get_blob_as_string(&error_buffer.into())?;
							bail!("{}", error);
						}
						Ok(result) => {
							let result = result.get_result()?;
							let data: &[u8] = result.as_slice();
							write_result(data)?;
						}
					};
				}
				SourceType::Glsl => {
					let data = fs::read_to_string(path)?;
					let shader_type = match shader_type {
						ShaderType::Vertex => shaderc::ShaderKind::Vertex,
						ShaderType::Fragment => shaderc::ShaderKind::Fragment,
					};

					let result = shaderc_compiler.compile_into_spirv(
						data.as_str(),
						shader_type,
						source_name,
						"main",
						Some(&shaderc_options),
					)?;

					let data = result.as_binary_u8();
					write_result(data)?;
				}
			}
		}
	}

	Ok(())
}
