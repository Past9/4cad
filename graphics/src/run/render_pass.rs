use std::sync::Arc;

use vulkano::render_pass::RenderPassCreateInfo;

use crate::run;
use crate::spec;

use super::framebuffer::Framebuffer;
use super::shaders::ShaderCache;

pub(super) struct RenderPass {
    attachments: Vec<run::Attachment>,
    framebuffers: Vec<run::Framebuffer>,
    render_pass: Arc<vulkano::render_pass::RenderPass>,
}
impl RenderPass {
    pub fn build(
        spec: &spec::RenderPass,
        device: Arc<vulkano::device::Device>,
        swapchain: Arc<vulkano::swapchain::Swapchain>,
        swapchain_images: Vec<Arc<vulkano::image::SwapchainImage>>,
        memory_allocator: &vulkano::memory::allocator::StandardMemoryAllocator,
        shader_cache: &mut ShaderCache,
    ) -> Self {
        let mut attachments = spec
            .attachments
            .iter()
            .enumerate()
            .map(|(id, attachment)| {
                let is_input = spec.subpasses.iter().any(|subpass| {
                    subpass
                        .input_attachments
                        .iter()
                        .any(|input_attachment_id| input_attachment_id.0 == id)
                });

                run::Attachment::build(
                    id as u32,
                    attachment,
                    spec.msaa_samples.into(),
                    swapchain.clone(),
                    is_input,
                )
            })
            .collect::<Vec<_>>();

        if spec.msaa_samples.is_multisampled() {
            attachments.push(run::Attachment::build_resolve(
                attachments.len() as u32,
                swapchain,
            ));
        }

        let mut subpasses = spec
            .subpasses
            .iter()
            .enumerate()
            .map(|(index, subpass)| {
                run::Subpass::build(
                    index,
                    subpass,
                    &attachments,
                    spec.msaa_samples.is_multisampled(),
                    device.clone(),
                    shader_cache,
                )
            })
            .collect::<Vec<_>>();

        let render_pass = vulkano::render_pass::RenderPass::new(
            device.clone(),
            RenderPassCreateInfo {
                attachments: attachments
                    .iter()
                    .map(|attachment| attachment.description.clone())
                    .collect(),
                subpasses: subpasses
                    .iter()
                    .map(|subpass| subpass.description.clone())
                    .collect(),
                dependencies: vec![],
                correlated_view_masks: vec![],
                ..Default::default()
            },
        )
        .unwrap();

        subpasses.iter_mut().for_each(|subpass| {
            subpass.build_pipelines(device.clone(), shader_cache, render_pass.clone())
        });

        let framebuffers = swapchain_images
            .iter()
            .map(|image| {
                Framebuffer::build(
                    render_pass.clone(),
                    image.clone(),
                    &attachments,
                    spec.msaa_samples.into(),
                    memory_allocator,
                )
            })
            .collect::<Vec<_>>();

        Self {
            attachments,
            framebuffers,
            render_pass,
        }
    }
}

impl From<spec::MsaaSamples> for vulkano::image::SampleCount {
    fn from(value: spec::MsaaSamples) -> Self {
        match value {
            crate::MsaaSamples::Samples1 => vulkano::image::SampleCount::Sample1,
            crate::MsaaSamples::Samples2 => vulkano::image::SampleCount::Sample2,
            crate::MsaaSamples::Samples4 => vulkano::image::SampleCount::Sample4,
            crate::MsaaSamples::Samples8 => vulkano::image::SampleCount::Sample8,
        }
    }
}
