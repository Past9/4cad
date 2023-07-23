use std::sync::Arc;

use vulkano::render_pass::RenderPassCreateInfo;

use crate::run;
use crate::spec;

use super::framebuffer::Framebuffer;

pub(super) struct Subpass {
    pub description: vulkano::render_pass::SubpassDescription,
}
impl Subpass {
    pub fn build(
        spec: &spec::Subpass,
        attachments: &[run::Attachment],
        is_multisampled: bool,
    ) -> Self {
        let input_attachments = spec
            .input_attachments
            .iter()
            .map(|id| {
                Some(vulkano::render_pass::AttachmentReference {
                    attachment: id.0 as u32,
                    layout: vulkano::image::ImageLayout::ShaderReadOnlyOptimal,
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>();

        let color_attachments = spec
            .color_attachments
            .iter()
            .map(|id| {
                Some(vulkano::render_pass::AttachmentReference {
                    attachment: id.0 as u32,
                    layout: vulkano::image::ImageLayout::ColorAttachmentOptimal,
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>();

        let depth_stencil_attachment =
            spec.depth_attachment
                .map(|id| vulkano::render_pass::AttachmentReference {
                    attachment: id.0 as u32,
                    layout: vulkano::image::ImageLayout::DepthStencilAttachmentOptimal,
                    ..Default::default()
                });

        let resolve_attachments = if spec.is_output && is_multisampled {
            vec![Some(vulkano::render_pass::AttachmentReference {
                attachment: attachments.len() as u32 - 1,
                layout: vulkano::image::ImageLayout::TransferDstOptimal,
                ..Default::default()
            })]
        } else {
            vec![]
        };

        println!("subpass spec {:#?}", spec);
        println!("subpass attachments {:#?}", attachments);

        let description = vulkano::render_pass::SubpassDescription {
            view_mask: 0,
            input_attachments,
            color_attachments,
            resolve_attachments,
            depth_stencil_attachment,
            preserve_attachments: vec![],
            ..Default::default()
        };

        Self { description }
    }
}
