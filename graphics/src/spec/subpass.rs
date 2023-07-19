use super::AttachmentId;
use crate::Pipeline;

#[derive(Debug, Clone, Copy)]
pub struct SubpassId(pub(crate) usize);
impl From<usize> for SubpassId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct Subpass {
    pub(crate) input_attachments: Vec<AttachmentId>,
    pub(crate) color_attachments: Vec<AttachmentId>,
    pub(crate) depth_attachment: Option<AttachmentId>,
    pub(crate) pipelines: Vec<Pipeline>,
}
impl Subpass {
    pub fn new() -> Self {
        Self {
            input_attachments: vec![],
            color_attachments: vec![],
            depth_attachment: None,
            pipelines: vec![],
        }
    }

    pub fn input_attachment(&mut self, attachment: AttachmentId) -> &mut Self {
        self.input_attachments.push(attachment);
        self
    }

    pub fn input_attachments(&mut self, attachments: &[AttachmentId]) -> &mut Self {
        self.input_attachments.extend(attachments);
        self
    }

    pub fn color_attachment(&mut self, attachment: AttachmentId) -> &mut Self {
        self.color_attachments.push(attachment);
        self
    }

    pub fn color_attachments(&mut self, attachments: &[AttachmentId]) -> &mut Self {
        self.color_attachments.extend(attachments);
        self
    }

    pub fn depth_attachment(&mut self, attachment: AttachmentId) -> &mut Self {
        self.depth_attachment = Some(attachment);
        self
    }

    pub fn pipeline(&mut self, build_pipeline: fn(pipeline: &mut Pipeline)) -> &mut Self {
        let mut pipeline = Pipeline::new();
        build_pipeline(&mut pipeline);
        self.pipelines.push(pipeline);
        self
    }
}
