use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use splines::Surface;
use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{Device, DeviceOwned, Queue},
    format::ClearValue,
    image::{ImageLayout, SampleCount},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, PolygonMode, RasterizationState},
            render_pass::PipelineRenderPassType,
            vertex_input::{Vertex, VertexBufferDescription},
            viewport::{Scissor, Viewport, ViewportState},
            GraphicsPipelineBuilder,
        },
        GraphicsPipeline, StateMode,
    },
    render_pass::{
        AttachmentDescription, AttachmentReference, RenderPass, RenderPassCreateInfo,
        SubpassDependency, SubpassDescription,
    },
    shader::ShaderModule,
    sync::{AccessFlags, DependencyFlags, GpuFuture, PipelineStages},
};

use super::{
    attachment::{Attachment, AttachmentKind, AttachmentWithId},
    subpass::{Shader, SubpassBuildInstructions},
    surface_vs, RendererImages, SurfaceMode,
};
use crate::{model::BufferedSurfaceVertex, renderer::subpass::Subpass, PixelViewport};

struct IdGenerator {
    last_id: u32,
}
impl IdGenerator {
    pub fn new() -> Self {
        Self { last_id: 0 }
    }

    pub fn next(&mut self) -> u32 {
        self.last_id += 1;
        self.last_id
    }
}

struct SubpassWithId<TBuildParams, TRunParams> {
    id: u32,
    subpass: Subpass<TBuildParams, TRunParams>,
}

pub struct Pass<TBuildParams, TRunParams> {
    attachments: Vec<AttachmentWithId>,
    subpasses: Vec<Box<SubpassWithId<TBuildParams, TRunParams>>>,
    phantom: PhantomData<TBuildParams>,
}
impl<TBuildParams, TRunParams> Pass<TBuildParams, TRunParams> {
    pub fn new() -> Self {
        Self {
            attachments: Vec::new(),
            subpasses: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn add_attachment(&mut self, attachment: Attachment) -> AttachmentWithId {
        let with_id = AttachmentWithId::new(self.attachments.len() as u32, attachment);
        //self.attachments.insert(with_id.id(), with_id.clone());
        self.attachments.push(with_id.clone());
        with_id
    }

    pub fn add_subpass(mut self, subpass: Subpass<TBuildParams, TRunParams>) -> Self {
        self.subpasses.push(Box::new(SubpassWithId {
            id: self.subpasses.len() as u32,
            subpass,
        }));

        self
    }

    pub fn build_runtime(
        self,
        samples: SampleCount,
        device: Arc<Device>,
    ) -> PassRuntime<TBuildParams, TRunParams> {
        PassRuntime::new(self, samples, device)
    }
}

pub struct PassRuntime<TBuildParams, TRunParams> {
    render_pass: Arc<RenderPass>,
    samples: SampleCount,
    subpasses: Vec<Box<SubpassWithId<TBuildParams, TRunParams>>>,
    subpass_pipelines: Vec<Arc<GraphicsPipeline>>,
    clear_values: Vec<Option<ClearValue>>,
    phantom: PhantomData<TBuildParams>,
}
impl<TBuildParams, TRunParams> PassRuntime<TBuildParams, TRunParams> {
    pub fn new(
        pass: Pass<TBuildParams, TRunParams>,
        samples: SampleCount,
        device: Arc<Device>,
    ) -> Self {
        let mut clear_values: Vec<Option<ClearValue>> = Vec::new();
        let mut attachment_descriptions: Vec<AttachmentDescription> = Vec::new();
        let mut attachment_ids_to_indices: HashMap<u32, usize> = HashMap::new();
        for (index, AttachmentWithId { id, attachment }) in pass.attachments.into_iter().enumerate()
        {
            clear_values.push(attachment.clear_value.clone());
            attachment_ids_to_indices.insert(id, index);
            attachment_descriptions.push(attachment.to_description(samples));
        }

        let mut subpass_descriptions: Vec<SubpassDescription> = Vec::new();
        for subpass in pass.subpasses.iter() {
            subpass_descriptions.push(SubpassDescription {
                input_attachments: subpass
                    .subpass
                    .input_attachments
                    .iter()
                    .map(|id| {
                        let index = attachment_ids_to_indices.get(&id).unwrap();
                        Some(AttachmentReference {
                            attachment: *index as u32,
                            layout: ImageLayout::ShaderReadOnlyOptimal,
                            ..Default::default()
                        })
                    })
                    .collect(),
                color_attachments: subpass
                    .subpass
                    .color_attachments
                    .iter()
                    .map(|id| {
                        let index = attachment_ids_to_indices.get(&id).unwrap();
                        Some(AttachmentReference {
                            attachment: *index as u32,
                            layout: ImageLayout::ColorAttachmentOptimal,
                            ..Default::default()
                        })
                    })
                    .collect(),
                resolve_attachments: subpass
                    .subpass
                    .resolve_attachments
                    .iter()
                    .map(|id| {
                        let index = attachment_ids_to_indices.get(&id).unwrap();
                        Some(AttachmentReference {
                            attachment: *index as u32,
                            layout: ImageLayout::TransferDstOptimal,
                            ..Default::default()
                        })
                    })
                    .collect(),
                depth_stencil_attachment: subpass.subpass.depth_attachment.map(|id| {
                    let index = attachment_ids_to_indices.get(&id).unwrap();
                    AttachmentReference {
                        attachment: *index as u32,
                        layout: ImageLayout::DepthStencilAttachmentOptimal,
                        ..Default::default()
                    }
                }),
                ..Default::default()
            })
        }

        let dependencies: Vec<_> = (0..subpass_descriptions.len().saturating_sub(1) as u32)
            .map(|id| {
                // TODO: correct values
                let src_stages = PipelineStages::ALL_GRAPHICS;
                let dst_stages = PipelineStages::ALL_GRAPHICS;
                let src_access = AccessFlags::MEMORY_READ | AccessFlags::MEMORY_WRITE;
                let dst_access = AccessFlags::MEMORY_READ | AccessFlags::MEMORY_WRITE;

                SubpassDependency {
                    src_subpass: id.into(),
                    dst_subpass: (id + 1).into(),
                    src_stages,
                    dst_stages,
                    src_access,
                    dst_access,
                    // TODO: correct values
                    dependency_flags: DependencyFlags::BY_REGION,
                    ..Default::default()
                }
            })
            .collect();

        Self {
            render_pass: RenderPass::new(
                device,
                RenderPassCreateInfo {
                    attachments: attachment_descriptions,
                    subpasses: subpass_descriptions,
                    dependencies,
                    ..Default::default()
                },
            )
            .unwrap(),
            samples,
            subpasses: pass.subpasses,
            subpass_pipelines: Vec::new(),
            clear_values,
            phantom: PhantomData,
        }
    }

    pub fn render(
        &self,
        params: &TRunParams,
        pixel_viewport: &PixelViewport,
        scissor: Scissor,
        images: RendererImages,
        memory_allocator: &StandardMemoryAllocator,
        command_buffer_allocator: &StandardCommandBufferAllocator,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
        queue: Arc<Queue>,
    ) {
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            command_buffer_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        )
        .unwrap();

        command_buffer_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: self.clear_values.clone(),
                    render_area_offset: scissor.origin,
                    render_area_extent: scissor.dimensions,
                    ..RenderPassBeginInfo::framebuffer(images.framebuffer.clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .set_viewport(
                0,
                [Viewport {
                    origin: [scissor.origin[0] as f32, scissor.origin[1] as f32],
                    dimensions: [scissor.dimensions[0] as f32, scissor.dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }],
            );

        for (index, subpass) in self.subpasses.iter().enumerate() {
            subpass.subpass.instructions.add_commands(
                params,
                self.subpass_pipelines[index].clone(),
                &mut command_buffer_builder,
                descriptor_set_allocator,
            );
        }

        command_buffer_builder.end_render_pass().unwrap();

        let command_buffer = command_buffer_builder.build().unwrap();

        let finished = command_buffer.execute(queue).unwrap();
        finished
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
    }

    pub fn build_pipeline(&self, params: &TBuildParams) {
        for subpass in self.subpasses.iter() {
            self.build_subpass(subpass, params);
        }
    }

    fn build_subpass(
        &self,
        subpass: &SubpassWithId<TBuildParams, TRunParams>,
        params: &TBuildParams,
    ) -> Arc<GraphicsPipeline> {
        let pipeline_builder = GraphicsPipeline::start().vertex_input_state(
            subpass
                .subpass
                .instructions
                .vertex_buffer_description(self.render_pass.device().clone(), params),
        );

        // Add vertex shader
        let vertex_shader = subpass
            .subpass
            .instructions
            .vertex_shader(self.render_pass.device().clone(), params);

        let pipeline_builder = match &vertex_shader {
            Some(shader) => pipeline_builder
                .vertex_shader(shader.module.entry_point(&shader.entry_point).unwrap(), ()),
            None => todo!(),
        };

        // Add fragment shader
        let fragment_shader = subpass
            .subpass
            .instructions
            .fragment_shader(self.render_pass.device().clone(), params);

        let pipeline_builder = match &fragment_shader {
            Some(shader) => pipeline_builder
                .fragment_shader(shader.module.entry_point(&shader.entry_point).unwrap(), ()),
            None => todo!(),
        };

        // Subpass-specific stuff
        let pipeline_builder = pipeline_builder.input_assembly_state(
            subpass
                .subpass
                .instructions
                .input_assembly_state(self.render_pass.device().clone(), params),
        );
        let pipeline_builder = pipeline_builder.rasterization_state(
            subpass
                .subpass
                .instructions
                .rasterization_state(self.render_pass.device().clone(), params),
        );
        let pipeline_builder = pipeline_builder.depth_stencil_state(
            subpass
                .subpass
                .instructions
                .depth_stencil_state(self.render_pass.device().clone(), params),
        );
        let pipeline_builder = pipeline_builder.color_blend_state(
            subpass
                .subpass
                .instructions
                .color_blend_state(self.render_pass.device().clone(), params),
        );

        // Set common settings and build the pipeline
        pipeline_builder
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .multisample_state(MultisampleState {
                rasterization_samples: self.samples,
                sample_shading: Some(0.5),
                ..Default::default()
            })
            .render_pass(
                vulkano::render_pass::Subpass::from(self.render_pass.clone(), subpass.id).unwrap(),
            )
            .build(self.render_pass.device().clone())
            .unwrap()
    }
}

pub(crate) struct PipelineBuilder {
    input_assembly_state: InputAssemblyState,
    rasterization_state: RasterizationState,
    depth_stencil_state: DepthStencilState,
    color_blend_state: ColorBlendState,
}
impl Default for PipelineBuilder {
    fn default() -> Self {
        Self {
            input_assembly_state: Default::default(),
            rasterization_state: Default::default(),
            depth_stencil_state: Default::default(),
            color_blend_state: Default::default(),
        }
    }
}
impl PipelineBuilder {
    pub fn input_assembly_state(self, state: InputAssemblyState) -> Self {
        Self {
            input_assembly_state: state,
            ..self
        }
    }

    pub fn rasterization_state(self, state: RasterizationState) -> Self {
        Self {
            rasterization_state: state,
            ..self
        }
    }

    pub fn depth_stencil_state(self, state: DepthStencilState) -> Self {
        Self {
            depth_stencil_state: state,
            ..self
        }
    }

    pub fn color_blend_state(self, state: ColorBlendState) -> Self {
        Self {
            color_blend_state: state,
            ..self
        }
    }
}
