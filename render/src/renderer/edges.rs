use super::GraphicsStage;
use crate::model::BufferedEdgeVertex;
use std::sync::Arc;
use vulkano::{
    buffer::Subbuffer,
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer, SubpassContents},
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::Device,
    image::SampleCount,
    pipeline::{
        graphics::{
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, LineRasterizationMode, RasterizationState},
            vertex_input::Vertex,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline, PipelineLayout, StateMode,
    },
    render_pass::{RenderPass, Subpass},
};

pub(super) struct Inputs<'a> {
    pub vertices: &'a Option<Subbuffer<[BufferedEdgeVertex]>>,
    pub indices: &'a Option<Subbuffer<[u32]>>,
    pub show: bool,
}

pub(super) struct EdgeStage {
    pipeline: Arc<GraphicsPipeline>,
}
impl EdgeStage {
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
            .vertex_input_state(BufferedEdgeVertex::per_vertex())
            .vertex_shader(
                edge_vs::load(device.clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
                (),
            )
            .input_assembly_state(
                InputAssemblyState::new()
                    .topology(PrimitiveTopology::LineStrip)
                    .primitive_restart_enable(),
            )
            .rasterization_state(RasterizationState {
                front_face: StateMode::Fixed(FrontFace::CounterClockwise),
                cull_mode: StateMode::Fixed(CullMode::None),
                line_width: StateMode::Fixed(2.0),
                line_rasterization_mode: LineRasterizationMode::Rectangular,
                ..RasterizationState::default()
            })
            .multisample_state(MultisampleState {
                rasterization_samples: msaa_samples,
                sample_shading: Some(0.5),
                ..Default::default()
            })
            .depth_stencil_state(DepthStencilState {
                depth: Some(DepthState {
                    enable_dynamic: false,
                    write_enable: StateMode::Fixed(true),
                    compare_op: StateMode::Fixed(CompareOp::Less),
                }),
                ..DepthStencilState::default()
            })
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(
                edge_fs::load(device.clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
                (),
            )
            .render_pass(Subpass::from(render_pass.clone(), 1).unwrap())
            .build(device.clone())
            .unwrap()
    }
}
impl GraphicsStage<Inputs<'_>> for EdgeStage {
    fn pipeline(&self) -> Arc<GraphicsPipeline> {
        self.pipeline.clone()
    }

    fn layout(&self) -> Arc<PipelineLayout> {
        self.pipeline.layout().clone()
    }

    fn add_commands<'a>(
        &self,
        inputs: Inputs<'a>,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        _descriptor_set_allocator: &StandardDescriptorSetAllocator,
    ) {
        builder
            .next_subpass(SubpassContents::Inline)
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone());

        if inputs.show {
            if let (Some(ref edge_vertex_buffer), Some(ref edge_index_buffer)) =
                (&inputs.vertices, &inputs.indices)
            {
                builder
                    .bind_vertex_buffers(0, edge_vertex_buffer.clone())
                    .bind_index_buffer(edge_index_buffer.clone())
                    .draw_indexed(edge_index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }
    }
}

mod edge_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/edge.vert",
    }
}

mod edge_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/edge.frag",
    }
}
