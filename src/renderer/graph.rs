use rendy::{
    graph::render::{
        RenderGroupBuilder,
        SimpleGraphicsPipeline,
    },
    hal,
};

use crate::scene::Scene;
use super::pipeline::TriangleRenderPipeline;

pub struct RenderGraph<B: hal::Backend> {
    graph: rendy::graph::Graph<B, Scene>,
}

impl<B> RenderGraph<B>
where
    B: hal::Backend,
{
    pub fn new(
        mut factory: &mut rendy::factory::Factory<B>,
        mut families: &mut rendy::command::Families<B>,
        mut scene: &mut Scene,
        window: &crate::window::Window,
    ) -> RenderGraph<B> {
        let surface = window.create_surface(&mut factory);
        
        let mut graph_builder = rendy::graph::GraphBuilder::<B, Scene>::new();

        let size = window.get_size();
        
        let color = graph_builder.create_image(
            hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1),
            1,
            factory.get_surface_format(&surface),
            Some(hal::command::ClearValue::Color([1.0, 1.0, 1.0, 1.0].into())),
        );

        let pass = graph_builder.add_node(
            TriangleRenderPipeline::builder()
                .into_subpass()
                .with_color(color)
                .into_pass(),
        );

        graph_builder.add_node(
            rendy::graph::present::PresentNode::builder(&factory, surface, color).with_dependency(pass),
        );

        let graph = graph_builder
            .build(&mut factory, &mut families, &mut scene)
            .unwrap();

        RenderGraph { graph }
    }

    pub fn run(
        &mut self,
        factory: &mut rendy::factory::Factory<B>,
        families: &mut rendy::command::Families<B>,
        scene: &Scene,
    ) {
        self.graph.run(factory, families, &scene);
    }

    pub fn dispose(
        self,
        factory: &mut rendy::factory::Factory<B>,
        scene: &Scene,
    ) {
        self.graph.dispose(factory, &scene);
    }
}