use super::{
    subpass::{Shader, SubpassBuildInstructions, SubpassInstructions, SubpassRunInstructions},
    surface_vs::{self, PushConstants},
    GraphicsStage, SubpassBuildParams, SubpassRunParams, SurfaceMode,
};
use crate::{
    lights::LightBuffers,
    model::{BufferedSurfaceVertex, Std140OpaqueMaterial},
};
use std::sync::Arc;
use vulkano::{
    buffer::Subbuffer,
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    image::SampleCount,
    pipeline::{
        graphics::{
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, PolygonMode, RasterizationState},
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
    pub materials: &'a Option<Subbuffer<[Std140OpaqueMaterial]>>,
    pub light_buffers: &'a LightBuffers,
    pub show: bool,
}

pub(super) struct OpaqueSurfaceStage {
    pipeline: Arc<GraphicsPipeline>,
}
impl OpaqueSurfaceStage {
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        mode: SurfaceMode,
        samples: SampleCount,
    ) -> Self {
        Self {
            pipeline: Self::build_pipeline(device, render_pass, mode, samples),
        }
    }

    fn build_pipeline(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        mode: SurfaceMode,
        samples: SampleCount,
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
                cull_mode: match mode {
                    SurfaceMode::Fill => StateMode::Fixed(CullMode::None),
                    SurfaceMode::Wireframe => StateMode::Fixed(CullMode::None),
                },
                polygon_mode: match mode {
                    SurfaceMode::Fill => PolygonMode::Fill,
                    SurfaceMode::Wireframe => PolygonMode::Line,
                },
                line_width: match mode {
                    SurfaceMode::Fill => StateMode::Fixed(1.0),
                    SurfaceMode::Wireframe => StateMode::Fixed(2.0),
                },
                ..RasterizationState::default()
            })
            .multisample_state(MultisampleState {
                rasterization_samples: samples,
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
                opaque_surface_fs::load(device.clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
                (),
            )
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap()
    }
}
impl GraphicsStage<Inputs<'_>> for OpaqueSurfaceStage {
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

                let opaque_surface_descriptor_set = PersistentDescriptorSet::new(
                    descriptor_set_allocator,
                    self.pipeline.layout().set_layouts().get(0).unwrap().clone(),
                    [
                        WriteDescriptorSet::buffer(0, point_light_buffer.clone()),
                        WriteDescriptorSet::buffer(1, ambient_light_buffer.clone()),
                        WriteDescriptorSet::buffer(2, directional_light_buffer.clone()),
                        WriteDescriptorSet::buffer(3, material_buffer.clone()),
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
                        opaque_surface_descriptor_set.clone(),
                    )
                    .draw_indexed(surface_index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }
    }
}

mod opaque_surface_fs {
    vulkano_shaders::shader! {
        include: ["src/shaders/includes"],
        ty: "fragment",
        path: "src/shaders/opaque_surface.frag",
    }
}

pub(crate) struct OpaqueSurfaceSubpass {}
impl OpaqueSurfaceSubpass {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}
impl SubpassInstructions<SubpassBuildParams, SubpassRunParams<'_>> for OpaqueSurfaceSubpass {}
impl SubpassBuildInstructions<SubpassBuildParams> for OpaqueSurfaceSubpass {
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
            module: opaque_surface_fs::load(device.clone()).unwrap(),
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
        params: &SubpassBuildParams,
    ) -> RasterizationState {
        RasterizationState {
            front_face: StateMode::Fixed(FrontFace::CounterClockwise),
            cull_mode: match params.surface_mode {
                SurfaceMode::Fill => StateMode::Fixed(CullMode::None),
                SurfaceMode::Wireframe => StateMode::Fixed(CullMode::None),
            },
            polygon_mode: match params.surface_mode {
                SurfaceMode::Fill => PolygonMode::Fill,
                SurfaceMode::Wireframe => PolygonMode::Line,
            },
            line_width: match params.surface_mode {
                SurfaceMode::Fill => StateMode::Fixed(1.0),
                SurfaceMode::Wireframe => StateMode::Fixed(2.0),
            },
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
                compare_op: StateMode::Fixed(CompareOp::Less),
            }),
            ..DepthStencilState::default()
        }
    }
}

impl<'a> SubpassRunInstructions<SubpassRunParams<'a>> for OpaqueSurfaceSubpass {
    fn add_commands(
        &self,
        inputs: &SubpassRunParams,
        pipeline: Arc<GraphicsPipeline>,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
    ) {
        builder
            .bind_pipeline_graphics(pipeline.clone())
            .push_constants(
                pipeline.layout().clone(),
                0,
                inputs.opaque_surface_push_constants,
            );

        if inputs.show_surfaces {
            if let (
                Some(ref surface_vertex_buffer),
                Some(ref surface_index_buffer),
                Some(ref material_buffer),
            ) = (
                &inputs.opaque_surface_vertices,
                &inputs.opaque_surface_indices,
                &inputs.opaque_surface_materials,
            ) {
                let (ambient_light_buffer, directional_light_buffer, point_light_buffer) = (
                    &inputs.light_buffers.ambient,
                    &inputs.light_buffers.directional,
                    &inputs.light_buffers.point,
                );

                let opaque_surface_descriptor_set = PersistentDescriptorSet::new(
                    descriptor_set_allocator,
                    pipeline.layout().set_layouts().get(0).unwrap().clone(),
                    [
                        WriteDescriptorSet::buffer(0, point_light_buffer.clone()),
                        WriteDescriptorSet::buffer(1, ambient_light_buffer.clone()),
                        WriteDescriptorSet::buffer(2, directional_light_buffer.clone()),
                        WriteDescriptorSet::buffer(3, material_buffer.clone()),
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
                        opaque_surface_descriptor_set.clone(),
                    )
                    .draw_indexed(surface_index_buffer.len() as u32, 1, 0, 0, 0)
                    .unwrap();
            }
        }
    }
}
