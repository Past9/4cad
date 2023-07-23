use super::{Attachment, AttachmentId, Subpass};

#[derive(Debug)]
pub struct RenderPass {
    pub(crate) msaa_samples: MsaaSamples,
    pub(crate) attachments: Vec<Attachment>,
    pub(crate) subpasses: Vec<Subpass>,
}
impl RenderPass {
    pub fn new() -> Self {
        Self {
            msaa_samples: MsaaSamples::Samples1,
            attachments: vec![],
            subpasses: vec![],
        }
    }

    pub fn msaa_samples(&mut self, msaa_samples: MsaaSamples) -> &mut Self {
        self.msaa_samples = msaa_samples;
        self
    }

    pub fn attachment(&mut self, attachment: Attachment) -> AttachmentId {
        let id = self.attachments.len().into();
        self.attachments.push(attachment);
        id
    }

    pub fn subpass(&mut self, build_subpass: impl Fn(&mut Subpass)) -> &mut Self {
        let mut subpass = Subpass::new();
        build_subpass(&mut subpass);
        self.subpasses.push(subpass);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MsaaSamples {
    Samples1,
    Samples2,
    Samples4,
    Samples8,
}
impl MsaaSamples {
    pub fn is_multisampled(&self) -> bool {
        match self {
            MsaaSamples::Samples1 => false,
            _ => true,
        }
    }
}
