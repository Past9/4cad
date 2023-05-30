use crate::{
    camera::Camera,
    error::{RenderError, RenderResult},
    model::model::{BufferedModel, EdgeVertex, PointVertex, SurfaceVertex},
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents},
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, DeviceOwned, Features, Queue, QueueCreateInfo,
    },
    format::Format,
    image::{
        view::ImageView, AttachmentImage, ImageAccess, ImageUsage, SampleCount, SwapchainImage,
    },
    instance::{Instance, InstanceCreateInfo},
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            rasterization::{CullMode, FrontFace, LineRasterizationMode, RasterizationState},
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, StateMode,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        acquire_next_image, AcquireError, PresentMode, Surface, Swapchain, SwapchainCreateInfo,
        SwapchainCreationError,
    },
    sync::{self, FlushError, GpuFuture},
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

const DEFAULT_VSYNC: bool = true;

mod surface_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/surface.vert"
    }
}

mod surface_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/surface.frag"
    }
}

mod edge_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/edge.vert"
    }
}

mod edge_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/edge.frag"
    }
}

mod point_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/point.vert"
    }
}

mod point_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/point.frag"
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MsaaSamples {
    Samples1,
    Samples2,
    Samples4,
    Samples8,
}
impl MsaaSamples {
    fn as_u32(&self) -> u32 {
        match self {
            MsaaSamples::Samples1 => 1,
            MsaaSamples::Samples2 => 2,
            MsaaSamples::Samples4 => 4,
            MsaaSamples::Samples8 => 8,
        }
    }

    fn as_vulkano_samples(&self) -> SampleCount {
        match self {
            MsaaSamples::Samples1 => SampleCount::Sample1,
            MsaaSamples::Samples2 => SampleCount::Sample2,
            MsaaSamples::Samples4 => SampleCount::Sample4,
            MsaaSamples::Samples8 => SampleCount::Sample8,
        }
    }
}

pub struct Renderer {
    background_color: [f32; 3],
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    surface: Arc<Surface<Window>>,
    framebuffers: Vec<Arc<Framebuffer>>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
    surface_pipeline: Arc<GraphicsPipeline>,
    edge_pipeline: Arc<GraphicsPipeline>,
    point_pipeline: Arc<GraphicsPipeline>,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    vsync: bool,
    last_render: Option<Instant>,
    fps: f32,
    msaa_samples: MsaaSamples,
    draw_surfaces: bool,
    draw_edges: bool,
    draw_points: bool,
}
impl Renderer {
    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn new(
        event_loop: &EventLoop<()>,
        msaa_samples: MsaaSamples,
        background_color: [f32; 3],
        draw_surfaces: bool,
        draw_edges: bool,
        draw_points: bool,
    ) -> RenderResult<Self> {
        let required_extensions = vulkano_win::required_extensions();

        let instance = Instance::new(InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        })?;

        let surface = WindowBuilder::new().build_vk_surface(event_loop, instance.clone())?;

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ext_line_rasterization: true,
            ..DeviceExtensions::none()
        };

        let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
            .filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
            .filter_map(|p| {
                p.queue_families()
                    .find(|&q| {
                        q.supports_graphics() && q.supports_surface(&surface).unwrap_or(false)
                    })
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
            })
            .ok_or(RenderError::NoDevice(
                "Could not create physical device and/or queue family".into(),
            ))?;

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: physical_device
                    .required_extensions()
                    .union(&device_extensions),
                queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
                enabled_features: Features {
                    wide_lines: true,
                    rectangular_lines: true,
                    smooth_lines: true,
                    bresenham_lines: true,
                    large_points: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;

        let queue = queues.next().ok_or(RenderError::NoQueues)?;

        let (swapchain, images) = {
            let surface_capabilities =
                physical_device.surface_capabilities(&surface, Default::default())?;

            let image_format =
                Some(physical_device.surface_formats(&surface, Default::default())?[0].0);

            Swapchain::new(
                device.clone(),
                surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count,
                    image_format,
                    image_extent: surface.window().inner_size().into(),
                    image_usage: ImageUsage::color_attachment(),
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .iter()
                        .next()
                        .ok_or(RenderError::NoCompositeAlpha)?,
                    present_mode: match DEFAULT_VSYNC {
                        true => PresentMode::Fifo,
                        false => PresentMode::Mailbox,
                    },
                    ..Default::default()
                },
            )?
        };

        let render_pass = vulkano::ordered_passes_renderpass!(
            device.clone(),
            attachments: {
                msaa: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: msaa_samples.as_u32(),
                },
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.image_format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D32_SFLOAT,
                    samples: msaa_samples.as_u32(),
                }
            },
            passes: [
                {
                    color: [msaa],
                    depth_stencil: {depth},
                    input: [],
                    resolve: [color]
                },
                {
                    color: [msaa],
                    depth_stencil: {depth},
                    input: [],
                    resolve: [color]
                },
                {
                    color: [msaa],
                    depth_stencil: {depth},
                    input: [],
                    resolve: [color]
                }
            ]
        )?;

        let surface_vs = surface_vs::load(device.clone())?;
        let surface_fs = surface_fs::load(device.clone())?;

        let surface_pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<SurfaceVertex>())
            .vertex_shader(
                surface_vs
                    .entry_point("main")
                    .ok_or(RenderError::NoShaderEntryPoint)?,
                (),
            )
            .input_assembly_state(
                InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
            )
            .rasterization_state(RasterizationState {
                front_face: StateMode::Fixed(FrontFace::Clockwise),
                cull_mode: StateMode::Fixed(CullMode::None),
                ..RasterizationState::default()
            })
            .depth_stencil_state(DepthStencilState {
                depth: Some(DepthState {
                    enable_dynamic: false,
                    write_enable: StateMode::Fixed(true),
                    compare_op: StateMode::Fixed(CompareOp::Greater),
                }),
                ..DepthStencilState::default()
            })
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(
                surface_fs
                    .entry_point("main")
                    .ok_or(RenderError::NoShaderEntryPoint)?,
                (),
            )
            .render_pass(Subpass::from(render_pass.clone(), 0).ok_or(RenderError::NoRenderSubpass)?)
            .build(device.clone())?;

        let edge_vs = edge_vs::load(device.clone())?;
        let edge_fs = edge_fs::load(device.clone())?;

        let edge_pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<EdgeVertex>())
            .vertex_shader(
                edge_vs
                    .entry_point("main")
                    .ok_or(RenderError::NoShaderEntryPoint)?,
                (),
            )
            .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::LineList))
            .rasterization_state(RasterizationState {
                front_face: StateMode::Fixed(FrontFace::Clockwise),
                cull_mode: StateMode::Fixed(CullMode::None),
                line_width: StateMode::Fixed(1.0),
                line_rasterization_mode: LineRasterizationMode::Rectangular,
                ..RasterizationState::default()
            })
            .depth_stencil_state(DepthStencilState {
                depth: Some(DepthState {
                    enable_dynamic: false,
                    write_enable: StateMode::Fixed(true),
                    compare_op: StateMode::Fixed(CompareOp::Greater),
                }),
                ..DepthStencilState::default()
            })
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(
                edge_fs
                    .entry_point("main")
                    .ok_or(RenderError::NoShaderEntryPoint)?,
                (),
            )
            .render_pass(Subpass::from(render_pass.clone(), 1).ok_or(RenderError::NoRenderSubpass)?)
            .build(device.clone())?;

        let point_vs = point_vs::load(device.clone())?;
        let point_fs = point_fs::load(device.clone())?;

        let blend = ColorBlendState::default();
        let blend = blend.blend_alpha();

        let point_pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<PointVertex>())
            .vertex_shader(
                point_vs
                    .entry_point("main")
                    .ok_or(RenderError::NoShaderEntryPoint)?,
                (),
            )
            .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::PointList))
            .rasterization_state(RasterizationState {
                front_face: StateMode::Fixed(FrontFace::Clockwise),
                cull_mode: StateMode::Fixed(CullMode::None),
                ..RasterizationState::default()
            })
            .depth_stencil_state(DepthStencilState {
                depth: Some(DepthState {
                    enable_dynamic: false,
                    write_enable: StateMode::Fixed(true),
                    compare_op: StateMode::Fixed(CompareOp::Greater),
                }),
                ..DepthStencilState::default()
            })
            .color_blend_state(blend)
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(
                point_fs
                    .entry_point("main")
                    .ok_or(RenderError::NoShaderEntryPoint)?,
                (),
            )
            .render_pass(Subpass::from(render_pass.clone(), 2).ok_or(RenderError::NoRenderSubpass)?)
            .build(device.clone())?;

        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let framebuffers = Self::build_framebuffers(
            device.clone(),
            swapchain.image_format(),
            msaa_samples,
            &images,
            render_pass.clone(),
            &mut viewport,
        )?;

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        Ok(Self {
            background_color,
            device,
            queue,
            swapchain,
            surface,
            framebuffers,
            render_pass,
            viewport,
            surface_pipeline,
            edge_pipeline,
            point_pipeline,
            recreate_swapchain: false,
            previous_frame_end,
            vsync: DEFAULT_VSYNC,
            last_render: None,
            fps: 0f32,
            msaa_samples,
            draw_surfaces,
            draw_edges,
            draw_points,
        })
    }

    pub fn dimensions(&self) -> (f32, f32) {
        (self.viewport.dimensions[0], self.viewport.dimensions[1])
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        self.vsync = vsync;
        self.recreate_swapchain();
    }

    pub fn get_vsync(&self) -> bool {
        self.vsync
    }

    pub fn recreate_swapchain(&mut self) {
        self.recreate_swapchain = true;
    }

    pub fn render(&mut self, model: &BufferedModel, camera: &Camera) -> RenderResult<()> {
        self.previous_frame_end
            .as_mut()
            .ok_or(RenderError::NoPreviousFrame)?
            .cleanup_finished();

        self.recreate_swapchain_if_needed()?;

        let (image_num, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => Ok(r),
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Ok(());
                }
                Err(e) => Err(RenderError::NextSwapchainImage(e)),
            }?;

        if suboptimal {
            self.recreate_swapchain();
        }

        let clear_values = vec![
            self.background_color.into(),
            self.background_color.into(),
            0.0.into(),
        ];

        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        let push_constants = surface_vs::ty::PushConstants {
            view: camera.get_view_matrix().cast::<f32>().unwrap().into(),
            model: model.get_transform_matrix().cast::<f32>().unwrap().into(),
            perspective: camera
                .get_perspective_matrix()
                .cast::<f32>()
                .unwrap()
                .into(),
        };

        // If we want to render no surfaces, edges, or points (no triangles), we
        // need to create a "fake" CpuAccessibleBuffer with a single vertex and
        // just not render it (set first argument of draw(...) to zero). This is
        // because Vulkan does not allow an empty CpuAccessibleBuffer. This is the
        // vertex we'll create and not draw just so there's something in the buffer.
        let noop_vertex = SurfaceVertex {
            position: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        };

        builder
            // Surfaces
            .begin_render_pass(
                self.framebuffers[image_num].clone(),
                SubpassContents::Inline,
                clear_values.clone(),
            )?
            .set_viewport(0, [self.viewport.clone()])
            .bind_pipeline_graphics(self.surface_pipeline.clone())
            .push_constants(self.surface_pipeline.layout().clone(), 0, push_constants);

        match (self.draw_surfaces, &model.surface) {
            (true, Some(surface)) => {
                builder
                    .bind_vertex_buffers(0, surface.vertices.clone())
                    //.bind_index_buffer(surface.indices.clone())
                    .draw(surface.vertices.len() as u32, 1, 0, 0)?;
            }
            _ => {
                builder
                    .bind_vertex_buffers(
                        0,
                        CpuAccessibleBuffer::from_iter(
                            self.device.clone(),
                            BufferUsage::all(),
                            false,
                            vec![noop_vertex.clone()],
                        )?,
                    )
                    .draw(0, 1, 0, 0)?;
            }
        }

        builder
            .next_subpass(SubpassContents::Inline)?
            .bind_pipeline_graphics(self.edge_pipeline.clone())
            .push_constants(self.edge_pipeline.layout().clone(), 0, push_constants);

        match (self.draw_edges, &model.edges) {
            (true, Some(edges)) => {
                builder
                    .bind_vertex_buffers(0, edges.vertices.clone())
                    //.bind_index_buffer(edges.indices.clone())
                    .draw(edges.vertices.len() as u32, 1, 0, 0)?;
            }
            _ => {
                builder
                    .bind_vertex_buffers(
                        0,
                        CpuAccessibleBuffer::from_iter(
                            self.device.clone(),
                            BufferUsage::all(),
                            false,
                            vec![noop_vertex.clone()],
                        )?,
                    )
                    .draw(0, 1, 0, 0)?;
            }
        };

        builder
            .next_subpass(SubpassContents::Inline)?
            .bind_pipeline_graphics(self.point_pipeline.clone())
            .push_constants(self.point_pipeline.layout().clone(), 0, push_constants);

        match (self.draw_points, &model.points) {
            (true, Some(points)) => {
                builder
                    .bind_vertex_buffers(0, points.vertices.clone())
                    .bind_index_buffer(points.indices.clone())
                    .draw(points.vertices.len() as u32, 1, 0, 0)?;
            }
            _ => {
                builder
                    .bind_vertex_buffers(
                        0,
                        CpuAccessibleBuffer::from_iter(
                            self.device.clone(),
                            BufferUsage::all(),
                            false,
                            vec![noop_vertex.clone()],
                        )?,
                    )
                    .draw(0, 1, 0, 0)?;
            }
        };

        builder.end_render_pass()?;

        let command_buffer = builder.build()?;

        let future = self
            .previous_frame_end
            .take()
            .ok_or(RenderError::NoPreviousFrame)?
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)?
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());

                let now = std::time::Instant::now();

                self.fps = match self.last_render {
                    Some(last_render) => {
                        let frame_duration = (now - last_render).as_micros();
                        if frame_duration > 0 {
                            (Duration::from_secs(1).as_micros() / frame_duration) as f32
                        } else {
                            0f32
                        }
                    }
                    None => 0f32,
                };

                self.last_render = Some(now);
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
        };

        Ok(())
    }

    fn build_framebuffers(
        device: Arc<Device>,
        image_format: Format,
        msaa_samples: MsaaSamples,
        images: &[Arc<SwapchainImage<Window>>],
        render_pass: Arc<RenderPass>,
        viewport: &mut Viewport,
    ) -> RenderResult<Vec<Arc<Framebuffer>>> {
        let dimensions = images[0].dimensions().width_height();
        viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

        let msaa_attachment = ImageView::new_default(AttachmentImage::transient_multisampled(
            device.clone(),
            dimensions,
            msaa_samples.as_vulkano_samples(),
            image_format,
        )?)?;

        let depth_attachment = ImageView::new_default(AttachmentImage::multisampled_with_usage(
            render_pass.device().clone(),
            dimensions,
            msaa_samples.as_vulkano_samples(),
            Format::D32_SFLOAT,
            ImageUsage {
                depth_stencil_attachment: true,
                transient_attachment: true,
                ..ImageUsage::none()
            },
        )?)?;

        let framebuffers: RenderResult<Vec<Arc<Framebuffer>>> = images
            .into_iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone())?;
                Ok(Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![msaa_attachment.clone(), view, depth_attachment.clone()],
                        ..Default::default()
                    },
                )?)
            })
            .collect();

        framebuffers
    }

    fn recreate_swapchain_if_needed(&mut self) -> RenderResult<()> {
        if self.recreate_swapchain {
            self.force_recreate_swapchain()?;
            self.recreate_swapchain = false;
        }

        Ok(())
    }

    fn force_recreate_swapchain(&mut self) -> RenderResult<()> {
        let (new_swapchain, new_images) = match self.swapchain.recreate(SwapchainCreateInfo {
            image_extent: self.surface.window().inner_size().into(),
            present_mode: match self.vsync {
                true => PresentMode::Fifo,
                false => PresentMode::Mailbox,
            },
            ..self.swapchain.create_info()
        }) {
            Ok(r) => r,
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return Ok(()),
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };

        self.swapchain = new_swapchain;
        self.framebuffers = Self::build_framebuffers(
            self.device.clone(),
            self.swapchain.image_format(),
            self.msaa_samples,
            &new_images,
            self.render_pass.clone(),
            &mut self.viewport,
        )?;

        Ok(())
    }
}
