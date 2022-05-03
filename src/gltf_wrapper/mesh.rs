use {
    super::{
        obsydian::{
            CommandBuffer,
            IntoSurface,
        },
        GraphicsCore,
        Primitive,
    },

    anyhow::Result,
};

pub struct Mesh
{
    primiteves: Vec<Primitive>,
}

impl Mesh
{
    pub fn new<SurfaceOwner: IntoSurface>(
        mesh: &gltf::Mesh,
        graphics_core: &GraphicsCore<SurfaceOwner>,
    ) -> Result<Self>
    {
        log::info!("Creating mesh; index: {}, name: {}", mesh.index(), mesh.name().unwrap_or_default());

        let primiteves = mesh.primitives().map(
            |ref primitive|
            {
                Primitive::new(
                    primitive,
                    &graphics_core
                )
            }
        ).collect::<Result<Vec<_>>>()?;

        let mesh = Self
        {
            primiteves,
        };

        Ok(mesh)
    }

    pub fn render(
        &self,
        command_buffer: &CommandBuffer,
    )
    {
        for primiteve in &self.primiteves
        {
            primiteve.render(command_buffer)
        }
    }
}