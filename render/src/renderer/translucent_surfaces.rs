use super::{surface_vs, translucent_surface_fs};
use crate::model::BufferedSurfaceVertex;
use std::sync::Arc;
use vulkano::{
    device::Device,
    image::SampleCount,
    pipeline::{
        graphics::{
            color_blend::{
                AttachmentBlend, BlendFactor, BlendOp, ColorBlendAttachmentState, ColorBlendState,
                ColorComponents,
            },
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
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
        .vertex_input_state(BufferedSurfaceVertex::per_vertex())
        .vertex_shader(
            surface_vs::load(device.clone())
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
        .depth_stencil_state(DepthStencilState {
            depth: Some(DepthState {
                enable_dynamic: false,
                write_enable: StateMode::Fixed(true),
                compare_op: StateMode::Fixed(CompareOp::Always),
            }),
            ..DepthStencilState::default()
        })
        .color_blend_state(ColorBlendState {
            attachments: vec![
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend {
                        color_op: BlendOp::Add,
                        color_source: BlendFactor::One,
                        color_destination: BlendFactor::One,
                        alpha_op: BlendOp::Add,
                        alpha_source: BlendFactor::One,
                        alpha_destination: BlendFactor::One,
                    }),
                    color_write_mask: ColorComponents::all(),
                    color_write_enable: StateMode::Fixed(true),
                },
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend {
                        color_op: BlendOp::Add,
                        color_source: BlendFactor::Zero,
                        color_destination: BlendFactor::OneMinusSrcColor,
                        alpha_op: BlendOp::Add,
                        alpha_source: BlendFactor::One,
                        alpha_destination: BlendFactor::One,
                    }),
                    color_write_mask: ColorComponents::all(),
                    color_write_enable: StateMode::Fixed(true),
                },
            ],
            ..ColorBlendState::default()
        })
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(
            translucent_surface_fs::load(device.clone())
                .unwrap()
                .entry_point("main")
                .unwrap(),
            (),
        )
        .render_pass(Subpass::from(render_pass.clone(), 3).unwrap())
        .build(device.clone())
        .unwrap()
}
