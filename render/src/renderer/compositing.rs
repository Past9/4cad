use super::{compositing_fs, compositing_vs, ScreenSpaceVertex};
use std::sync::Arc;
use vulkano::{
    device::Device,
    image::SampleCount,
    pipeline::{
        graphics::{
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
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
        .vertex_input_state(ScreenSpaceVertex::per_vertex())
        .vertex_shader(
            compositing_vs::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap(),
            (),
        )
        .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::TriangleList))
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
