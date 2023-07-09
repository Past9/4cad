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

pub struct SubpassParams {
    surface_mode: SurfaceMode,
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

fn foo() {
    const FINAL_IMAGE_FORMAT: AttachmentKind = AttachmentKind::ColorUNormBgra8;
    const TRANSLUCENT_ACCUM_FORMAT: AttachmentKind = AttachmentKind::ColorSFloatRgba16;
    const TRANSLUCENT_TRANSMISSION_FORMAT: AttachmentKind = AttachmentKind::ColorUNormRgba8;
    const DEPTH_FORMAT: AttachmentKind = AttachmentKind::DepthSFloat32;

    let mut pass = Pass::new();

    let opaque_image =
        pass.add_attachment(Attachment::new(FINAL_IMAGE_FORMAT).load_cleared().store());

    let translucent_accum_image = pass.add_attachment(
        Attachment::new(TRANSLUCENT_ACCUM_FORMAT)
            .load_cleared()
            .store(),
    );

    let translucent_transmit_image = pass.add_attachment(
        Attachment::new(TRANSLUCENT_TRANSMISSION_FORMAT)
            .load_cleared()
            .store(),
    );

    let composite_image =
        pass.add_attachment(Attachment::new(FINAL_IMAGE_FORMAT).load_cleared().store());

    let depth_stencil = pass.add_attachment(Attachment::new(DEPTH_FORMAT).load_cleared().store());

    let view = pass.add_attachment(Attachment::new(FINAL_IMAGE_FORMAT).load_cleared().store());

    pass.add_subpass(
        Subpass::new(OpaqueSurfaceSubpass::new())
            .color(&opaque_image)
            .depth(&depth_stencil),
    ) // Opaque surfaces
    .add_subpass(
        Subpass::new(OpaqueSurfaceSubpass::new())
            .color(&opaque_image)
            .depth(&depth_stencil),
    ) // Opaque edges
    .add_subpass(
        Subpass::new(OpaqueSurfaceSubpass::new())
            .color(&opaque_image)
            .depth(&depth_stencil),
    ) // Opaque points
    .add_subpass(
        Subpass::new(OpaqueSurfaceSubpass::new())
            .inputs([&opaque_image, &depth_stencil])
            .colors([&translucent_accum_image, &translucent_transmit_image]),
    ) // Translucent surfaces
    .add_subpass(
        Subpass::new(OpaqueSurfaceSubpass::new())
            .inputs([
                &opaque_image,
                &translucent_accum_image,
                &translucent_transmit_image,
            ])
            .color(&composite_image)
            .resolve(&view),
    ); // Translucent surfaces

    todo!()
}

struct OpaqueSurfaceSubpass {}
impl OpaqueSurfaceSubpass {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}
impl SubpassInstructions<SubpassParams> for OpaqueSurfaceSubpass {
    fn vertex_buffer_description(
        &self,
        device: Arc<Device>,
        params: &SubpassParams,
    ) -> VertexBufferDescription {
        BufferedSurfaceVertex::per_vertex()
    }

    fn vertex_shader(&self, device: Arc<Device>, params: &SubpassParams) -> Option<Shader> {
        Some(Shader {
            module: surface_vs::load(device.clone()).unwrap(),
            entry_point: "main".into(),
        })
    }

    fn fragment_shader(&self, device: Arc<Device>, params: &SubpassParams) -> Option<Shader> {
        Some(Shader {
            module: opaque_surface_fs::load(device.clone()).unwrap(),
            entry_point: "main".into(),
        })
    }

    fn input_assembly_state(
        &self,
        device: Arc<Device>,
        params: &SubpassParams,
    ) -> InputAssemblyState {
        InputAssemblyState::new().topology(PrimitiveTopology::TriangleList)
    }

    fn rasterization_state(
        &self,
        device: Arc<Device>,
        params: &SubpassParams,
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
        device: Arc<Device>,
        params: &SubpassParams,
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

mod opaque_surface_fs {
    vulkano_shaders::shader! {
        include: ["src/shaders/includes"],
        ty: "fragment",
        path: "src/shaders/opaque_surface.frag",
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
