use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use splines::Surface;
use vulkano::{
    device::{Device, DeviceOwned},
    image::{ImageLayout, SampleCount},
    pipeline::{
        graphics::{
            color_blend::ColorBlendState,
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, PolygonMode, RasterizationState},
            render_pass::PipelineRenderPassType,
            vertex_input::{Vertex, VertexBufferDescription},
            viewport::ViewportState,
            GraphicsPipelineBuilder,
        },
        GraphicsPipeline, StateMode,
    },
    render_pass::{
        AttachmentDescription, AttachmentReference, RenderPass, RenderPassCreateInfo,
        SubpassDependency, SubpassDescription,
    },
    shader::ShaderModule,
    sync::{AccessFlags, DependencyFlags, PipelineStages},
};

use super::{
    attachment::{Attachment, AttachmentKind, AttachmentWithId},
    subpass::{Shader, SubpassInstructions},
    surface_vs, SurfaceMode,
};
use crate::{model::BufferedSurfaceVertex, renderer::subpass::Subpass};

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

struct SubpassWithId<TBuildParams> {
    id: u32,
    subpass: Subpass<TBuildParams>,
}

pub struct Pass<TSubpassParams> {
    attachments: Vec<AttachmentWithId>,
    subpasses: Vec<Box<SubpassWithId<TSubpassParams>>>,
    phantom: PhantomData<TSubpassParams>,
}
impl<TSubpassParams> Pass<TSubpassParams> {
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

    pub fn add_subpass(mut self, subpass: Subpass<TSubpassParams>) -> Self {
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
    ) -> PassRuntime<TSubpassParams> {
        PassRuntime::new(self, samples, device)
    }
}

pub struct PassRuntime<TSubpassParams> {
    render_pass: Arc<RenderPass>,
    samples: SampleCount,
    subpasses: Vec<Box<SubpassWithId<TSubpassParams>>>,
    phantom: PhantomData<TSubpassParams>,
}
impl<TSubpassParams> PassRuntime<TSubpassParams> {
    pub fn new(pass: Pass<TSubpassParams>, samples: SampleCount, device: Arc<Device>) -> Self {
        let mut attachment_descriptions: Vec<AttachmentDescription> = Vec::new();
        let mut attachment_ids_to_indices: HashMap<u32, usize> = HashMap::new();
        for (index, AttachmentWithId { id, attachment }) in pass.attachments.into_iter().enumerate()
        {
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
            phantom: PhantomData,
        }
    }

    fn build_pipeline(&self, params: &TSubpassParams) {
        for subpass in self.subpasses.iter() {
            self.build_subpass(subpass, params);
        }
    }

    fn build_subpass(
        &self,
        subpass: &SubpassWithId<TSubpassParams>,
        params: &TSubpassParams,
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
