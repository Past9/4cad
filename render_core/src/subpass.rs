use std::{collections::HashMap, sync::Arc};

use regex::internal::Input;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::Device,
    format::ClearValue,
    pipeline::{
        graphics::{
            color_blend::ColorBlendState, depth_stencil::DepthStencilState,
            input_assembly::InputAssemblyState, rasterization::RasterizationState,
            vertex_input::VertexBufferDescription,
        },
        GraphicsPipeline,
    },
    shader::{EntryPoint, ShaderModule},
};

use super::{
    attachment::{self, AttachmentWithId},
    pass::PipelineBuilder,
};

pub struct Shader {
    pub module: Arc<ShaderModule>,
    pub entry_point: String,
}

pub trait SubpassInstructions<TBuildParams, TRunParams>:
    SubpassBuildInstructions<TBuildParams> + SubpassRunInstructions<TRunParams>
{
}

pub trait SubpassBuildInstructions<TBuildParams> {
    fn vertex_buffer_description(
        &self,
        device: Arc<Device>,
        params: &TBuildParams,
    ) -> VertexBufferDescription;

    fn input_assembly_state(
        &self,
        device: Arc<Device>,
        params: &TBuildParams,
    ) -> InputAssemblyState {
        InputAssemblyState::default()
    }

    fn rasterization_state(
        &self,
        device: Arc<Device>,
        params: &TBuildParams,
    ) -> RasterizationState {
        RasterizationState::default()
    }

    fn depth_stencil_state(&self, device: Arc<Device>, params: &TBuildParams) -> DepthStencilState {
        DepthStencilState::default()
    }

    fn color_blend_state(&self, device: Arc<Device>, params: &TBuildParams) -> ColorBlendState {
        ColorBlendState::default()
    }
    fn vertex_shader(&self, device: Arc<Device>, params: &TBuildParams) -> Option<Shader>;
    fn fragment_shader(&self, device: Arc<Device>, params: &TBuildParams) -> Option<Shader>;
}

pub trait SubpassRunInstructions<TRunParams> {
    fn add_commands(
        &self,
        inputs: &TRunParams,
        pipeline: Arc<GraphicsPipeline>,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        descriptor_set_allocator: &StandardDescriptorSetAllocator,
    );
}

pub struct SubpassWithId<TBuildParams, TRunParams> {
    pub id: u32,
    pub subpass: Subpass<TBuildParams, TRunParams>,
}

pub struct Subpass<TBuildParams, TRunParams> {
    pub(crate) clear_value: Option<ClearValue>,
    pub(crate) input_attachments: Vec<u32>,
    pub(crate) color_attachments: Vec<u32>,
    pub(crate) depth_attachment: Option<u32>,
    pub(crate) resolve_attachments: Vec<u32>,
    pub(crate) instructions: Box<dyn SubpassInstructions<TBuildParams, TRunParams>>,
}
impl<TParams, TRunParams> Subpass<TParams, TRunParams> {
    pub fn new(instructions: Box<dyn SubpassInstructions<TParams, TRunParams>>) -> Self {
        Self {
            clear_value: None,
            input_attachments: Vec::new(),
            color_attachments: Vec::new(),
            depth_attachment: None,
            resolve_attachments: Vec::new(),
            instructions,
        }
    }

    pub fn clear_value(mut self, clear_value: Option<ClearValue>) -> Self {
        self.clear_value = clear_value;
        self
    }

    pub fn input(mut self, attachment: &AttachmentWithId) -> Self {
        self.input_attachments.push(attachment.id());
        self
    }

    pub fn inputs<const N: usize>(mut self, attachments: [&AttachmentWithId; N]) -> Self {
        for attachment in attachments.into_iter() {
            self.input_attachments.push(attachment.id());
        }
        self
    }

    pub fn color(mut self, attachment: &AttachmentWithId) -> Self {
        self.color_attachments.push(attachment.id());
        self
    }

    pub fn colors<const N: usize>(mut self, attachments: [&AttachmentWithId; N]) -> Self {
        for attachment in attachments.into_iter() {
            self.color_attachments.push(attachment.id());
        }
        self
    }

    pub fn depth(mut self, attachment: &AttachmentWithId) -> Self {
        self.depth_attachment = Some(attachment.id());
        self
    }

    pub fn resolve(mut self, attachment: &AttachmentWithId) -> Self {
        self.resolve_attachments.push(attachment.id());
        self
    }
}
