use {
    // super::{
    //     obsydian::{
    //         GraphicsPipeline,
    //         IntoSurface,
    //         CommandBuffer,
    //     },
    //     GraphicsCore,
    //     PrimitivePipelineInfo,
    // },

    crate::obsydian::CommandBuffer,

    gltf::{
        accessor::{
            Dimensions,
            DataType,
            Accessor,
        },
        buffer::{
            View
        },
        Semantic,
    },

    ash::vk::{
        self,
        Format,
    },
    std::rc::Rc,
    anyhow::Result,
};

pub struct Primitive
{
    // pipeline: Rc<GraphicsPipeline>,
}

impl Primitive
{
    const MAX_ACCESSORS : usize = 8;

    pub fn new(
        primitive: &gltf::Primitive,
        // graphics_core: &GraphicsCore<SurfaceOwner>,
    ) -> Result<Self>
    {
        log::info!("Creating primitive; index: {}", primitive.index());

        let topology = Self::topology(primitive)?;

        use std::mem::{self, MaybeUninit};

        let mut binding_descriptions: [
            MaybeUninit<vk::VertexInputBindingDescription>;
            Self::MAX_ACCESSORS
        ] = unsafe {
            MaybeUninit::uninit().assume_init()
        };

        let mut attribute_descriptions: [
            MaybeUninit<vk::VertexInputAttributeDescription>;
            Self::MAX_ACCESSORS
        ] = unsafe {
            MaybeUninit::uninit().assume_init()
        };

        let mut index = 0_usize;

        // positions
        let positions = match primitive.get(&Semantic::Positions)
        {
            Some(positions) => positions,
            None => Err(anyhow::anyhow!("Primitive has no positions!"))?
        };

        let (binding_description, attribute_description) = Self::description(
            &positions,
            index as _,
            GraphicsCore::<SurfaceOwner>::POSITIONS_LOCATION,
        )?;

        binding_descriptions[index] = MaybeUninit::new(binding_description);
        attribute_descriptions[index] = MaybeUninit::new(attribute_description);
        index += 1;

        // normals
        if let Some(normals) = primitive.get(&Semantic::Normals)
        {
            let (binding_description, attribute_description) = Self::description(
                &normals,
                index as _,
                GraphicsCore::<SurfaceOwner>::NORMALS_LOCATION,
            )?;

            binding_descriptions[index] = MaybeUninit::new(binding_description);
            attribute_descriptions[index] = MaybeUninit::new(attribute_description);
            index += 1;
        }

        // tex_coords
        if let Some(tex_coords) = primitive.get(&Semantic::TexCoords(0))
        {
            let (binding_description, attribute_description) = Self::description(
                &tex_coords,
                index as _,
                GraphicsCore::<SurfaceOwner>::TEX_COORDS_LOCATION,
            )?;

            binding_descriptions[index] = MaybeUninit::new(binding_description);
            attribute_descriptions[index] = MaybeUninit::new(attribute_description);
            index += 1;
        }

        TODO: process all accessors

        let binding_descriptions = unsafe { 
            mem::transmute::<_, [vk::VertexInputBindingDescription; Self::MAX_ACCESSORS]>(binding_descriptions)
        };

        let attribute_descriptions = unsafe { 
            mem::transmute::<_, [vk::VertexInputAttributeDescription; Self::MAX_ACCESSORS]>(attribute_descriptions)
        };

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding_descriptions[0..index])
            .vertex_attribute_descriptions(&attribute_descriptions[0..index])
            .build();

        let pipeline_info = PrimitivePipelineInfo::new(
            topology,
            vertex_input_state_create_info,
        );

        let pipeline = graphics_core.acquire_primitive_pipeline(&pipeline_info)?;

        let primitive = Self
        {
            // pipeline,
        };

        Ok(primitive)
    }

    // pub fn pipeline(&self)
    // -> &GraphicsPipeline
    // {
    //     &self.pipeline
    // }

    // pub fn render(
    //     &self,
    //     command_buffer: &CommandBuffer,
    // )
    // {
    //     let pipeline: &GraphicsPipeline = &self.pipeline;
    //     command_buffer.bind_pipeline(pipeline);

        

    // }

    fn description(
        accessor: &Accessor,
        binding: u32,
        location: u32,
    ) -> Result<(
        vk::VertexInputBindingDescription,
        vk::VertexInputAttributeDescription
    )>
    {
        let buffer_view = Self::buffer_view(accessor)?;

        let buffer_index = buffer_view.buffer().index();
        let stride = match buffer_view.stride()
        {
            Some(stride) => stride,
            None => accessor.size(),
        };

        let binding_description = vk::VertexInputBindingDescription::builder()
            .binding(buffer_index as _)
            .stride(stride as _)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build();

        let data_type = accessor.data_type();
        let dimensions = accessor.dimensions();
        let format = Self::attribute_format(
            data_type, 
            dimensions
        )?;

        let offset = accessor.offset() + buffer_view.offset();
        
        let attribute_desctiption = vk::VertexInputAttributeDescription::builder()
            .binding(binding)
            .location(location)
            .format(format)
            .offset(offset as _)
            .build();

        let description = (binding_description, attribute_desctiption);
        Ok(description)
    }

    fn buffer_view<'a>(accessor: &Accessor<'a>
    ) -> Result<View<'a>>
    {
        let buffer_view = accessor.view().ok_or(
            anyhow::anyhow!("Sparse accessor is unsupported!")
        )?;

        Ok(buffer_view)
    }

    fn binding_description(accessor: &Accessor)
    -> Result<vk::VertexInputBindingDescription>
    {
        let buffer_view = Self::buffer_view(accessor)?;

        let buffer_index = buffer_view.buffer().index();
        let stride = match buffer_view.stride()
        {
            Some(stride) => stride,
            None => accessor.size(),
        };

        let description = vk::VertexInputBindingDescription::builder()
            .binding(buffer_index as _)
            .stride(stride as _)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build();

        Ok(description)
    }

    fn attribute_description(
        accessor: &Accessor,
        binding: u32,
        location: u32,
    ) -> Result<vk::VertexInputAttributeDescription>
    {
        let buffer_view = Self::buffer_view(accessor)?;

        let data_type = accessor.data_type();
        let dimensions = accessor.dimensions();
        let format = Self::attribute_format(
            data_type, 
            dimensions
        )?;

        let offset = accessor.offset() + buffer_view.offset();
        
        let desctiption = vk::VertexInputAttributeDescription::builder()
            .binding(binding)
            .location(location)
            .format(format)
            .offset(offset as _)
            .build();

        Ok(desctiption)
    }

    fn topology(primitive: &gltf::Primitive)
    -> Result<vk::PrimitiveTopology>
    {
        use vk::PrimitiveTopology;

        use gltf::mesh::Mode;
        
        let topology = match primitive.mode()
        {
            Mode::Points => PrimitiveTopology::POINT_LIST,
            Mode::Lines => PrimitiveTopology::LINE_LIST,
            Mode::LineStrip => PrimitiveTopology::LINE_STRIP,
            Mode::Triangles => PrimitiveTopology::TRIANGLE_LIST,
            Mode::TriangleStrip => PrimitiveTopology::TRIANGLE_STRIP,
            Mode::TriangleFan => PrimitiveTopology::TRIANGLE_FAN,

            _ => {
                // There is only one unhandled topology mode - LineLoop
                // TODO: investigate how properly implement support LineLoop for Vulkan
                Err(anyhow::anyhow!("Unsupported primitive topology!"))?
            }

        };

        Ok(topology)
    }

    fn attribute_format(
        data_type: DataType,
        dimensions: Dimensions,
    ) -> Result<vk::Format>
    {
        let attribute_format = match dimensions
        {
            Dimensions::Scalar =>
            {
                match data_type
                {
                    DataType::I8 => Format::R8_SINT,
                    DataType::U8 => Format::R8_UINT,
                    DataType::I16 => Format::R16_SINT,
                    DataType::U16 => Format::R16_UINT,
                    DataType::U32 => Format::R32_UINT,
                    DataType::F32 => Format::R32_SFLOAT,
                }
            },
            Dimensions::Vec2 =>
            {
                match data_type
                {
                    DataType::I8 => Format::R8G8_SINT,
                    DataType::U8 => Format::R8G8_UINT,
                    DataType::I16 => Format::R16G16_SINT,
                    DataType::U16 => Format::R16G16_UINT,
                    DataType::U32 => Format::R32G32_UINT,
                    DataType::F32 => Format::R32G32_SFLOAT,
                }
            },
            Dimensions::Vec3 =>
            {
                match data_type
                {
                    DataType::I8 => Format::R8G8B8_SINT,
                    DataType::U8 => Format::R8G8B8_UINT,
                    DataType::I16 => Format::R16G16B16_SINT,
                    DataType::U16 => Format::R16G16B16_UINT,
                    DataType::U32 => Format::R32G32B32_UINT,
                    DataType::F32 => Format::R32G32B32_SFLOAT,
                }
            },
            Dimensions::Vec4 =>
            {
                match data_type
                {
                    DataType::I8 => Format::R8G8B8A8_SINT,
                    DataType::U8 => Format::R8G8B8A8_UINT,
                    DataType::I16 => Format::R16G16B16A16_SINT,
                    DataType::U16 => Format::R16G16B16A16_UINT,
                    DataType::U32 => Format::R32G32B32A32_UINT,
                    DataType::F32 => Format::R32G32B32A32_SFLOAT,
                }
            },
            _ => Err(anyhow::anyhow!("Unsupported accessor type!"))?
        };

        Ok(attribute_format)
    }
}