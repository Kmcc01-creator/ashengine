//! Enhanced pipeline system with caching and variants
//!
//! Provides a flexible pipeline system with support for:
//! - Pipeline variants and specialization
//! - State caching and reuse
//! - Dynamic state configuration
//! - Descriptor set management

use crate::{
    error::Result,
    graphics::resource::{ResourceHandle, ResourceManager, ShaderStage},
};
use ash::vk;
use std::sync::Arc;

mod cache;
mod config;
mod layout;
mod state;
mod variants;

pub use config::{BlendMode, DepthConfig, RasterizationConfig, VertexConfig};
pub use layout::{
    BindGroupLayout, DescriptorSetLayoutCache, PipelineLayoutCache, PipelineLayoutDesc,
};
pub use variants::{PipelineKey, PipelineVariant, SpecConstantValue, SpecializationInfo};

/// Manager for creating and caching pipeline objects
pub struct PipelineManager {
    device: Arc<ash::Device>,
    resource_manager: Arc<ResourceManager>,
    layout_cache: PipelineLayoutCache,
    pipeline_cache: cache::PipelineCache,
}

impl PipelineManager {
    /// Create a new pipeline manager
    pub fn new(device: Arc<ash::Device>, resource_manager: Arc<ResourceManager>) -> Result<Self> {
        Ok(Self {
            layout_cache: PipelineLayoutCache::new(device.clone()),
            pipeline_cache: cache::PipelineCache::new(device.clone(), 1000)?, // TODO: Make configurable
            device,
            resource_manager,
        })
    }

    /// Get or create a pipeline for the given variant
    pub fn get_pipeline(&mut self, variant: PipelineVariant) -> Result<vk::Pipeline> {
        // Check cache first
        if let Some(pipeline) = self.pipeline_cache.get(&variant) {
            return Ok(pipeline);
        }

        // Create new pipeline
        let pipeline = self.create_pipeline(&variant)?;
        self.pipeline_cache.insert(variant, pipeline);
        Ok(pipeline)
    }

    /// Create a pipeline layout from descriptor
    pub fn create_layout(&mut self, desc: &PipelineLayoutDesc) -> Result<vk::PipelineLayout> {
        self.layout_cache.get_or_create(desc)
    }

    /// Save pipeline cache to disk
    pub fn save_cache(&self, path: &std::path::Path) -> Result<()> {
        self.pipeline_cache.save_to_disk(path)
    }

    /// Load pipeline cache from disk
    pub fn load_cache(&self, path: &std::path::Path) -> Result<()> {
        self.pipeline_cache.load_from_disk(path)
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> cache::CacheStats {
        self.pipeline_cache.stats()
    }

    // Private helpers

    fn create_pipeline(&mut self, variant: &PipelineVariant) -> Result<vk::Pipeline> {
        // Get render pass
        let render_pass = self
            .resource_manager
            .get_render_pass(variant.base.render_pass)
            .ok_or_else(|| {
                crate::error::VulkanError::InvalidResource("Render pass not found".into())
            })?;

        // Create pipeline layout
        let layout_desc = self.create_layout_desc(variant)?;
        let layout = self.create_layout(&layout_desc)?;

        // Create shader stages
        let shader_stages = self.create_shader_stages(variant)?;

        // Create pipeline states using state module
        let vertex_input_state = state::create_vertex_input_state(&variant.state.vertex_config);
        let rasterization_state = state::create_rasterization_state(&variant.state.rasterization);
        let multisample_state = state::create_multisample_state(&variant.state.multisample);
        let depth_stencil_state = state::create_depth_stencil_state(&variant.state.depth_config);
        let color_blend_state = state::create_color_blend_state(&variant.state.blend_mode);
        let dynamic_state = state::create_dynamic_state(&variant.state.dynamic_state);

        // Input assembly state
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        // Viewport state
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        // Create pipeline
        let create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(layout)
            .render_pass(render_pass)
            .subpass(variant.base.subpass);

        let pipeline = unsafe {
            self.device
                .create_graphics_pipelines(
                    self.pipeline_cache.vk_cache(),
                    &[create_info.build()],
                    None,
                )
                .map_err(|e| crate::error::VulkanError::PipelineCreation(e.1.to_string()))?[0]
        };

        Ok(pipeline)
    }

    fn create_layout_desc(&self, variant: &PipelineVariant) -> Result<PipelineLayoutDesc> {
        // TODO: Create layout description from shader reflection
        Ok(PipelineLayoutDesc {
            bind_group_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
        })
    }

    fn create_shader_stages(
        &self,
        variant: &PipelineVariant,
    ) -> Result<Vec<vk::PipelineShaderStageCreateInfo>> {
        let mut stages = Vec::new();

        for (stage_flags, shader) in &variant.base.shaders {
            if let Some(mut stage_info) = self.resource_manager.get_shader_stage_info(*shader) {
                // Apply specialization if available
                if let Some(spec_info) = &variant.specialization {
                    if spec_info.stages.contains(*stage_flags) {
                        let (info, _data, _entries) = spec_info.create_info();
                        stage_info = stage_info.specialization_info(&info);
                    }
                }
                stages.push(stage_info.build());
            }
        }

        Ok(stages)
    }
}

impl Drop for PipelineManager {
    fn drop(&mut self) {
        // Pipeline cache cleanup is handled by its Drop impl
    }
}
