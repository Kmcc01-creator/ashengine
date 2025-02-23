//! Pipeline builder and management system
//!
//! Provides a flexible builder pattern for creating graphics pipelines
//! with support for different render configurations.

use super::graph::PassId;
use crate::{
    error::Result,
    graphics::resource::{ResourceHandle, ResourceManager, ShaderStage},
};
use ash::vk;
use std::sync::Arc;

/// Vertex input rate for vertex attributes
#[derive(Debug, Clone, Copy)]
pub enum VertexInputRate {
    Vertex,
    Instance,
}

impl From<VertexInputRate> for vk::VertexInputRate {
    fn from(rate: VertexInputRate) -> Self {
        match rate {
            VertexInputRate::Vertex => vk::VertexInputRate::VERTEX,
            VertexInputRate::Instance => vk::VertexInputRate::INSTANCE,
        }
    }
}

/// Description of vertex attribute
#[derive(Debug, Clone)]
pub struct VertexAttributeDesc {
    pub location: u32,
    pub binding: u32,
    pub format: vk::Format,
    pub offset: u32,
}

/// Description of vertex binding
#[derive(Debug, Clone)]
pub struct VertexBindingDesc {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: VertexInputRate,
}

/// Configuration for rasterization
#[derive(Debug, Clone)]
pub struct RasterizationConfig {
    pub polygon_mode: vk::PolygonMode,
    pub cull_mode: vk::CullModeFlags,
    pub front_face: vk::FrontFace,
    pub line_width: f32,
}

impl Default for RasterizationConfig {
    fn default() -> Self {
        Self {
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
        }
    }
}

/// Configuration for depth testing
#[derive(Debug, Clone)]
pub struct DepthConfig {
    pub test_enable: bool,
    pub write_enable: bool,
    pub compare_op: vk::CompareOp,
}

impl Default for DepthConfig {
    fn default() -> Self {
        Self {
            test_enable: true,
            write_enable: true,
            compare_op: vk::CompareOp::LESS,
        }
    }
}

/// Builder for creating graphics pipelines
pub struct PipelineBuilder {
    device: Arc<ash::Device>,
    resource_manager: Arc<ResourceManager>,
    vertex_bindings: Vec<VertexBindingDesc>,
    vertex_attributes: Vec<VertexAttributeDesc>,
    shader_stages: Vec<(ShaderStage, ResourceHandle)>,
    rasterization: RasterizationConfig,
    depth: DepthConfig,
    blend_enable: bool,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
    push_constant_ranges: Vec<vk::PushConstantRange>,
    pipeline_cache: Option<vk::PipelineCache>,
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new(device: Arc<ash::Device>, resource_manager: Arc<ResourceManager>) -> Self {
        Self {
            device,
            resource_manager,
            vertex_bindings: Vec::new(),
            vertex_attributes: Vec::new(),
            shader_stages: Vec::new(),
            rasterization: RasterizationConfig::default(),
            depth: DepthConfig::default(),
            blend_enable: false,
            descriptor_set_layouts: Vec::new(),
            push_constant_ranges: Vec::new(),
            pipeline_cache: None,
        }
    }

    /// Add descriptor set layout
    pub fn add_descriptor_set_layout(mut self, layout: vk::DescriptorSetLayout) -> Self {
        self.descriptor_set_layouts.push(layout);
        self
    }

    /// Add push constant range
    pub fn add_push_constant_range(
        mut self,
        stage_flags: vk::ShaderStageFlags,
        range: std::ops::Range<u32>,
    ) -> Self {
        self.push_constant_ranges.push(
            vk::PushConstantRange::builder()
                .stage_flags(stage_flags)
                .offset(range.start)
                .size(range.end - range.start)
                .build(),
        );
        self
    }

    /// Set pipeline cache
    pub fn with_pipeline_cache(mut self, cache: vk::PipelineCache) -> Self {
        self.pipeline_cache = Some(cache);
        self
    }

    /// Add vertex binding
    pub fn add_vertex_binding(mut self, binding: VertexBindingDesc) -> Self {
        self.vertex_bindings.push(binding);
        self
    }

    /// Add vertex attribute
    pub fn add_vertex_attribute(mut self, attribute: VertexAttributeDesc) -> Self {
        self.vertex_attributes.push(attribute);
        self
    }

    /// Add shader stage
    pub fn add_shader(mut self, stage: ShaderStage, shader: ResourceHandle) -> Self {
        self.shader_stages.push((stage, shader));
        self
    }

    /// Set rasterization configuration
    pub fn rasterization(mut self, config: RasterizationConfig) -> Self {
        self.rasterization = config;
        self
    }

    /// Set depth configuration
    pub fn depth(mut self, config: DepthConfig) -> Self {
        self.depth = config;
        self
    }

    /// Enable/disable blending
    pub fn blend(mut self, enable: bool) -> Self {
        self.blend_enable = enable;
        self
    }

    /// Build the pipeline for a specific render pass
    pub fn build(&self, render_pass: vk::RenderPass, subpass: u32) -> Result<vk::Pipeline> {
        // Vertex input state
        let vertex_binding_descriptions: Vec<_> = self
            .vertex_bindings
            .iter()
            .map(|binding| {
                vk::VertexInputBindingDescription::builder()
                    .binding(binding.binding)
                    .stride(binding.stride)
                    .input_rate(binding.input_rate.into())
                    .build()
            })
            .collect();

        let vertex_attribute_descriptions: Vec<_> = self
            .vertex_attributes
            .iter()
            .map(|attr| {
                vk::VertexInputAttributeDescription::builder()
                    .location(attr.location)
                    .binding(attr.binding)
                    .format(attr.format)
                    .offset(attr.offset)
                    .build()
            })
            .collect();

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_binding_descriptions)
            .vertex_attribute_descriptions(&vertex_attribute_descriptions);

        // Input assembly state
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        // Shader stages
        let shader_stages: Vec<_> = self
            .shader_stages
            .iter()
            .filter_map(|(stage, handle)| self.resource_manager.get_shader_stage_info(*handle))
            .collect();

        // Rasterization state
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(self.rasterization.polygon_mode)
            .cull_mode(self.rasterization.cull_mode)
            .front_face(self.rasterization.front_face)
            .depth_bias_enable(false)
            .line_width(self.rasterization.line_width);

        // Multisample state
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        // Depth stencil state
        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(self.depth.test_enable)
            .depth_write_enable(self.depth.write_enable)
            .depth_compare_op(self.depth.compare_op)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        // Color blend state
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(self.blend_enable)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .build();

        let color_blend_attachments = [color_blend_attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        // Dynamic state
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        // Viewport state
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        // Create layout (TODO: Make this configurable)
        let layout_info = vk::PipelineLayoutCreateInfo::builder();
        let pipeline_layout = unsafe {
            self.device
                .create_pipeline_layout(&layout_info, None)
                .map_err(|e| crate::error::VulkanError::PipelineLayoutCreation(e.to_string()))?
        };

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
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(subpass);

        let pipeline = unsafe {
            self.device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info.build()], None)
                .map_err(|e| crate::error::VulkanError::PipelineCreation(e.to_string()))?[0]
        };

        Ok(pipeline)
    }
}
