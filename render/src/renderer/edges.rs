use super::{edge_fs, edge_vs};
use crate::model::BufferedEdgeVertex;
use std::sync::Arc;
use vulkano::{
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
        GraphicsPipeline, StateMode,
    },
    render_pass::{RenderPass, Subpass},
};

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
