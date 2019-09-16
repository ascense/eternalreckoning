use rendy::{
    factory::Factory,
    graph::render::{
        SimpleGraphicsPipeline,
        SimpleGraphicsPipelineDesc,
    },
    hal,
    hal::{
        device::Device,
    },
};

use std::{fs::File, io::BufReader};

use crate::scene::Scene;

lazy_static::lazy_static! {
    static ref VERTEX: rendy::shader::SpirvShader = rendy::shader::SourceShaderInfo::new(
        include_str!("ui.vert"),
        "ui.vert",
        rendy::shader::ShaderKind::Vertex,
        rendy::shader::SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();
    
    static ref FRAGMENT: rendy::shader::SpirvShader = rendy::shader::SourceShaderInfo::new(
        include_str!("ui.frag"),
        "ui.frag",
        rendy::shader::ShaderKind::Fragment,
        rendy::shader::SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SHADERS: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();
    
    static ref SHADER_REFLECTION: rendy::shader::SpirvReflection = SHADERS.reflect().unwrap();
}

#[derive(Clone, Debug)]
#[repr(C, align(16))]
struct UniformArgs {
    proj: nalgebra::Matrix4<f32>,
    view: nalgebra::Matrix4<f32>,
}

#[derive(Debug, Default)]
pub struct SpriteGraphicsPipelineDesc;

#[derive(Debug)]
pub struct SpriteGraphicsPipeline<B: hal::Backend> {
    texture: rendy::texture::Texture<B>,
    vbuf: rendy::resource::Escape<rendy::resource::Buffer<B>>,
    set: rendy::resource::Escape<rendy::resource::DescriptorSet<B>>,
}

impl<B> SimpleGraphicsPipelineDesc<B, Scene> for SpriteGraphicsPipelineDesc
where
    B: hal::Backend,
{
    type Pipeline = SpriteGraphicsPipeline<B>;

    fn depth_stencil(&self) -> Option<hal::pso::DepthStencilDesc> {
        None
    }

    fn load_shader_set(
        &self,
        factory: &mut Factory<B>,
        _scene: &Scene,
    ) -> rendy::shader::ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn vertices(
        &self,
    ) -> Vec<(
        Vec<hal::pso::Element<hal::format::Format>>,
        hal::pso::ElemStride,
        hal::pso::VertexInputRate,
    )> {
        return vec![SHADER_REFLECTION
            .attributes_range(..)
            .unwrap()
            .gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex)
        ];
    }

    fn layout(&self) -> rendy::util::types::Layout {
        SHADER_REFLECTION.layout().unwrap()
    }

    fn build<'a>(
        self,
        _ctx: &rendy::graph::GraphContext<B>,
        factory: &mut Factory<B>,
        queue: rendy::command::QueueId,
        _scene: &Scene,
        buffers: Vec<rendy::graph::NodeBuffer>,
        images: Vec<rendy::graph::NodeImage>,
        set_layouts: &[rendy::resource::Handle<rendy::resource::DescriptorSetLayout<B>>],
    ) -> Result<SpriteGraphicsPipeline<B>, failure::Error> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert_eq!(set_layouts.len(), 1);

        let image_reader = BufReader::new(
            File::open(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/assets/icon_attack.png"
            ))
            .map_err(|e| {
                log::error!("Unable to open {}: {:?}", "/assets/icon_attack.png", e);
                hal::pso::CreationError::Other
            })?
        );

        let texture_builder = rendy::texture::image::load_from_image(
            image_reader,
            rendy::texture::image::ImageTextureConfig {
                generate_mips: true,
                ..Default::default()
            }
        ).map_err(|e| {
            log::error!("Unable to load image: {:?}", e);
            hal::pso::CreationError::Other
        })?;

        let texture = texture_builder
            .build(
                rendy::factory::ImageState {
                    queue,
                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                    access: hal::image::Access::SHADER_READ,
                    layout: hal::image::Layout::ShaderReadOnlyOptimal,
                },
                factory,
            )
            .unwrap();

        let set = factory
            .create_descriptor_set(set_layouts[0].clone())
            .unwrap();
        
        unsafe {
            factory.device().write_descriptor_sets(vec![
                hal::pso::DescriptorSetWrite {
                    set: set.raw(),
                    binding: 0,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Image(
                        texture.view().raw(),
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )],
                },
                hal::pso::DescriptorSetWrite {
                    set: set.raw(),
                    binding: 1,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Sampler(texture.sampler().raw())],
                },
            ]);
        }

        let vbuf_size = SHADER_REFLECTION.attributes_range(..).unwrap().stride as u64 * 6;

        let mut vbuf = factory
            .create_buffer(
                rendy::resource::BufferInfo {
                    size: vbuf_size,
                    usage: hal::buffer::Usage::VERTEX,
                },
                rendy::memory::Dynamic,
            )
            .unwrap();

        unsafe {
            factory.upload_visible_buffer(
                &mut vbuf,
                0,
                &[
                    rendy::mesh::PosTex {
                        position: [-0.05, 0.9, 0.0].into(),
                        tex_coord: [0.0, 1.0].into(),
                    },
                    rendy::mesh::PosTex {
                        position: [0.05, 0.9, 0.0].into(),
                        tex_coord: [1.0, 1.0].into(),
                    },
                    rendy::mesh::PosTex {
                        position: [0.05, 0.8, 0.0].into(),
                        tex_coord: [1.0, 0.0].into(),
                    },
                    rendy::mesh::PosTex {
                        position: [-0.05, 0.9, 0.0].into(),
                        tex_coord: [0.0, 1.0].into(),
                    },
                    rendy::mesh::PosTex {
                        position: [0.05, 0.8, 0.0].into(),
                        tex_coord: [1.0, 0.0].into(),
                    },
                    rendy::mesh::PosTex {
                        position: [-0.05, 0.8, 0.0].into(),
                        tex_coord: [0.0, 0.0].into(),
                    },
                ],
            )
                .unwrap();
        }

        Ok(SpriteGraphicsPipeline { texture, vbuf, set })
    }
}

impl<B> SimpleGraphicsPipeline<B, Scene> for SpriteGraphicsPipeline<B>
where
    B: hal::Backend,
{
    type Desc = SpriteGraphicsPipelineDesc;

    fn prepare(
        &mut self,
        _factory: &rendy::factory::Factory<B>,
        _queue: rendy::command::QueueId,
        _set_layouts: &[rendy::resource::Handle<rendy::resource::DescriptorSetLayout<B>>],
        _index: usize,
        _scene: &Scene,
    ) -> rendy::graph::render::PrepareResult {
        rendy::graph::render::PrepareResult::DrawReuse
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: rendy::command::RenderPassEncoder<'_, B>,
        _index: usize,
        _scene: &Scene,
    ) {
        unsafe {
            encoder.bind_graphics_descriptor_sets(
                layout,
                0,
                std::iter::once(self.set.raw()),
                std::iter::empty::<u32>(),
            );

            encoder.bind_vertex_buffers(
                0,
                Some((self.vbuf.raw(), 0))
            );

            encoder.draw(0..6, 0..1);
        }
    }

    fn dispose(self, _factory: &mut rendy::factory::Factory<B>, _scene: &Scene) {}
}