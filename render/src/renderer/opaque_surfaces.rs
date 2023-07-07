use super::{
    surface_vs::{self, PushConstants},
    GraphicsStage, SurfaceMode,
};
use crate::{
    lights::LightBuffers,
    model::{BufferedSurfaceVertex, GeometryBuffers, Std140OpaqueMaterial},
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
            vertex_input::Vertex,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint, StateMode,
    },
    render_pass::{RenderPass, Subpass},
};

pub struct Inputs<'a> {
    push_constants: PushConstants,
    pipeline: Arc<GraphicsPipeline>,
    vertices: &'a Option<Subbuffer<[BufferedSurfaceVertex]>>,
    indices: &'a Option<Subbuffer<[u32]>>,
    materials: &'a Option<Subbuffer<[Std140OpaqueMaterial]>>,
    light_buffers: &'a LightBuffers,
    show: bool,
}

pub struct OpaqueSurfaceStage {
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
    mode: SurfaceMode,
    samples: SampleCount,
    pipeline: Arc<GraphicsPipeline>,
}
impl OpaqueSurfaceStage {
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        mode: SurfaceMode,
        samples: SampleCount,
    ) -> Self {
        let pipeline =
            Self::build_pipeline(device.clone(), render_pass.clone(), mode.clone(), samples);

        Self {
            device,
            render_pass,
            mode,
            samples,
            pipeline,
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

    fn add_commands(
        &self,
        inputs: Inputs<'_>,
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

pub fn build_pipeline(
    device: Arc<Device>,
    mode: SurfaceMode,
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

pub fn add_commands(
    push_constants: PushConstants,
    pipeline: Arc<GraphicsPipeline>,
    vertices: &Option<Subbuffer<[BufferedSurfaceVertex]>>,
    indices: &Option<Subbuffer<[u32]>>,
    materials: &Option<Subbuffer<[Std140OpaqueMaterial]>>,
    light_buffers: &LightBuffers,
    builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    descriptor_set_allocator: &StandardDescriptorSetAllocator,
    show: bool,
) {
    builder
        .bind_pipeline_graphics(pipeline.clone())
        .push_constants(pipeline.layout().clone(), 0, push_constants);

    if show {
        if let (
            Some(ref surface_vertex_buffer),
            Some(ref surface_index_buffer),
            Some(ref material_buffer),
        ) = (&vertices, &indices, &materials)
        {
            let (ambient_light_buffer, directional_light_buffer, point_light_buffer) = (
                &light_buffers.ambient,
                &light_buffers.directional,
                &light_buffers.point,
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

mod opaque_surface_fs {
    vulkano_shaders::shader! {
        include: ["src/shaders/includes"],
        ty: "fragment",
        path: "src/shaders/opaque_surface.frag",
    }
}
