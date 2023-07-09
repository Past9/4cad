use super::{GraphicsStage, ScreenSpaceVertex};
use std::sync::Arc;
use vulkano::{
    buffer::Subbuffer,
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, SubpassContents},
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    image::{view::ImageView, AttachmentImage, SampleCount},
    pipeline::{
        graphics::{
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::Vertex,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, StateMode,
    },
    render_pass::{RenderPass, Subpass},
};

pub(super) struct Inputs {
    pub opaque_image: Arc<ImageView<AttachmentImage>>,
    pub translucent_accum_image: Arc<ImageView<AttachmentImage>>,
    pub translucent_transmit_image: Arc<ImageView<AttachmentImage>>,
    pub quad_vertices: Subbuffer<[ScreenSpaceVertex]>,
    pub quad_indices: Subbuffer<[u32]>,
}

pub(super) struct CompositingStage {
    pipeline: Arc<GraphicsPipeline>,
}
impl CompositingStage {
    pub fn new(device: Arc<Device>, render_pass: Arc<RenderPass>, samples: SampleCount) -> Self {
        Self {
            pipeline: Self::build_pipeline(device, render_pass, samples),
        }
    }

    pub fn build_pipeline(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        msaa_samples: SampleCount,
    ) -> Arc<GraphicsPipeline> {
        GraphicsPipeline::start()
            .vertex_input_state(ScreenSpaceVertex::per_vertex())
            .vertex_shader(
                compositing_vs::load(device.clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
                (),
            )
            .input_assembly_state(
                InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
            )
            .rasterization_state(RasterizationState {
                front_face: StateMode::Fixed(FrontFace::CounterClockwise),
                cull_mode: StateMode::Fixed(CullMode::None),
                ..RasterizationState::default()
            })
            .multisample_state(MultisampleState {
                rasterization_samples: msaa_samples,
                sample_shading: Some(0.5),
                ..Default::default()
            })
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(
                compositing_fs::load(device.clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
                (),
            )
            .render_pass(Subpass::from(render_pass.clone(), 4).unwrap())
            .build(device.clone())
            .unwrap()
    }
}
impl GraphicsStage<Inputs> for CompositingStage {
    fn pipeline(&self) -> Arc<GraphicsPipeline> {
        self.pipeline.clone()
    }

    fn layout(&self) -> Arc<PipelineLayout> {
        self.pipeline.layout().clone()
    }

    fn add_commands(
        &self,
        inputs: Inputs,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
    ) {
        builder
            .next_subpass(SubpassContents::Inline)
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone());

        let compositing_descriptor_set = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            self.pipeline.layout().set_layouts().get(0).unwrap().clone(),
            [
                WriteDescriptorSet::image_view(0, inputs.opaque_image.clone()),
                WriteDescriptorSet::image_view(1, inputs.translucent_accum_image.clone()),
                WriteDescriptorSet::image_view(2, inputs.translucent_transmit_image.clone()),
            ],
        )
        .unwrap();

        builder
            .bind_vertex_buffers(0, inputs.quad_vertices.clone())
            .bind_index_buffer(inputs.quad_indices.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                compositing_descriptor_set.clone(),
            )
            .draw_indexed(inputs.quad_indices.len() as u32, 1, 0, 0, 0)
            .unwrap();
    }
}

mod compositing_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
    }
}

mod compositing_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/compositing.frag",
    }
}
