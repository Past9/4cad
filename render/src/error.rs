use thiserror::Error;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilderContextError, BeginRenderPassError, BuildError,
        CommandBufferExecError, DrawError,
    },
    device::{physical::SurfacePropertiesError, DeviceCreationError},
    image::{view::ImageViewCreationError, ImageCreationError},
    instance::InstanceCreationError,
    memory::DeviceMemoryAllocationError,
    pipeline::graphics::GraphicsPipelineCreationError,
    render_pass::{FramebufferCreationError, RenderPassCreationError},
    shader::ShaderCreationError,
    swapchain::{AcquireError, SwapchainCreationError},
    OomError,
};
use vulkano_win::CreationError;

pub type RenderResult<T> = Result<T, RenderError>;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Error creating Vulkan instance")]
    InstanceCreation(#[from] InstanceCreationError),

    #[error("Error creating window")]
    WindowCreation(#[from] CreationError),

    #[error("No device was created")]
    NoDevice(String),

    #[error("Error creating device")]
    DeviceCreation(#[from] DeviceCreationError),

    #[error("No queues are available")]
    NoQueues,

    #[error("Error getting surface properties")]
    SurfaceProperties(#[from] SurfacePropertiesError),

    #[error("No composite alpha supported")]
    NoCompositeAlpha,

    #[error("Error creating swapchain")]
    SwapchainCreation(#[from] SwapchainCreationError),

    #[error("Error creating render passes")]
    RenderPassCreation(#[from] RenderPassCreationError),

    #[error("Error creating shader")]
    ShaderCreation(#[from] ShaderCreationError),

    #[error("No shader entry point")]
    NoShaderEntryPoint,

    #[error("No render subpass")]
    NoRenderSubpass,

    #[error("Error creating graphics pipeline")]
    GraphicsPipelineCreation(#[from] GraphicsPipelineCreationError),

    #[error("No previous frame")]
    NoPreviousFrame,

    #[error("Failed to acquire next swapchain image")]
    NextSwapchainImage(AcquireError),

    #[error("Out of memory")]
    OutOfMemory(#[from] OomError),

    #[error("Error while beginning render pass")]
    BeginRenderPass(#[from] BeginRenderPassError),

    #[error("Error while drawing")]
    Draw(#[from] DrawError),

    #[error("Error while allocating device memory")]
    DeviceMemoryAllocation(#[from] DeviceMemoryAllocationError),

    #[error("Auto command buffer builder context error")]
    AutoCommandBufferBuilderContext(#[from] AutoCommandBufferBuilderContextError),

    #[error("Error while building command buffer")]
    CommandBufferBuild(#[from] BuildError),

    #[error("Error while executing command buffer")]
    CommandBufferExecution(#[from] CommandBufferExecError),

    #[error("Error while creating image")]
    ImageCreation(#[from] ImageCreationError),

    #[error("Error while creating image view")]
    ImageViewCreation(#[from] ImageViewCreationError),

    #[error("Error while creating framebuffer")]
    FramebufferCreation(#[from] FramebufferCreationError),
}
