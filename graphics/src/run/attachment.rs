use std::sync::Arc;

use vulkano::image::ImageAccess;
use vulkano::image::SampleCount;
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::render_pass::AttachmentDescription;

use crate::run;
use crate::spec;

use super::IsMultisampled;

pub(super) enum AttachmentImage {
    Internal {
        //
    },
    Output,
}

pub(super) struct Attachment {
    id: u32,
    usage: spec::AttachmentUsage,
    format: vulkano::format::Format,
    is_input: bool,
    is_presentation: bool,
}
impl Attachment {
    pub fn build(
        id: u32,
        spec: &spec::Attachment,
        samples: SampleCount,
        swapchain: Arc<vulkano::swapchain::Swapchain>,
        is_input: bool,
    ) -> Self {
        let format = if let Some(ref format) = spec.format {
            get_vk_format(format, swapchain)
        } else {
            panic!("Format not defined for attachment {}", id);
        };

        let description = AttachmentDescription {
            format: Some(format),
            samples,
            load_op: spec.load_op.into(),
            store_op: spec.store_op.into(),
            stencil_load_op: vulkano::render_pass::LoadOp::DontCare,
            stencil_store_op: vulkano::render_pass::StoreOp::DontCare,
            initial_layout: spec.initial_layout.into(),
            final_layout: spec.final_layout.into(),
            ..Default::default()
        };

        Self {
            id,
            usage: spec.usage,
            format,
            is_input,
            is_presentation: spec.is_output,
        }
    }

    fn get_attachment_image_usage_flags(&self) -> vulkano::image::ImageUsage {
        let mut flags = vulkano::image::ImageUsage::empty();

        if self.usage == spec::AttachmentUsage::Depth {
            flags |= vulkano::image::ImageUsage::DEPTH_STENCIL_ATTACHMENT;
        }

        if self.is_input {
            flags |= vulkano::image::ImageUsage::INPUT_ATTACHMENT;
        }

        if !self.is_presentation {
            flags |= vulkano::image::ImageUsage::TRANSIENT_ATTACHMENT;
        }

        flags
    }

    pub fn make_image(
        &self,
        samples: SampleCount,
        swapchain_image: Arc<vulkano::image::SwapchainImage>,
        memory_allocator: &StandardMemoryAllocator,
    ) -> Arc<dyn vulkano::image::ImageViewAbstract> {
        let dimensions = swapchain_image.dimensions().width_height();

        let image_view = vulkano::image::view::ImageView::new_default(
            vulkano::image::attachment::AttachmentImage::multisampled_with_usage(
                memory_allocator,
                dimensions,
                samples,
                self.format,
                self.get_attachment_image_usage_flags(),
            )
            .unwrap(),
        )
        .unwrap();

        image_view
    }
}

impl From<spec::Layout> for vulkano::image::ImageLayout {
    fn from(value: spec::Layout) -> Self {
        match value {
            crate::Layout::Undefined => vulkano::image::ImageLayout::Undefined,
            crate::Layout::PresentationSource => vulkano::image::ImageLayout::PresentSrc,
            crate::Layout::Color => vulkano::image::ImageLayout::ColorAttachmentOptimal,
            crate::Layout::DepthStencil => {
                vulkano::image::ImageLayout::DepthStencilAttachmentOptimal
            }
            crate::Layout::DepthStencilReadOnly => {
                vulkano::image::ImageLayout::DepthStencilReadOnlyOptimal
            }
            crate::Layout::ShaderReadOnly => vulkano::image::ImageLayout::ShaderReadOnlyOptimal,
            crate::Layout::TransferSource => vulkano::image::ImageLayout::TransferSrcOptimal,
            crate::Layout::TransferDestination => vulkano::image::ImageLayout::TransferDstOptimal,
            crate::Layout::Preinitialized => vulkano::image::ImageLayout::Preinitialized,
        }
    }
}

impl From<spec::LoadOp> for vulkano::render_pass::LoadOp {
    fn from(value: spec::LoadOp) -> Self {
        match value {
            crate::LoadOp::Clear => vulkano::render_pass::LoadOp::Clear,
            crate::LoadOp::Load => vulkano::render_pass::LoadOp::Load,
            crate::LoadOp::DontCare => vulkano::render_pass::LoadOp::DontCare,
        }
    }
}

impl From<spec::StoreOp> for vulkano::render_pass::StoreOp {
    fn from(value: spec::StoreOp) -> Self {
        match value {
            crate::StoreOp::Store => vulkano::render_pass::StoreOp::Store,
            crate::StoreOp::DontCare => vulkano::render_pass::StoreOp::DontCare,
        }
    }
}

pub(super) fn get_vk_format(
    format: &spec::Format,
    swapchain: Arc<vulkano::swapchain::Swapchain>,
) -> vulkano::format::Format {
    match format {
        spec::Format::Absolute(format) => match format {
            spec::AbsoluteFormat::R4G4_UNORM_PACK8 => vulkano::format::Format::R4G4_UNORM_PACK8,
            spec::AbsoluteFormat::R4G4B4A4_UNORM_PACK16 => {
                vulkano::format::Format::R4G4B4A4_UNORM_PACK16
            }
            spec::AbsoluteFormat::B4G4R4A4_UNORM_PACK16 => {
                vulkano::format::Format::B4G4R4A4_UNORM_PACK16
            }
            spec::AbsoluteFormat::R5G6B5_UNORM_PACK16 => {
                vulkano::format::Format::R5G6B5_UNORM_PACK16
            }
            spec::AbsoluteFormat::B5G6R5_UNORM_PACK16 => {
                vulkano::format::Format::B5G6R5_UNORM_PACK16
            }
            spec::AbsoluteFormat::R5G5B5A1_UNORM_PACK16 => {
                vulkano::format::Format::R5G5B5A1_UNORM_PACK16
            }
            spec::AbsoluteFormat::B5G5R5A1_UNORM_PACK16 => {
                vulkano::format::Format::B5G5R5A1_UNORM_PACK16
            }
            spec::AbsoluteFormat::A1R5G5B5_UNORM_PACK16 => {
                vulkano::format::Format::A1R5G5B5_UNORM_PACK16
            }
            spec::AbsoluteFormat::R8_UNORM => vulkano::format::Format::R8_UNORM,
            spec::AbsoluteFormat::R8_SNORM => vulkano::format::Format::R8_SNORM,
            spec::AbsoluteFormat::R8_USCALED => vulkano::format::Format::R8_USCALED,
            spec::AbsoluteFormat::R8_SSCALED => vulkano::format::Format::R8_SSCALED,
            spec::AbsoluteFormat::R8_UINT => vulkano::format::Format::R8_UINT,
            spec::AbsoluteFormat::R8_SINT => vulkano::format::Format::R8_SINT,
            spec::AbsoluteFormat::R8_SRGB => vulkano::format::Format::R8_SRGB,
            spec::AbsoluteFormat::R8G8_UNORM => vulkano::format::Format::R8G8_UNORM,
            spec::AbsoluteFormat::R8G8_SNORM => vulkano::format::Format::R8G8_SNORM,
            spec::AbsoluteFormat::R8G8_USCALED => vulkano::format::Format::R8G8_USCALED,
            spec::AbsoluteFormat::R8G8_SSCALED => vulkano::format::Format::R8G8_SSCALED,
            spec::AbsoluteFormat::R8G8_UINT => vulkano::format::Format::R8G8_UINT,
            spec::AbsoluteFormat::R8G8_SINT => vulkano::format::Format::R8G8_SINT,
            spec::AbsoluteFormat::R8G8_SRGB => vulkano::format::Format::R8G8_SRGB,
            spec::AbsoluteFormat::R8G8B8_UNORM => vulkano::format::Format::R8G8B8_UNORM,
            spec::AbsoluteFormat::R8G8B8_SNORM => vulkano::format::Format::R8G8B8_SNORM,
            spec::AbsoluteFormat::R8G8B8_USCALED => vulkano::format::Format::R8G8B8_USCALED,
            spec::AbsoluteFormat::R8G8B8_SSCALED => vulkano::format::Format::R8G8B8_SSCALED,
            spec::AbsoluteFormat::R8G8B8_UINT => vulkano::format::Format::R8G8B8_UINT,
            spec::AbsoluteFormat::R8G8B8_SINT => vulkano::format::Format::R8G8B8_SINT,
            spec::AbsoluteFormat::R8G8B8_SRGB => vulkano::format::Format::R8G8B8_SRGB,
            spec::AbsoluteFormat::B8G8R8_UNORM => vulkano::format::Format::B8G8R8_UNORM,
            spec::AbsoluteFormat::B8G8R8_SNORM => vulkano::format::Format::B8G8R8_SNORM,
            spec::AbsoluteFormat::B8G8R8_USCALED => vulkano::format::Format::B8G8R8_USCALED,
            spec::AbsoluteFormat::B8G8R8_SSCALED => vulkano::format::Format::B8G8R8_SSCALED,
            spec::AbsoluteFormat::B8G8R8_UINT => vulkano::format::Format::B8G8R8_UINT,
            spec::AbsoluteFormat::B8G8R8_SINT => vulkano::format::Format::B8G8R8_SINT,
            spec::AbsoluteFormat::B8G8R8_SRGB => vulkano::format::Format::B8G8R8_SRGB,
            spec::AbsoluteFormat::R8G8B8A8_UNORM => vulkano::format::Format::R8G8B8A8_UNORM,
            spec::AbsoluteFormat::R8G8B8A8_SNORM => vulkano::format::Format::R8G8B8A8_SNORM,
            spec::AbsoluteFormat::R8G8B8A8_USCALED => vulkano::format::Format::R8G8B8A8_USCALED,
            spec::AbsoluteFormat::R8G8B8A8_SSCALED => vulkano::format::Format::R8G8B8A8_SSCALED,
            spec::AbsoluteFormat::R8G8B8A8_UINT => vulkano::format::Format::R8G8B8A8_UINT,
            spec::AbsoluteFormat::R8G8B8A8_SINT => vulkano::format::Format::R8G8B8A8_SINT,
            spec::AbsoluteFormat::R8G8B8A8_SRGB => vulkano::format::Format::R8G8B8A8_SRGB,
            spec::AbsoluteFormat::B8G8R8A8_UNORM => vulkano::format::Format::B8G8R8A8_UNORM,
            spec::AbsoluteFormat::B8G8R8A8_SNORM => vulkano::format::Format::B8G8R8A8_SNORM,
            spec::AbsoluteFormat::B8G8R8A8_USCALED => vulkano::format::Format::B8G8R8A8_USCALED,
            spec::AbsoluteFormat::B8G8R8A8_SSCALED => vulkano::format::Format::B8G8R8A8_SSCALED,
            spec::AbsoluteFormat::B8G8R8A8_UINT => vulkano::format::Format::B8G8R8A8_UINT,
            spec::AbsoluteFormat::B8G8R8A8_SINT => vulkano::format::Format::B8G8R8A8_SINT,
            spec::AbsoluteFormat::B8G8R8A8_SRGB => vulkano::format::Format::B8G8R8A8_SRGB,
            spec::AbsoluteFormat::A8B8G8R8_UNORM_PACK32 => {
                vulkano::format::Format::A8B8G8R8_UNORM_PACK32
            }
            spec::AbsoluteFormat::A8B8G8R8_SNORM_PACK32 => {
                vulkano::format::Format::A8B8G8R8_SNORM_PACK32
            }
            spec::AbsoluteFormat::A8B8G8R8_USCALED_PACK32 => {
                vulkano::format::Format::A8B8G8R8_USCALED_PACK32
            }
            spec::AbsoluteFormat::A8B8G8R8_SSCALED_PACK32 => {
                vulkano::format::Format::A8B8G8R8_SSCALED_PACK32
            }
            spec::AbsoluteFormat::A8B8G8R8_UINT_PACK32 => {
                vulkano::format::Format::A8B8G8R8_UINT_PACK32
            }
            spec::AbsoluteFormat::A8B8G8R8_SINT_PACK32 => {
                vulkano::format::Format::A8B8G8R8_SINT_PACK32
            }
            spec::AbsoluteFormat::A8B8G8R8_SRGB_PACK32 => {
                vulkano::format::Format::A8B8G8R8_SRGB_PACK32
            }
            spec::AbsoluteFormat::A2R10G10B10_UNORM_PACK32 => {
                vulkano::format::Format::A2R10G10B10_UNORM_PACK32
            }
            spec::AbsoluteFormat::A2R10G10B10_SNORM_PACK32 => {
                vulkano::format::Format::A2R10G10B10_SNORM_PACK32
            }
            spec::AbsoluteFormat::A2R10G10B10_USCALED_PACK32 => {
                vulkano::format::Format::A2R10G10B10_USCALED_PACK32
            }
            spec::AbsoluteFormat::A2R10G10B10_SSCALED_PACK32 => {
                vulkano::format::Format::A2R10G10B10_SSCALED_PACK32
            }
            spec::AbsoluteFormat::A2R10G10B10_UINT_PACK32 => {
                vulkano::format::Format::A2R10G10B10_UINT_PACK32
            }
            spec::AbsoluteFormat::A2R10G10B10_SINT_PACK32 => {
                vulkano::format::Format::A2R10G10B10_SINT_PACK32
            }
            spec::AbsoluteFormat::A2B10G10R10_UNORM_PACK32 => {
                vulkano::format::Format::A2B10G10R10_UNORM_PACK32
            }
            spec::AbsoluteFormat::A2B10G10R10_SNORM_PACK32 => {
                vulkano::format::Format::A2B10G10R10_SNORM_PACK32
            }
            spec::AbsoluteFormat::A2B10G10R10_USCALED_PACK32 => {
                vulkano::format::Format::A2B10G10R10_USCALED_PACK32
            }
            spec::AbsoluteFormat::A2B10G10R10_SSCALED_PACK32 => {
                vulkano::format::Format::A2B10G10R10_SSCALED_PACK32
            }
            spec::AbsoluteFormat::A2B10G10R10_UINT_PACK32 => {
                vulkano::format::Format::A2B10G10R10_UINT_PACK32
            }
            spec::AbsoluteFormat::A2B10G10R10_SINT_PACK32 => {
                vulkano::format::Format::A2B10G10R10_SINT_PACK32
            }
            spec::AbsoluteFormat::R16_UNORM => vulkano::format::Format::R16_UNORM,
            spec::AbsoluteFormat::R16_SNORM => vulkano::format::Format::R16_SNORM,
            spec::AbsoluteFormat::R16_USCALED => vulkano::format::Format::R16_USCALED,
            spec::AbsoluteFormat::R16_SSCALED => vulkano::format::Format::R16_SSCALED,
            spec::AbsoluteFormat::R16_UINT => vulkano::format::Format::R16_UINT,
            spec::AbsoluteFormat::R16_SINT => vulkano::format::Format::R16_SINT,
            spec::AbsoluteFormat::R16_SFLOAT => vulkano::format::Format::R16_SFLOAT,
            spec::AbsoluteFormat::R16G16_UNORM => vulkano::format::Format::R16G16_UNORM,
            spec::AbsoluteFormat::R16G16_SNORM => vulkano::format::Format::R16G16_SNORM,
            spec::AbsoluteFormat::R16G16_USCALED => vulkano::format::Format::R16G16_USCALED,
            spec::AbsoluteFormat::R16G16_SSCALED => vulkano::format::Format::R16G16_SSCALED,
            spec::AbsoluteFormat::R16G16_UINT => vulkano::format::Format::R16G16_UINT,
            spec::AbsoluteFormat::R16G16_SINT => vulkano::format::Format::R16G16_SINT,
            spec::AbsoluteFormat::R16G16_SFLOAT => vulkano::format::Format::R16G16_SFLOAT,
            spec::AbsoluteFormat::R16G16B16_UNORM => vulkano::format::Format::R16G16B16_UNORM,
            spec::AbsoluteFormat::R16G16B16_SNORM => vulkano::format::Format::R16G16B16_SNORM,
            spec::AbsoluteFormat::R16G16B16_USCALED => vulkano::format::Format::R16G16B16_USCALED,
            spec::AbsoluteFormat::R16G16B16_SSCALED => vulkano::format::Format::R16G16B16_SSCALED,
            spec::AbsoluteFormat::R16G16B16_UINT => vulkano::format::Format::R16G16B16_UINT,
            spec::AbsoluteFormat::R16G16B16_SINT => vulkano::format::Format::R16G16B16_SINT,
            spec::AbsoluteFormat::R16G16B16_SFLOAT => vulkano::format::Format::R16G16B16_SFLOAT,
            spec::AbsoluteFormat::R16G16B16A16_UNORM => vulkano::format::Format::R16G16B16A16_UNORM,
            spec::AbsoluteFormat::R16G16B16A16_SNORM => vulkano::format::Format::R16G16B16A16_SNORM,
            spec::AbsoluteFormat::R16G16B16A16_USCALED => {
                vulkano::format::Format::R16G16B16A16_USCALED
            }
            spec::AbsoluteFormat::R16G16B16A16_SSCALED => {
                vulkano::format::Format::R16G16B16A16_SSCALED
            }
            spec::AbsoluteFormat::R16G16B16A16_UINT => vulkano::format::Format::R16G16B16A16_UINT,
            spec::AbsoluteFormat::R16G16B16A16_SINT => vulkano::format::Format::R16G16B16A16_SINT,
            spec::AbsoluteFormat::R16G16B16A16_SFLOAT => {
                vulkano::format::Format::R16G16B16A16_SFLOAT
            }
            spec::AbsoluteFormat::R32_UINT => vulkano::format::Format::R32_UINT,
            spec::AbsoluteFormat::R32_SINT => vulkano::format::Format::R32_SINT,
            spec::AbsoluteFormat::R32_SFLOAT => vulkano::format::Format::R32_SFLOAT,
            spec::AbsoluteFormat::R32G32_UINT => vulkano::format::Format::R32G32_UINT,
            spec::AbsoluteFormat::R32G32_SINT => vulkano::format::Format::R32G32_SINT,
            spec::AbsoluteFormat::R32G32_SFLOAT => vulkano::format::Format::R32G32_SFLOAT,
            spec::AbsoluteFormat::R32G32B32_UINT => vulkano::format::Format::R32G32B32_UINT,
            spec::AbsoluteFormat::R32G32B32_SINT => vulkano::format::Format::R32G32B32_SINT,
            spec::AbsoluteFormat::R32G32B32_SFLOAT => vulkano::format::Format::R32G32B32_SFLOAT,
            spec::AbsoluteFormat::R32G32B32A32_UINT => vulkano::format::Format::R32G32B32A32_UINT,
            spec::AbsoluteFormat::R32G32B32A32_SINT => vulkano::format::Format::R32G32B32A32_SINT,
            spec::AbsoluteFormat::R32G32B32A32_SFLOAT => {
                vulkano::format::Format::R32G32B32A32_SFLOAT
            }
            spec::AbsoluteFormat::R64_UINT => vulkano::format::Format::R64_UINT,
            spec::AbsoluteFormat::R64_SINT => vulkano::format::Format::R64_SINT,
            spec::AbsoluteFormat::R64_SFLOAT => vulkano::format::Format::R64_SFLOAT,
            spec::AbsoluteFormat::R64G64_UINT => vulkano::format::Format::R64G64_UINT,
            spec::AbsoluteFormat::R64G64_SINT => vulkano::format::Format::R64G64_SINT,
            spec::AbsoluteFormat::R64G64_SFLOAT => vulkano::format::Format::R64G64_SFLOAT,
            spec::AbsoluteFormat::R64G64B64_UINT => vulkano::format::Format::R64G64B64_UINT,
            spec::AbsoluteFormat::R64G64B64_SINT => vulkano::format::Format::R64G64B64_SINT,
            spec::AbsoluteFormat::R64G64B64_SFLOAT => vulkano::format::Format::R64G64B64_SFLOAT,
            spec::AbsoluteFormat::R64G64B64A64_UINT => vulkano::format::Format::R64G64B64A64_UINT,
            spec::AbsoluteFormat::R64G64B64A64_SINT => vulkano::format::Format::R64G64B64A64_SINT,
            spec::AbsoluteFormat::R64G64B64A64_SFLOAT => {
                vulkano::format::Format::R64G64B64A64_SFLOAT
            }
            spec::AbsoluteFormat::B10G11R11_UFLOAT_PACK32 => {
                vulkano::format::Format::B10G11R11_UFLOAT_PACK32
            }
            spec::AbsoluteFormat::E5B9G9R9_UFLOAT_PACK32 => {
                vulkano::format::Format::E5B9G9R9_UFLOAT_PACK32
            }
            spec::AbsoluteFormat::D16_UNORM => vulkano::format::Format::D16_UNORM,
            spec::AbsoluteFormat::X8_D24_UNORM_PACK32 => {
                vulkano::format::Format::X8_D24_UNORM_PACK32
            }
            spec::AbsoluteFormat::D32_SFLOAT => vulkano::format::Format::D32_SFLOAT,
            spec::AbsoluteFormat::S8_UINT => vulkano::format::Format::S8_UINT,
            spec::AbsoluteFormat::D16_UNORM_S8_UINT => vulkano::format::Format::D16_UNORM_S8_UINT,
            spec::AbsoluteFormat::D24_UNORM_S8_UINT => vulkano::format::Format::D24_UNORM_S8_UINT,
            spec::AbsoluteFormat::D32_SFLOAT_S8_UINT => vulkano::format::Format::D32_SFLOAT_S8_UINT,
            spec::AbsoluteFormat::BC1_RGB_UNORM_BLOCK => {
                vulkano::format::Format::BC1_RGB_UNORM_BLOCK
            }
            spec::AbsoluteFormat::BC1_RGB_SRGB_BLOCK => vulkano::format::Format::BC1_RGB_SRGB_BLOCK,
            spec::AbsoluteFormat::BC1_RGBA_UNORM_BLOCK => {
                vulkano::format::Format::BC1_RGBA_UNORM_BLOCK
            }
            spec::AbsoluteFormat::BC1_RGBA_SRGB_BLOCK => {
                vulkano::format::Format::BC1_RGBA_SRGB_BLOCK
            }
            spec::AbsoluteFormat::BC2_UNORM_BLOCK => vulkano::format::Format::BC2_UNORM_BLOCK,
            spec::AbsoluteFormat::BC2_SRGB_BLOCK => vulkano::format::Format::BC2_SRGB_BLOCK,
            spec::AbsoluteFormat::BC3_UNORM_BLOCK => vulkano::format::Format::BC3_UNORM_BLOCK,
            spec::AbsoluteFormat::BC3_SRGB_BLOCK => vulkano::format::Format::BC3_SRGB_BLOCK,
            spec::AbsoluteFormat::BC4_UNORM_BLOCK => vulkano::format::Format::BC4_UNORM_BLOCK,
            spec::AbsoluteFormat::BC4_SNORM_BLOCK => vulkano::format::Format::BC4_SNORM_BLOCK,
            spec::AbsoluteFormat::BC5_UNORM_BLOCK => vulkano::format::Format::BC5_UNORM_BLOCK,
            spec::AbsoluteFormat::BC5_SNORM_BLOCK => vulkano::format::Format::BC5_SNORM_BLOCK,
            spec::AbsoluteFormat::BC6H_UFLOAT_BLOCK => vulkano::format::Format::BC6H_UFLOAT_BLOCK,
            spec::AbsoluteFormat::BC6H_SFLOAT_BLOCK => vulkano::format::Format::BC6H_SFLOAT_BLOCK,
            spec::AbsoluteFormat::BC7_UNORM_BLOCK => vulkano::format::Format::BC7_UNORM_BLOCK,
            spec::AbsoluteFormat::BC7_SRGB_BLOCK => vulkano::format::Format::BC7_SRGB_BLOCK,
            spec::AbsoluteFormat::ETC2_R8G8B8_UNORM_BLOCK => {
                vulkano::format::Format::ETC2_R8G8B8_UNORM_BLOCK
            }
            spec::AbsoluteFormat::ETC2_R8G8B8_SRGB_BLOCK => {
                vulkano::format::Format::ETC2_R8G8B8_SRGB_BLOCK
            }
            spec::AbsoluteFormat::ETC2_R8G8B8A1_UNORM_BLOCK => {
                vulkano::format::Format::ETC2_R8G8B8A1_UNORM_BLOCK
            }
            spec::AbsoluteFormat::ETC2_R8G8B8A1_SRGB_BLOCK => {
                vulkano::format::Format::ETC2_R8G8B8A1_SRGB_BLOCK
            }
            spec::AbsoluteFormat::ETC2_R8G8B8A8_UNORM_BLOCK => {
                vulkano::format::Format::ETC2_R8G8B8A8_UNORM_BLOCK
            }
            spec::AbsoluteFormat::ETC2_R8G8B8A8_SRGB_BLOCK => {
                vulkano::format::Format::ETC2_R8G8B8A8_SRGB_BLOCK
            }
            spec::AbsoluteFormat::EAC_R11_UNORM_BLOCK => {
                vulkano::format::Format::EAC_R11_UNORM_BLOCK
            }
            spec::AbsoluteFormat::EAC_R11_SNORM_BLOCK => {
                vulkano::format::Format::EAC_R11_SNORM_BLOCK
            }
            spec::AbsoluteFormat::EAC_R11G11_UNORM_BLOCK => {
                vulkano::format::Format::EAC_R11G11_UNORM_BLOCK
            }
            spec::AbsoluteFormat::EAC_R11G11_SNORM_BLOCK => {
                vulkano::format::Format::EAC_R11G11_SNORM_BLOCK
            }
        },
        spec::Format::Ref(reference) => match reference {
            spec::FormatRef::Surface => swapchain.image_format(),
        },
    }
}
