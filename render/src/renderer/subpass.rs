use std::collections::HashMap;

use crate::renderer::attachment::Attachment;

use super::attachment::AttachmentWithId;

pub struct Subpass {
    input_attachments: Vec<u32>,
    color_attachments: Vec<u32>,
    depth_attachments: Vec<u32>,
    resolve_attachments: Vec<u32>,
}
impl Subpass {
    pub fn new() -> Self {
        Self {
            input_attachments: Vec::new(),
            color_attachments: Vec::new(),
            depth_attachments: Vec::new(),
            resolve_attachments: Vec::new(),
        }
    }

    pub fn input(mut self, attachment: &AttachmentWithId) -> Self {
        self.input_attachments.push(attachment.id());
        self
    }

    pub fn color(mut self, attachment: &AttachmentWithId) -> Self {
        self.color_attachments.push(attachment.id());
        self
    }

    pub fn depth(mut self, attachment: &AttachmentWithId) -> Self {
        self.depth_attachments.push(attachment.id());
        self
    }

    pub fn resolve(mut self, attachment: &AttachmentWithId) -> Self {
        self.resolve_attachments.push(attachment.id());
        self
    }
}
