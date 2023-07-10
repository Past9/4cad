use std::sync::Arc;

use vulkano::{
    format::{ClearValue, Format},
    image::{ImageLayout, SampleCount},
    render_pass::{AttachmentDescription, LoadOp, StoreOp},
};

#[derive(Clone, Debug, PartialEq)]
pub struct AttachmentWithId {
    pub(crate) id: u32,
    pub(crate) attachment: Attachment,
}
impl AttachmentWithId {
    pub fn new(id: u32, attachment: Attachment) -> Self {
        Self { id, attachment }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn attachment(&self) -> &Attachment {
        &self.attachment
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum AttachmentKind {
    ColorUNormBgra8,
    ColorSFloatRgba16,
    ColorUNormRgba8,
    DepthSFloat32,
}
impl AttachmentKind {}

struct AttachmentFormatDetails {
    format: Format,
    initial_layout: ImageLayout,
    final_layout: ImageLayout,
}
impl From<AttachmentKind> for AttachmentFormatDetails {
    fn from(value: AttachmentKind) -> Self {
        match value {
            AttachmentKind::ColorUNormBgra8 => Self {
                format: Format::B8G8R8A8_UNORM,
                initial_layout: ImageLayout::ColorAttachmentOptimal,
                final_layout: ImageLayout::ColorAttachmentOptimal,
            },
            AttachmentKind::ColorSFloatRgba16 => Self {
                format: Format::R16G16B16A16_SFLOAT,
                initial_layout: ImageLayout::ColorAttachmentOptimal,
                final_layout: ImageLayout::ColorAttachmentOptimal,
            },
            AttachmentKind::ColorUNormRgba8 => Self {
                format: Format::R8G8B8A8_UNORM,
                initial_layout: ImageLayout::ColorAttachmentOptimal,
                final_layout: ImageLayout::ColorAttachmentOptimal,
            },
            AttachmentKind::DepthSFloat32 => Self {
                format: Format::D32_SFLOAT,
                initial_layout: ImageLayout::DepthStencilAttachmentOptimal,
                final_layout: ImageLayout::DepthStencilAttachmentOptimal,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attachment {
    pub(crate) kind: AttachmentKind,
    pub(crate) load_op: LoadOp,
    pub(crate) store_op: StoreOp,
    pub(crate) stencil_load_op: LoadOp,
    pub(crate) stencil_store_op: StoreOp,
    pub(crate) initial_layout: ImageLayout,
    pub(crate) final_layout: ImageLayout,
    pub(crate) clear_value: Option<ClearValue>,
}
impl Attachment {
    pub fn new(kind: AttachmentKind) -> Self {
        Self {
            kind,
            load_op: LoadOp::DontCare,
            store_op: StoreOp::DontCare,
            stencil_load_op: LoadOp::DontCare,
            stencil_store_op: StoreOp::DontCare,
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::Undefined,
            clear_value: None,
        }
    }

    pub(crate) fn to_description(self, samples: SampleCount) -> AttachmentDescription {
        let AttachmentFormatDetails {
            format,
            initial_layout,
            final_layout,
        } = self.kind.into();

        AttachmentDescription {
            format: Some(format),
            samples,
            load_op: self.load_op,
            store_op: self.store_op,
            stencil_load_op: self.stencil_load_op,
            stencil_store_op: self.stencil_store_op,
            initial_layout,
            final_layout,
            ..Default::default()
        }
    }

    pub fn load_undefined(self) -> Self {
        Self {
            load_op: LoadOp::DontCare,
            clear_value: None,
            ..self
        }
    }

    pub fn load_stored(self) -> Self {
        Self {
            load_op: LoadOp::Load,
            ..self
        }
    }

    pub fn load_cleared(self, clear_value: ClearValue) -> Self {
        Self {
            load_op: LoadOp::Clear,
            clear_value: Some(clear_value),
            ..self
        }
    }

    pub fn store_maybe(self) -> Self {
        Self {
            store_op: StoreOp::DontCare,
            ..self
        }
    }

    pub fn store(self) -> Self {
        Self {
            store_op: StoreOp::Store,
            ..self
        }
    }

    pub fn load_stencil_undefined(self) -> Self {
        Self {
            stencil_load_op: LoadOp::DontCare,
            ..self
        }
    }

    pub fn load_stencil_stored(self) -> Self {
        Self {
            stencil_load_op: LoadOp::Load,
            ..self
        }
    }

    pub fn load_stencil_cleared(self) -> Self {
        Self {
            stencil_load_op: LoadOp::Clear,
            ..self
        }
    }

    pub fn store_stencil_maybe(self) -> Self {
        Self {
            stencil_store_op: StoreOp::DontCare,
            ..self
        }
    }

    pub fn store_stencil(self) -> Self {
        Self {
            stencil_store_op: StoreOp::Store,
            ..self
        }
    }
}
