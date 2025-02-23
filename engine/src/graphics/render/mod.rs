//! Render system module providing high-level rendering abstractions
//!
//! This module contains components for:
//! - Render graph management
//! - Pipeline creation and configuration
//! - Pass management for deferred rendering

pub mod graph;
pub mod pass;
pub mod pipeline;

use crate::{
    error::Result,
    graphics::resource::{ResourceHandle, ResourceManager, TextureFormat},
};
use ash::vk;
use std::collections::HashSet;
use std::sync::Arc;

pub use self::{
    graph::{AttachmentDesc, AttachmentType, PassDesc, PassId, RenderGraph},
    pass::{PassConfig, PassManager, PassType},
    pipeline::{DepthConfig, PipelineBuilder, RasterizationConfig},
};

// Extended texture formats needed for deferred rendering
impl TextureFormat {
    pub const R32G32B32A32_SFLOAT: Self = Self::Custom(vk::Format::R32G32B32A32_SFLOAT);
    pub const R16G16B16A16_SFLOAT: Self = Self::Custom(vk::Format::R16G16B16A16_SFLOAT);
    pub const R8G8B8A8_UNORM: Self = Self::Custom(vk::Format::R8G8B8A8_UNORM);
    pub const D32_SFLOAT: Self = Self::Custom(vk::Format::D32_SFLOAT);

    pub(crate) fn to_vk_format(&self) -> vk::Format {
        match self {
            TextureFormat::R8G8B8A8Unorm => vk::Format::R8G8B8A8_UNORM,
            TextureFormat::B8G8R8A8Unorm => vk::Format::B8G8R8A8_UNORM,
            TextureFormat::R8G8B8Unorm => vk::Format::R8G8B8_UNORM,
            TextureFormat::R8Unorm => vk::Format::R8_UNORM,
            TextureFormat::Depth32Float => vk::Format::D32_SFLOAT,
            TextureFormat::Custom(format) => *format,
        }
    }
}

/// Configuration for deferred rendering
#[derive(Debug, Clone)]
pub struct DeferredConfig {
    pub width: u32,
    pub height: u32,
    pub samples: vk::SampleCountFlags,
}

/// Main render system managing all rendering operations
pub struct RenderSystem {
    device: Arc<ash::Device>,
    resource_manager: Arc<ResourceManager>,
    graph: RenderGraph,
    pass_manager: PassManager,
}

impl RenderSystem {
    /// Create a new render system
    pub fn new(device: Arc<ash::Device>, resource_manager: Arc<ResourceManager>) -> Self {
        Self {
            graph: RenderGraph::new(device.clone(), resource_manager.clone()),
            pass_manager: PassManager::new(device.clone(), resource_manager.clone()),
            device,
            resource_manager,
        }
    }

    /// Initialize deferred rendering pipeline
    pub fn init_deferred(&mut self, config: DeferredConfig) -> Result<()> {
        // Create geometry pass
        let gbuffer_config =
            self.pass_manager
                .create_geometry_pass(config.width, config.height, config.samples);
        let gbuffer_desc = self.pass_manager.create_pass_desc(&gbuffer_config);
        let gbuffer_pass_id = self.graph.add_pass(gbuffer_desc)?;

        // Create lighting pass
        let lighting_config =
            self.pass_manager
                .create_lighting_pass(config.width, config.height, config.samples);
        let mut lighting_desc = self.pass_manager.create_pass_desc(&lighting_config);

        // Add dependency on geometry pass
        lighting_desc.dependencies.insert(gbuffer_pass_id);

        // Add G-buffer attachments as inputs
        lighting_desc.input_attachments = vec![0, 1, 2, 3]; // Position, Normal, Albedo, Depth

        self.graph.add_pass(lighting_desc)?;

        Ok(())
    }

    /// Get the render graph
    pub fn graph(&self) -> &RenderGraph {
        &self.graph
    }

    /// Get a mutable reference to the render graph
    pub fn graph_mut(&mut self) -> &mut RenderGraph {
        &mut self.graph
    }

    /// Get the pass manager
    pub fn pass_manager(&self) -> &PassManager {
        &self.pass_manager
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    R8G8B8A8Unorm,
    B8G8R8A8Unorm,
    R8G8B8Unorm,
    R8Unorm,
    Depth32Float,
    Custom(vk::Format),
}
