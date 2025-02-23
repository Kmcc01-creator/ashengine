use ash::LoadingError;
use std::ffi::NulError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VulkanError {
    #[error("Failed to create instance: {0}")]
    InstanceCreation(String),

    #[error("Failed to create device: {0}")]
    DeviceCreation(String),

    #[error("Failed to create surface: {0}")]
    SurfaceCreation(String),

    #[error("Failed to create swapchain: {0}")]
    SwapchainCreation(String),

    #[error("Swapchain is out of date and needs recreation")]
    SwapchainOutOfDate,

    #[error("Swapchain is suboptimal for current conditions")]
    SwapchainSuboptimal,

    #[error("Failed to create image: {0}")]
    ImageCreation(String),

    #[error("Failed to create image view: {0}")]
    ImageViewCreation(String),

    #[error("Failed to create buffer: {0}")]
    BufferCreation(String),

    #[error("Failed to create render pass: {0}")]
    RenderPassCreation(String),

    #[error("Failed to create pipeline: {0}")]
    PipelineCreation(String),

    #[error("Failed to create pipeline layout: {0}")]
    PipelineLayoutCreation(String),

    #[error("Failed to create shader module: {0}")]
    ShaderCreation(String),

    #[error("Failed to create sampler: {0}")]
    SamplerCreation(String),

    #[error("Failed to create descriptor pool: {0}")]
    DescriptorPoolCreation(String),

    #[error("Failed to create descriptor set layout: {0}")]
    DescriptorSetLayoutCreation(String),

    #[error("Failed to allocate descriptor sets: {0}")]
    DescriptorSetAllocation(String),

    #[error("Failed to create framebuffer: {0}")]
    FramebufferCreation(String),

    #[error("Failed to create command pool: {0}")]
    CommandPoolCreation(String),

    #[error("Failed to allocate command buffers: {0}")]
    CommandBufferAllocation(String),

    #[error("Failed to begin command buffer: {0}")]
    CommandBufferBegin(String),

    #[error("Failed to end command buffer: {0}")]
    CommandBufferEnd(String),

    #[error("Failed to allocate memory: {0}")]
    MemoryAllocation(String),

    #[error("Failed to bind memory: {0}")]
    MemoryBinding(String),

    #[error("Failed to map memory: {0}")]
    MemoryMapping(String),

    #[error("Failed to create semaphore: {0}")]
    SemaphoreCreation(String),

    #[error("Failed to create fence: {0}")]
    FenceCreation(String),

    #[error("Failed to submit queue: {0}")]
    QueueSubmit(String),

    #[error("Failed to wait for queue idle: {0}")]
    QueueWaitIdle(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Window error: {0}")]
    WindowError(String),

    #[error("Invalid shader file: {0}")]
    InvalidShader(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("No suitable GPU found")]
    NoSuitableGpu,

    #[error("No suitable memory type found")]
    NoSuitableMemoryType,

    #[error("Synchronization error: {0}")]
    SyncError(String),

    #[error("General error: {0}")]
    General(String),
}

pub type Result<T> = std::result::Result<T, VulkanError>;

impl From<LoadingError> for VulkanError {
    fn from(e: LoadingError) -> Self {
        VulkanError::General(e.to_string())
    }
}

impl From<winit::error::OsError> for VulkanError {
    fn from(e: winit::error::OsError) -> Self {
        VulkanError::WindowError(e.to_string())
    }
}

impl From<NulError> for VulkanError {
    fn from(e: NulError) -> Self {
        VulkanError::General(e.to_string())
    }
}
