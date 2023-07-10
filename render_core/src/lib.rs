mod attachment;
mod pass;
mod subpass;

pub use attachment::{Attachment, AttachmentKind};
pub use pass::{Pass, PassRuntime, PipelineBuilder};
pub use subpass::{
    Shader, Subpass, SubpassBuildInstructions, SubpassInstructions, SubpassRunInstructions,
    SubpassWithId,
};

pub struct PixelViewport {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}
