use crate::run;
use std::sync::Arc;

pub(super) struct Framebuffer {
    framebuffer: Arc<vulkano::render_pass::Framebuffer>,
    images: Vec<Arc<dyn vulkano::image::ImageViewAbstract>>,
}
impl Framebuffer {
    pub fn build(
        render_pass: Arc<vulkano::render_pass::RenderPass>,
        swapchain_image: Arc<vulkano::image::SwapchainImage>,
        attachments: &Vec<run::Attachment>,
        samples: vulkano::image::SampleCount,
        memory_allocator: &vulkano::memory::allocator::StandardMemoryAllocator,
    ) -> Self {
        let images = attachments
            .iter()
            .map(|attachment| {
                attachment.make_image(samples, swapchain_image.clone(), memory_allocator)
            })
            .collect::<Vec<_>>();

        let framebuffer = vulkano::render_pass::Framebuffer::new(
            render_pass,
            vulkano::render_pass::FramebufferCreateInfo {
                attachments: images.iter().map(|image| image.clone()).collect(),
                ..Default::default()
            },
        )
        .unwrap();

        Self {
            framebuffer,
            images,
        }
    }
}
