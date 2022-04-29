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

const VERTEX_DEFINE: &str = "VERTEX";
const FRAGMENT_DEFINE: &str = "FRAGMENT";

const GEN_FOLDER: &str = "gen";
const SHADER_ENTRY_POINT: &str = "main";

fn gen_shaderc_common_options<'a>() -> Result<shaderc::CompileOptions<'a>> {
	let mut shaderc_options = shaderc::CompileOptions::new()
		.ok_or_else(|| anyhow!("shaderc options initialization is failed!"))?;

	shaderc_options.set_warnings_as_errors();
	shaderc_options.set_optimization_level(if cfg!(debug_assertions) {
		shaderc::OptimizationLevel::Zero
	} else {
		shaderc::OptimizationLevel::Performance
	});

	#[cfg(debug_assertions)]
	shaderc_options.set_generate_debug_info();

	Ok(shaderc_options)
}

fn main() -> Result<()> {
	cargo_emit::rerun_if_env_changed!("PROFILE");
	cargo_emit::rerun_if_changed!("shaders");

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
		.ok_or_else(|| anyhow!("shaderc initialization is failed!"))?;

	let mut shaderc_vertex_options = gen_shaderc_common_options()?;
	shaderc_vertex_options.add_macro_definition(VERTEX_DEFINE, None);

	let mut shaderc_fragment_options = gen_shaderc_common_options()?;
	shaderc_fragment_options.add_macro_definition(FRAGMENT_DEFINE, None);

	fs::create_dir_all(GEN_FOLDER)?;
	fs::remove_dir_all(GEN_FOLDER)?;

	let final_folder = Path::new(GEN_FOLDER).join(if cfg!(debug_assertions) {
		"debug"
	} else {
		"release"
	});

	for entry in walkdir::WalkDir::new("shaders") {
		let entry = entry?;
		let file_type = entry.file_type();

		let path = entry.path();

		let shader = file_type
			.is_file()
			.then(|| path.extension())
			.and_then(|extension| extension)
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

		if let Some((source_type, shader_type)) = shader {
			let write_result = |data: &[u8]| -> Result<()> {
				let out_file = final_folder.join(path.with_extension(""));
				let folder = out_file.parent().unwrap();

				fs::create_dir_all(&folder)?;
				fs::write(&out_file, data)?;

				Ok(())
			};

			let source_name = path.to_str().unwrap();

			match source_type {
				SourceType::Hlsl => {
					let data = fs::read(path)?;
					let (target_profile, defines) = match shader_type {
						ShaderType::Vertex => {
							("vs_6_0", [(VERTEX_DEFINE, None)].as_slice())
						}
						ShaderType::Fragment => {
							("ps_6_0", [(FRAGMENT_DEFINE, None)].as_slice())
						}
					};

					let result = dxc_compiler.compile(
						&dxc_library.create_blob_with_encoding(&data)?,
						source_name,
						SHADER_ENTRY_POINT,
						target_profile,
						dxc_args,
						None,
						defines,
					);

					match result {
						Err(result) => {
							let error_buffer = result.0.get_error_buffer()?;
							let error =
								dxc_library.get_blob_as_string(&error_buffer.into())?;

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
					let (shader_type, additional_options) = match shader_type {
						ShaderType::Vertex => {
							(shaderc::ShaderKind::Vertex, &shaderc_vertex_options)
						}
						ShaderType::Fragment => {
							(shaderc::ShaderKind::Fragment, &shaderc_fragment_options)
						}
					};

					let result = shaderc_compiler.compile_into_spirv(
						data.as_str(),
						shader_type,
						source_name,
						SHADER_ENTRY_POINT,
						Some(additional_options),
					)?;

					let data = result.as_binary_u8();
					write_result(data)?;
				}
			}
		}
	}

	Ok(())
}
