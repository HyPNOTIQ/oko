use anyhow::Result;

pub struct Scene {
	// nodes: Vec<Node>,
}

impl Scene {
	pub fn new(scene: &gltf::Scene) -> Result<Self> {
		log::info!(
			"Creating scene; index: {}, name: {}",
			scene.index(),
			scene.name().unwrap_or_default()
		);

		// let nodes = scene
		// 	.nodes()
		// 	.map(|ref node| Node::new(node, graphics_core))
		// 	.collect::<Result<Vec<_>>>()?;

		let scene = Self {
						// nodes 
				};

		Ok(scene)
	}

	pub fn render(&self) {
		// for node in self.nodes {
		// 	node.render(command_buffer)
		// }
	}
}
