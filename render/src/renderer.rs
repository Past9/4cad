use super::{
    model::{BufferedEdgeVertex, BufferedPointVertex, BufferedSurfaceVertex},
    scene::Scene,
};
use crate::lights::LightBuffers;
use crate::model::GeometryBuffers;
use crate::PixelViewport;
use bytemuck::{Pod, Zeroable};
use cgmath::{Point3, Vector2, Vector3};
use std::sync::Arc;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract, RenderPassBeginInfo,
        SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::Queue,
    format::Format,
    image::{
        view::ImageView, AttachmentImage, ImageDimensions, ImageLayout, ImageUsage,
        ImageViewAbstract, SampleCount, StorageImage,
    },
    memory::allocator::StandardMemoryAllocator,
    pipeline::{
        graphics::{
            color_blend::{
                AttachmentBlend, BlendFactor, BlendOp, ColorBlendAttachmentState, ColorBlendState,
                ColorComponents,
            },
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{
                CullMode, FrontFace, LineRasterizationMode, PolygonMode, RasterizationState,
            },
            vertex_input::BuffersDefinition,
            viewport::{Scissor, Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint, StateMode,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
};

const FINAL_IMAGE_FORMAT: Format = Format::B8G8R8A8_UNORM;
const TRANSLUCENT_ACCUM_FORMAT: Format = Format::R16G16B16A16_SFLOAT;
const TRANSLUCENT_TRANSMISSION_FORMAT: Format = Format::R8G8B8A8_UNORM;

pub enum SurfaceMode {
    Fill,
    Wireframe,
}

pub struct Renderer {
    scene: Scene,
    render_pass: Arc<RenderPass>,

    // Pipelines
    opaque_surface_pipeline: Arc<GraphicsPipeline>,
    edge_pipeline: Arc<GraphicsPipeline>,
    point_pipeline: Arc<GraphicsPipeline>,
    translucent_surface_pipeline: Arc<GraphicsPipeline>,
    compositing_pipeline: Arc<GraphicsPipeline>,

    images: RendererImages,
    msaa_samples: SampleCount,
    scissor: Scissor,
    framebuffers_rebuilt: bool,

    light_buffers: LightBuffers,
    geometry_buffers: GeometryBuffers,

    // Image quad buffers
    full_quad_vertex_buffer: Arc<CpuAccessibleBuffer<[ScreenSpaceVertex]>>,
    full_quad_index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
}
impl Renderer {
    pub fn new<'a>(
        scene: Scene,
        msaa_samples: SampleCount,
        memory_allocator: &StandardMemoryAllocator,
        queue: Arc<Queue>,
    ) -> Self {
        let scissor = Scissor {
            origin: [0, 0],
            dimensions: [0, 0],
        };

        let (
            render_pass,
            images,
            opaque_surface_pipeline,
            edge_pipeline,
            point_pipeline,
            translucent_surface_pipeline,
            compositing_pipeline,
        ) = Self::create_pipelines(
            SurfaceMode::Wireframe,
            msaa_samples,
            &scissor,
            memory_allocator,
            queue,
        );

        let geometry_buffers = scene.geometry_buffers(memory_allocator);
        let light_buffers = scene.light_buffers(memory_allocator);

        let full_quad_vertex_buffer = CpuAccessibleBuffer::from_iter(
            memory_allocator,
            BufferUsage {
                vertex_buffer: true,
                ..Default::default()
            },
            false,
            [
                ScreenSpaceVertex {
                    position: [-1.0, -1.0],
                },
                ScreenSpaceVertex {
                    position: [1.0, -1.0],
                },
                ScreenSpaceVertex {
                    position: [1.0, 1.0],
                },
                ScreenSpaceVertex {
                    position: [-1.0, 1.0],
                },
            ],
        )
        .unwrap();

        let full_quad_index_buffer = CpuAccessibleBuffer::from_iter(
            memory_allocator,
            BufferUsage {
                index_buffer: true,
                ..Default::default()
            },
            false,
            [0, 1, 2, 0, 2, 3],
        )
        .unwrap();

        Self {
            scene,
            render_pass,

            // Pipelines
            opaque_surface_pipeline,
            edge_pipeline,
            point_pipeline,
            translucent_surface_pipeline,
            compositing_pipeline,

            images,
            msaa_samples,
            scissor,
            framebuffers_rebuilt: true,

            geometry_buffers,
            light_buffers,

            // Image quad buffers
            full_quad_vertex_buffer,
            full_quad_index_buffer,
        }
    }

    pub fn camera_vec_to(&self, location: Point3<f32>) -> Vector3<f32> {
        self.scene.camera().vec_to(location)
    }

    pub fn viewport_size_at_dist(&self, dist: f32) -> Vector2<f32> {
        self.scene.camera().viewport_size_at_dist(dist)
    }

    pub fn framebuffers_rebuilt(&self) -> bool {
        self.framebuffers_rebuilt
    }

    fn create_pipelines<'a>(
        mode: SurfaceMode,
        msaa_samples: SampleCount,
        scissor: &Scissor,
        memory_allocator: &StandardMemoryAllocator,
        queue: Arc<Queue>,
    ) -> (
        Arc<RenderPass>,
        RendererImages,
        Arc<GraphicsPipeline>,
        Arc<GraphicsPipeline>,
        Arc<GraphicsPipeline>,
        Arc<GraphicsPipeline>,
        Arc<GraphicsPipeline>,
    ) {
        let device = queue.device();
        let render_pass = vulkano::ordered_passes_renderpass!(
            device.clone(),
            attachments: {
                opaque: {
                    load: Clear,
                    store: Store,
                    format: FINAL_IMAGE_FORMAT,
                    samples: msaa_samples,
                    initial_layout: ImageLayout::ColorAttachmentOptimal,
                    final_layout: ImageLayout::ColorAttachmentOptimal,
                },
                translucent_accum: {
                    load: Clear,
                    store: Store,
                    format: TRANSLUCENT_ACCUM_FORMAT,
                    samples: msaa_samples,
                },
                translucent_transmit: {
                    load: Clear,
                    store: Store,
                    format: TRANSLUCENT_TRANSMISSION_FORMAT,
                    samples: msaa_samples,
                },
                composite: {
                    load: Clear,
                    store: DontCare,
                    format: FINAL_IMAGE_FORMAT,
                    samples: msaa_samples,
                    initial_layout: ImageLayout::ColorAttachmentOptimal,
                    final_layout: ImageLayout::ColorAttachmentOptimal,
                },
                view: {
                    load: Clear,
                    store: DontCare,
                    format: FINAL_IMAGE_FORMAT,
                    samples: 1,
                    initial_layout: ImageLayout::ColorAttachmentOptimal,
                    final_layout: ImageLayout::ColorAttachmentOptimal,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D32_SFLOAT,
                    samples: msaa_samples,
                    initial_layout: ImageLayout::DepthStencilAttachmentOptimal,
                    final_layout: ImageLayout::DepthStencilAttachmentOptimal,
                }
            },
            passes: [
                // Opaque surfaces
                {
                    color: [opaque],
                    depth_stencil: {depth},
                    input: [],
                    resolve: []
                },
                // Edges
                {
                    color: [opaque],
                    depth_stencil: {depth},
                    input: [],
                    resolve: []
                },
                // Points
                {
                    color: [opaque],
                    depth_stencil: {depth},
                    input: [],
                    resolve: []
                },
                // Translucent surfaces
                {
                    color: [translucent_accum, translucent_transmit],
                    depth_stencil: {},
                    input: [opaque, depth]
                    resolve: [],
                },
                // Composite
                {
                    color: [composite],
                    depth_stencil: {},
                    input: [opaque, translucent_accum, translucent_transmit],
                    resolve: [view]
                }
            ]
        )
        .unwrap();

        let images = RendererImages::new(
            render_pass.clone(),
            &scissor,
            msaa_samples,
            memory_allocator,
            queue.clone(),
        );

        let opaque_surface_pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<BufferedSurfaceVertex>())
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
                    SurfaceMode::Fill => StateMode::Fixed(CullMode::Back),
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
            .unwrap();

        let edge_pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<BufferedEdgeVertex>())
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
            .unwrap();

        let point_pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<BufferedPointVertex>())
            .vertex_shader(
                point_vs::load(device.clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
                (),
            )
            .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::PointList))
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
                    compare_op: StateMode::Fixed(CompareOp::Less),
                }),
                ..DepthStencilState::default()
            })
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(
                point_fs::load(device.clone())
                    .unwrap()
                    .entry_point("main")
                    .unwrap(),
                (),
            )
            .render_pass(Subpass::from(render_pass.clone(), 2).unwrap())
            .build(device.clone())
            .unwrap();

        let translucent_surface_pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<BufferedSurfaceVertex>())
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
                cull_mode: StateMode::Fixed(CullMode::Back),
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
            .unwrap();

        let compositing_pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<ScreenSpaceVertex>())
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
            .unwrap();

        (
            render_pass,
            images,
            opaque_surface_pipeline,
            edge_pipeline,
            point_pipeline,
            translucent_surface_pipeline,
            compositing_pipeline,
        )
    }

    fn update_viewport<'a>(
        &mut self,
        pixel_viewport: &PixelViewport,
        memory_allocator: &StandardMemoryAllocator,
        queue: Arc<Queue>,
    ) {
        let new_scissor = Scissor {
            origin: [pixel_viewport.left, pixel_viewport.top],
            dimensions: [pixel_viewport.width, pixel_viewport.height],
        };

        if new_scissor != self.scissor {
            self.scissor = new_scissor;
            self.scene
                .camera_mut()
                .set_viewport_in_pixels(self.scissor.dimensions);

            self.images = RendererImages::new(
                self.render_pass.clone(),
                &self.scissor,
                self.msaa_samples,
                memory_allocator,
                queue,
            );

            self.framebuffers_rebuilt = true;
        } else {
            self.framebuffers_rebuilt = false;
        }
    }

    pub fn render<'a>(
        &mut self,
        pixel_viewport: &PixelViewport,
        memory_allocator: &StandardMemoryAllocator,
        command_buffer_allocator: &StandardCommandBufferAllocator,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
        queue: Arc<Queue>,
    ) {
        self.update_viewport(pixel_viewport, memory_allocator, queue.clone());

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        )
        .unwrap();

        let clear_values = {
            let bg_color = self.scene.background().to_floats();

            let clear_values = vec![
                Some(bg_color.into()),
                Some([0.0, 0.0, 0.0, 0.0].into()), // RT0
                Some([1.0, 1.0, 1.0, 0.0].into()), // RT1
                Some([0.0, 0.0, 0.0, 0.0].into()),
                Some(bg_color.into()),
                Some(1.0.into()),
            ];

            clear_values
        };

        command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: clear_values,
                    render_area_offset: self.scissor.origin,
                    render_area_extent: self.scissor.dimensions,
                    ..RenderPassBeginInfo::framebuffer(self.images.framebuffer.clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(
                0,
                [Viewport {
                    origin: [self.scissor.origin[0] as f32, self.scissor.origin[1] as f32],
                    dimensions: [
                        self.scissor.dimensions[0] as f32,
                        self.scissor.dimensions[1] as f32,
                    ],
                    depth_range: 0.0..1.0,
                }],
            );

        self.add_opaque_surface_commands(&mut command_buffer_builder, descriptor_set_allocator);
        self.add_edge_commands(&mut command_buffer_builder);
        self.add_point_commands(&mut command_buffer_builder);
        self.add_translucent_surface_commands(
            &mut command_buffer_builder,
            descriptor_set_allocator,
        );
        self.add_compositing_commands(&mut command_buffer_builder, descriptor_set_allocator);

        command_buffer_builder.end_render_pass().unwrap();

        let command_buffer = command_buffer_builder.build().unwrap();

        let finished = command_buffer.execute(queue).unwrap();
        finished
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
    }

    fn add_opaque_surface_commands(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
    ) {
        let push_constants = surface_vs::ty::PushConstants {
            model_matrix: self.scene.orientation().matrix().into(),
            projection_matrix: self.scene.camera().projection_matrix().into(),
        };

        builder
            .bind_pipeline_graphics(self.opaque_surface_pipeline.clone())
            .push_constants(
                self.opaque_surface_pipeline.layout().clone(),
                0,
                push_constants,
            );

        if let (
            Some(ref surface_vertex_buffer),
            Some(ref surface_index_buffer),
            Some(ref material_buffer),
        ) = (
            &self.geometry_buffers.opaque_surface_vertices,
            &self.geometry_buffers.opaque_surface_indices,
            &self.geometry_buffers.opaque_materials,
        ) {
            let (ambient_light_buffer, directional_light_buffer, point_light_buffer) = (
                &self.light_buffers.ambient,
                &self.light_buffers.directional,
                &self.light_buffers.point,
            );

            let opaque_surface_descriptor_set = PersistentDescriptorSet::new(
                descriptor_set_allocator,
                self.opaque_surface_pipeline
                    .layout()
                    .set_layouts()
                    .get(0)
                    .unwrap()
                    .clone(),
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
                    self.opaque_surface_pipeline.layout().clone(),
                    0,
                    opaque_surface_descriptor_set.clone(),
                )
                .draw_indexed(surface_index_buffer.len() as u32, 1, 0, 0, 0)
                .unwrap();
        }
    }

    fn add_edge_commands(&self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) {
        builder
            .next_subpass(SubpassContents::Inline)
            .unwrap()
            .bind_pipeline_graphics(self.edge_pipeline.clone());

        if let (Some(ref edge_vertex_buffer), Some(ref edge_index_buffer)) = (
            &self.geometry_buffers.edge_vertices,
            &self.geometry_buffers.edge_indices,
        ) {
            builder
                .bind_vertex_buffers(0, edge_vertex_buffer.clone())
                .bind_index_buffer(edge_index_buffer.clone())
                .draw_indexed(edge_index_buffer.len() as u32, 1, 0, 0, 0)
                .unwrap();
        }
    }

    fn add_point_commands(&self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) {
        builder
            .next_subpass(SubpassContents::Inline)
            .unwrap()
            .bind_pipeline_graphics(self.point_pipeline.clone());

        if let Some(ref point_vertex_buffer) = &self.geometry_buffers.point_vertices {
            builder
                .bind_vertex_buffers(0, point_vertex_buffer.clone())
                .draw(point_vertex_buffer.len() as u32, 1, 0, 0)
                .unwrap();
        }
    }

    fn add_translucent_surface_commands(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
    ) {
        let push_constants = surface_vs::ty::PushConstants {
            model_matrix: self.scene.orientation().matrix().into(),
            projection_matrix: self.scene.camera().projection_matrix().into(),
        };

        builder
            .next_subpass(SubpassContents::Inline)
            .unwrap()
            .bind_pipeline_graphics(self.translucent_surface_pipeline.clone())
            .push_constants(
                self.translucent_surface_pipeline.layout().clone(),
                0,
                push_constants,
            );

        if let (
            Some(ref surface_vertex_buffer),
            Some(ref surface_index_buffer),
            Some(ref material_buffer),
        ) = (
            &self.geometry_buffers.translucent_surface_vertices,
            &self.geometry_buffers.translucent_surface_indices,
            &self.geometry_buffers.translucent_materials,
        ) {
            let (ambient_light_buffer, directional_light_buffer, point_light_buffer) = (
                &self.light_buffers.ambient,
                &self.light_buffers.directional,
                &self.light_buffers.point,
            );

            let translucent_surface_descriptor_set = PersistentDescriptorSet::new(
                descriptor_set_allocator,
                self.translucent_surface_pipeline
                    .layout()
                    .set_layouts()
                    .get(0)
                    .unwrap()
                    .clone(),
                [
                    WriteDescriptorSet::buffer(0, point_light_buffer.clone()),
                    WriteDescriptorSet::buffer(1, ambient_light_buffer.clone()),
                    WriteDescriptorSet::buffer(2, directional_light_buffer.clone()),
                    WriteDescriptorSet::buffer(3, material_buffer.clone()),
                    WriteDescriptorSet::image_view(4, self.images.depth.clone()),
                ],
            )
            .unwrap();

            builder
                .bind_vertex_buffers(0, surface_vertex_buffer.clone())
                .bind_index_buffer(surface_index_buffer.clone())
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.translucent_surface_pipeline.layout().clone(),
                    0,
                    translucent_surface_descriptor_set.clone(),
                )
                .draw_indexed(surface_index_buffer.len() as u32, 1, 0, 0, 0)
                .unwrap();
        }
    }

    fn add_compositing_commands(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
    ) {
        builder
            .next_subpass(SubpassContents::Inline)
            .unwrap()
            .bind_pipeline_graphics(self.compositing_pipeline.clone());

        let compositing_descriptor_set = PersistentDescriptorSet::new(
            descriptor_set_allocator,
            self.compositing_pipeline
                .layout()
                .set_layouts()
                .get(0)
                .unwrap()
                .clone(),
            [
                WriteDescriptorSet::image_view(0, self.images.opaque.clone()),
                WriteDescriptorSet::image_view(1, self.images.translucent_accum.clone()),
                WriteDescriptorSet::image_view(2, self.images.translucent_transmit.clone()),
            ],
        )
        .unwrap();

        builder
            .bind_vertex_buffers(0, self.full_quad_vertex_buffer.clone())
            .bind_index_buffer(self.full_quad_index_buffer.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.compositing_pipeline.layout().clone(),
                0,
                compositing_descriptor_set.clone(),
            )
            .draw_indexed(self.full_quad_index_buffer.len() as u32, 1, 0, 0, 0)
            .unwrap();
    }

    pub fn view(&self) -> Arc<dyn ImageViewAbstract> {
        self.images.view.clone()
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }
}

struct RendererImages {
    framebuffer: Arc<Framebuffer>,
    opaque: Arc<ImageView<AttachmentImage>>,
    translucent_accum: Arc<ImageView<AttachmentImage>>,
    translucent_transmit: Arc<ImageView<AttachmentImage>>,
    _composite: Arc<ImageView<AttachmentImage>>,
    depth: Arc<ImageView<AttachmentImage>>,
    view: Arc<ImageView<StorageImage>>,
}
impl RendererImages {
    fn new(
        render_pass: Arc<RenderPass>,
        scissor: &Scissor,
        samples: SampleCount,
        memory_allocator: &StandardMemoryAllocator,
        queue: Arc<Queue>,
    ) -> Self {
        // Make sure the images are at least 1 pixel in each dimension or Vulkan will
        // throw an error. Also make the images cover the offset area so they line up
        // with the egui area that we're painting. We'll only render to the area that
        // will be shown by egui.
        //
        // TODO: Is there a way to do this that doesn't waste memory on the offset area?
        let dimensions = [
            match scissor.dimensions[0] > 0 {
                true => scissor.dimensions[0],
                false => 1,
            } + scissor.origin[0],
            match scissor.dimensions[1] > 0 {
                true => scissor.dimensions[1],
                false => 1,
            } + scissor.origin[1],
        ];

        let mut attachments: Vec<Arc<dyn ImageViewAbstract>> = Vec::new();

        let opaque = {
            let opaque = ImageView::new_default(
                AttachmentImage::multisampled_with_usage(
                    memory_allocator,
                    dimensions,
                    samples,
                    FINAL_IMAGE_FORMAT,
                    ImageUsage {
                        transient_attachment: true,
                        input_attachment: true,
                        ..ImageUsage::empty()
                    },
                )
                .unwrap(),
            )
            .unwrap();

            attachments.push(opaque.clone());

            opaque
        };

        let translucent_accum = {
            let translucent_accum = ImageView::new_default(
                AttachmentImage::multisampled_with_usage(
                    memory_allocator,
                    dimensions,
                    samples,
                    TRANSLUCENT_ACCUM_FORMAT,
                    ImageUsage {
                        transient_attachment: true,
                        input_attachment: true,
                        ..ImageUsage::empty()
                    },
                )
                .unwrap(),
            )
            .unwrap();

            attachments.push(translucent_accum.clone());

            translucent_accum
        };

        let translucent_transmit = {
            let translucent_transmit = ImageView::new_default(
                AttachmentImage::multisampled_with_usage(
                    memory_allocator,
                    dimensions,
                    samples,
                    TRANSLUCENT_TRANSMISSION_FORMAT,
                    ImageUsage {
                        transient_attachment: true,
                        input_attachment: true,
                        ..ImageUsage::empty()
                    },
                )
                .unwrap(),
            )
            .unwrap();

            attachments.push(translucent_transmit.clone());

            translucent_transmit
        };

        let composite = {
            let composite = ImageView::new_default(
                AttachmentImage::multisampled(
                    memory_allocator,
                    dimensions,
                    samples,
                    FINAL_IMAGE_FORMAT,
                )
                .unwrap(),
            )
            .unwrap();

            attachments.push(composite.clone());

            composite
        };

        let view = {
            let view = ImageView::new_default(
                StorageImage::new(
                    memory_allocator,
                    ImageDimensions::Dim2d {
                        width: dimensions[0],
                        height: dimensions[1],
                        array_layers: 1,
                    },
                    FINAL_IMAGE_FORMAT,
                    Some(queue.queue_family_index()),
                )
                .unwrap(),
            )
            .unwrap();

            attachments.push(view.clone());

            view
        };

        let depth = {
            let depth = ImageView::new_default(
                AttachmentImage::multisampled_with_usage(
                    memory_allocator,
                    dimensions,
                    samples,
                    Format::D32_SFLOAT,
                    ImageUsage {
                        depth_stencil_attachment: true,
                        transient_attachment: true,
                        input_attachment: true,
                        ..ImageUsage::empty()
                    },
                )
                .unwrap(),
            )
            .unwrap();

            attachments.push(depth.clone());

            depth
        };

        let framebuffer = Framebuffer::new(
            render_pass,
            FramebufferCreateInfo {
                attachments,
                ..Default::default()
            },
        )
        .unwrap();

        Self {
            framebuffer,
            opaque,
            translucent_accum,
            translucent_transmit,
            _composite: composite,
            depth,
            view,
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
struct ScreenSpaceVertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(ScreenSpaceVertex, position);

mod surface_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/surface.vert",
        types_meta: {
            #[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
        },
    }
}

mod opaque_surface_fs {
    vulkano_shaders::shader! {
        include: ["src/shaders/includes"],
        ty: "fragment",
        path: "src/shaders/opaque_surface.frag",
    }
}

mod translucent_surface_fs {
    vulkano_shaders::shader! {
        include: ["src/shaders/includes"],
        ty: "fragment",
        path: "src/shaders/translucent_surface.frag",
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

mod point_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/point.vert",
    }
}

mod point_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/point.frag",
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
        types_meta: {
            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}
