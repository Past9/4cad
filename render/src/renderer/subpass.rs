use std::{collections::HashMap, sync::Arc};

use regex::internal::Input;
use vulkano::{
    device::Device,
    pipeline::graphics::{
        color_blend::ColorBlendState, depth_stencil::DepthStencilState,
        input_assembly::InputAssemblyState, rasterization::RasterizationState,
        vertex_input::VertexBufferDescription,
    },
    shader::{EntryPoint, ShaderModule},
};

use crate::renderer::attachment::Attachment;

use super::{
    attachment::{self, AttachmentWithId},
    pass::PipelineBuilder,
};

pub struct Shader {
    pub(crate) module: Arc<ShaderModule>,
    pub(crate) entry_point: String,
}

pub trait SubpassInstructions<TBuildParams> {
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

pub struct Subpass<TParams> {
    pub(crate) input_attachments: Vec<u32>,
    pub(crate) color_attachments: Vec<u32>,
    pub(crate) depth_attachment: Option<u32>,
    pub(crate) resolve_attachments: Vec<u32>,
    pub(crate) instructions: Box<dyn SubpassInstructions<TParams>>,
}
impl<TParams> Subpass<TParams> {
    pub fn new(instructions: Box<dyn SubpassInstructions<TParams>>) -> Self {
        Self {
            input_attachments: Vec::new(),
            color_attachments: Vec::new(),
            depth_attachment: None,
            resolve_attachments: Vec::new(),
            instructions: instructions,
        }
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
