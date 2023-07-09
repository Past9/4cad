use std::collections::HashMap;

use super::attachment::{Attachment, AttachmentKind, AttachmentWithId};
use crate::renderer::subpass::Subpass;

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

struct SubpassWithId {
    id: u32,
    subpass: Subpass,
}

pub struct Pass {
    attachments: HashMap<u32, AttachmentWithId>,
    subpasses: Vec<SubpassWithId>,
}
impl Pass {
    pub fn new() -> Self {
        Self {
            attachments: HashMap::new(),
            subpasses: Vec::new(),
        }
    }

    pub fn add_attachment(&mut self, attachment: Attachment) -> AttachmentWithId {
        let with_id = AttachmentWithId::new(self.attachments.len() as u32, attachment);
        self.attachments.insert(with_id.id(), with_id.clone());
        with_id
    }

    pub fn add_subpass(mut self, subpass: Subpass) -> Self {
        self.subpasses.push(SubpassWithId {
            id: self.subpasses.len() as u32,
            subpass,
        });

        self
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

    pass.add_subpass(Subpass::new().color(&opaque_image).depth(&depth_stencil)) // Opaque surfaces
        .add_subpass(Subpass::new().color(&opaque_image).depth(&depth_stencil)) // Opaque edges
        .add_subpass(Subpass::new().color(&opaque_image).depth(&depth_stencil)) // Opaque points
        .add_subpass(
            Subpass::new()
                .input(&opaque_image)
                .input(&depth_stencil)
                .color(&translucent_accum_image)
                .color(&translucent_transmit_image),
        ) // Translucent surfaces
        .add_subpass(
            Subpass::new()
                .input(&opaque_image)
                .input(&translucent_accum_image)
                .input(&translucent_transmit_image)
                .color(&composite_image)
                .resolve(&view),
        ); // Translucent surfaces

    todo!()
}
