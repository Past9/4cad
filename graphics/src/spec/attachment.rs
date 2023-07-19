use super::{AbsoluteFormat, Format};

#[derive(Debug, Clone, Copy)]
pub struct AttachmentId(pub(crate) usize);
impl From<usize> for AttachmentId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct Attachment {
    pub(crate) is_output: bool,
    pub(crate) usage: AttachmentUsage,
    pub(crate) format: Format,
    pub(crate) load_op: LoadOp,
    pub(crate) store_op: StoreOp,
    pub(crate) initial_layout: Layout,
    pub(crate) final_layout: Layout,
}
impl Attachment {
    pub fn color() -> Self {
        Self::new(AttachmentUsage::Color)
    }

    pub fn depth() -> Self {
        Self::new(AttachmentUsage::Depth)
    }

    pub fn new(usage: AttachmentUsage) -> Self {
        Self {
            is_output: false,
            usage,
            format: Format::absolute(AbsoluteFormat::UNDEFINED),
            load_op: LoadOp::DontCare,
            store_op: StoreOp::Store,
            initial_layout: Layout::Undefined,
            final_layout: Layout::Undefined,
        }
    }

    pub fn output(mut self) -> Self {
        self.is_output = true;
        self
    }

    pub fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    pub fn load_op(mut self, op: LoadOp) -> Self {
        self.load_op = op;
        self
    }

    pub fn store_op(mut self, op: StoreOp) -> Self {
        self.store_op = op;
        self
    }

    pub fn initial_layout(mut self, layout: Layout) -> Self {
        self.initial_layout = layout;
        self
    }

    pub fn final_layout(mut self, layout: Layout) -> Self {
        self.final_layout = layout;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AttachmentUsage {
    Color,
    Depth,
}

#[derive(Debug, Clone, Copy)]
pub enum LoadOp {
    Clear,
    Load,
    DontCare,
}

#[derive(Debug, Clone, Copy)]
pub enum StoreOp {
    Store,
    DontCare,
}

#[derive(Debug, Clone, Copy)]
pub enum Layout {
    Undefined,
    PresentationSource,
    Color,
    DepthStencil,
    DepthStencilReadOnly,
    ShaderReadOnly,
    TransferSource,
    TransferDestination,
    Preinitialized,
}
