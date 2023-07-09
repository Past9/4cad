use super::{
    subpass::{Shader, SubpassBuildInstructions, SubpassInstructions, SubpassRunInstructions},
    surface_vs::{self, PushConstants},
    GraphicsStage, SubpassBuildParams, SubpassRunParams,
};
use crate::{
    lights::LightBuffers,
    model::{BufferedPointVertex, BufferedSurfaceVertex, Std140TranslucentMaterial},
};
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
            color_blend::{
                AttachmentBlend, BlendFactor, BlendOp, ColorBlendAttachmentState, ColorBlendState,
                ColorComponents,
            },
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::{Vertex, VertexBufferDescription},
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, StateMode,
    },
    render_pass::{RenderPass, Subpass},
};

pub(super) struct Inputs<'a> {
    pub push_constants: PushConstants,
    pub vertices: &'a Option<Subbuffer<[BufferedSurfaceVertex]>>,
    pub indices: &'a Option<Subbuffer<[u32]>>,
    pub materials: &'a Option<Subbuffer<[Std140TranslucentMaterial]>>,
    pub light_buffers: &'a LightBuffers,
    pub depth_image: Arc<ImageView<AttachmentImage>>,
    pub show: bool,
}

pub(super) struct TranslucentSurfaceStage {
    pipeline: Arc<GraphicsPipeline>,
}
impl TranslucentSurfaceStage {
    pub fn new(device: Arc<Device>, render_pass: Arc<RenderPass>, samples: SampleCount) -> Self {
        Self {
            pipeline: Self::build_pipeline(device, render_pass, samples),
        }
    }

    fn build_pipeline(
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
}
impl GraphicsStage<Inputs<'_>> for TranslucentSurfaceStage {
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
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
    ) {
        builder
            .next_subpass(SubpassContents::Inline)
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .push_constants(self.pipeline.layout().clone(), 0, inputs.push_constants);

        if inputs.show {
            if let (
                Some(ref surface_vertex_buffer),
                Some(ref surface_index_buffer),
                Some(ref material_buffer),
            ) = (&inputs.vertices, &inputs.indices, &inputs.materials)
            {
                let (ambient_light_buffer, directional_light_buffer, point_light_buffer) = (
                    &inputs.light_buffers.ambient,
                    &inputs.light_buffers.directional,
                    &inputs.light_buffers.point,
                );

                let translucent_surface_descriptor_set = PersistentDescriptorSet::new(
                    descriptor_set_allocator,
                    self.pipeline.layout().set_layouts().get(0).unwrap().clone(),
                    [
                        WriteDescriptorSet::buffer(0, point_light_buffer.clone()),
                        WriteDescriptorSet::buffer(1, ambient_light_buffer.clone()),
                        WriteDescriptorSet::buffer(2, directional_light_buffer.clone()),
                        WriteDescriptorSet::buffer(3, material_buffer.clone()),
                        WriteDescriptorSet::image_view(4, inputs.depth_image.clone()),
                    ],
                )
                .unwrap();

                builder
                    .bind_vertex_buffers(0, surface_vertex_buffer.clone())
                    .bind_index_buffer(surface_index_buffer.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        self.pipeline.layout().clone(),
                        0,
                        translucent_surface_descriptor_set.clone(),
                    )
                    .draw_indexed(surface_index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }
    }
}

mod translucent_surface_fs {
    vulkano_shaders::shader! {
        include: ["src/shaders/includes"],
        ty: "fragment",
        path: "src/shaders/translucent_surface.frag",
    }
}

pub(crate) struct TranslucentSurfaceSubpass {}
impl TranslucentSurfaceSubpass {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}
impl SubpassInstructions<SubpassBuildParams, SubpassRunParams<'_>> for TranslucentSurfaceSubpass {}
impl SubpassBuildInstructions<SubpassBuildParams> for TranslucentSurfaceSubpass {
    fn vertex_buffer_description(
        &self,
        _device: Arc<Device>,
        _params: &SubpassBuildParams,
    ) -> VertexBufferDescription {
        BufferedSurfaceVertex::per_vertex()
    }

    fn vertex_shader(&self, device: Arc<Device>, _params: &SubpassBuildParams) -> Option<Shader> {
        Some(Shader {
            module: surface_vs::load(device.clone()).unwrap(),
            entry_point: "main".into(),
        })
    }

    fn fragment_shader(&self, device: Arc<Device>, _params: &SubpassBuildParams) -> Option<Shader> {
        Some(Shader {
            module: translucent_surface_fs::load(device.clone()).unwrap(),
            entry_point: "main".into(),
        })
    }

    fn input_assembly_state(
        &self,
        _device: Arc<Device>,
        _params: &SubpassBuildParams,
    ) -> InputAssemblyState {
        InputAssemblyState::new().topology(PrimitiveTopology::TriangleList)
    }

    fn rasterization_state(
        &self,
        _device: Arc<Device>,
        _params: &SubpassBuildParams,
    ) -> RasterizationState {
        RasterizationState {
            front_face: StateMode::Fixed(FrontFace::CounterClockwise),
            cull_mode: StateMode::Fixed(CullMode::None),
            ..RasterizationState::default()
        }
    }

    fn depth_stencil_state(
        &self,
        _device: Arc<Device>,
        _params: &SubpassBuildParams,
    ) -> DepthStencilState {
        DepthStencilState {
            depth: Some(DepthState {
                enable_dynamic: false,
                write_enable: StateMode::Fixed(true),
                compare_op: StateMode::Fixed(CompareOp::Always),
            }),
            ..DepthStencilState::default()
        }
    }

    fn color_blend_state(
        &self,
        device: Arc<Device>,
        params: &SubpassBuildParams,
    ) -> ColorBlendState {
        ColorBlendState {
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
        }
    }
}

impl<'a> SubpassRunInstructions<SubpassRunParams<'a>> for TranslucentSurfaceSubpass {
    fn add_commands(
        &self,
        inputs: &SubpassRunParams,
        pipeline: Arc<GraphicsPipeline>,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
    ) {
        builder
            .next_subpass(SubpassContents::Inline)
            .unwrap()
            .bind_pipeline_graphics(pipeline.clone())
            .push_constants(
                pipeline.layout().clone(),
                0,
                inputs.translucent_surface_push_constants,
            );

        if inputs.show_surfaces {
            if let (
                Some(ref surface_vertex_buffer),
                Some(ref surface_index_buffer),
                Some(ref material_buffer),
            ) = (
                &inputs.translucent_surface_vertices,
                &inputs.translucent_surface_indices,
                &inputs.translucent_surface_materials,
            ) {
                let (ambient_light_buffer, directional_light_buffer, point_light_buffer) = (
                    &inputs.light_buffers.ambient,
                    &inputs.light_buffers.directional,
                    &inputs.light_buffers.point,
                );

                let translucent_surface_descriptor_set = PersistentDescriptorSet::new(
                    descriptor_set_allocator,
                    pipeline.layout().set_layouts().get(0).unwrap().clone(),
                    [
                        WriteDescriptorSet::buffer(0, point_light_buffer.clone()),
                        WriteDescriptorSet::buffer(1, ambient_light_buffer.clone()),
                        WriteDescriptorSet::buffer(2, directional_light_buffer.clone()),
                        WriteDescriptorSet::buffer(3, material_buffer.clone()),
                        WriteDescriptorSet::image_view(4, inputs.depth_image.clone()),
                    ],
                )
                .unwrap();

                builder
                    .bind_vertex_buffers(0, surface_vertex_buffer.clone())
                    .bind_index_buffer(surface_index_buffer.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline.layout().clone(),
                        0,
                        translucent_surface_descriptor_set.clone(),
                    )
                    .draw_indexed(surface_index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }
    }
}
