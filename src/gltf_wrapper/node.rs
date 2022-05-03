use {
    super::{
        obsydian::{
            IntoSurface,
            Buffer,
            GraphicsPipeline,
            CommandBuffer,
        },
        Mesh,
        Scene,
        GraphicsCore,
        PrimitivePipelineInfo,
        Primitive,
    },

    ash::{
        vk,
    },

    std::rc::Rc,
    anyhow::Result,

    gltf::Gltf,
};

pub struct Node
{
    mesh: Option<Mesh>,
    children: Vec<Node>,
}

impl Node
{
    pub fn new<SurfaceOwner: IntoSurface>(
        node: &gltf::Node,
        graphics_core: &GraphicsCore<SurfaceOwner>,
    )
    -> Result<Self>
    {
        log::info!("Creating node; index: {}, name: {}", node.index(), node.name().unwrap_or_default());

        let mesh = match node.mesh()
        {
            Some(ref mesh) => {
                let mesh = Mesh::new(
                    mesh,
                    &graphics_core
                )?;
                Some(mesh)
            },
            None => {
                None
            }
        };

        let children = node.children().map(
            |ref node|
            {
                Node::new(
                    node,
                    graphics_core
                )
            }
        ).collect::<Result<Vec<_>>>()?;

        let node = Self
        {
            mesh,
            children,
        };

        Ok(node)
    }

    pub fn render(
        &self,
        command_buffer: &CommandBuffer,
    )
    {
        if let Some(mesh) = &self.mesh
        {
            mesh.render(command_buffer)
        }

        for node in &self.children
        {
            node.render(command_buffer)
        }
    }
}