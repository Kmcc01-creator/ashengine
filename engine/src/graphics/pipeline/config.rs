//! Pipeline configuration types
//!
//! Provides configuration structs for different aspects of pipeline state

use ash::vk;
use std::hash::{Hash, Hasher};

/// Blend modes for color attachments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// No blending (replace)
    None,
    /// Alpha blending (standard transparent blend)
    Alpha,
    /// Additive blending (add source to destination)
    Add,
    /// Multiplicative blending (multiply source with destination)
    Multiply,
    /// Custom blend mode
    Custom {
        src_color: vk::BlendFactor,
        dst_color: vk::BlendFactor,
        color_op: vk::BlendOp,
        src_alpha: vk::BlendFactor,
        dst_alpha: vk::BlendFactor,
        alpha_op: vk::BlendOp,
    },
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::None
    }
}

/// Configuration for depth testing and writing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepthConfig {
    pub test_enable: bool,
    pub write_enable: bool,
    pub compare_op: vk::CompareOp,
    pub bounds_test: Option<(f32, f32)>,
}

impl Default for DepthConfig {
    fn default() -> Self {
        Self {
            test_enable: true,
            write_enable: true,
            compare_op: vk::CompareOp::LESS,
            bounds_test: None,
        }
    }
}

/// Input rate for vertex attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexInputRate {
    /// Advance per vertex
    Vertex,
    /// Advance per instance
    Instance(u32),
}

/// Description of vertex attribute
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexAttribute {
    pub location: u32,
    pub binding: u32,
    pub format: vk::Format,
    pub offset: u32,
}

/// Description of vertex binding
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexBinding {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: VertexInputRate,
}

/// Configuration for vertex input
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct VertexConfig {
    pub bindings: Vec<VertexBinding>,
    pub attributes: Vec<VertexAttribute>,
}

/// Rasterization configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RasterizationConfig {
    pub polygon_mode: vk::PolygonMode,
    pub cull_mode: vk::CullModeFlags,
    pub front_face: vk::FrontFace,
    pub depth_bias: Option<(f32, f32, f32)>, // constant, clamp, slope
    pub line_width: f32,
}

impl Default for RasterizationConfig {
    fn default() -> Self {
        Self {
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            depth_bias: None,
            line_width: 1.0,
        }
    }
}

/// Multisample configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MultisampleConfig {
    pub samples: vk::SampleCountFlags,
    pub sample_shading: Option<f32>,
    pub sample_mask: Option<u64>,
}

impl Default for MultisampleConfig {
    fn default() -> Self {
        Self {
            samples: vk::SampleCountFlags::TYPE_1,
            sample_shading: None,
            sample_mask: None,
        }
    }
}

/// Dynamic state configuration
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct DynamicState {
    pub states: Vec<vk::DynamicState>,
}

/// Complete pipeline state configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineStateConfig {
    pub blend_mode: BlendMode,
    pub depth_config: DepthConfig,
    pub vertex_config: VertexConfig,
    pub rasterization: RasterizationConfig,
    pub multisample: MultisampleConfig,
    pub dynamic_state: DynamicState,
}

impl Default for PipelineStateConfig {
    fn default() -> Self {
        Self {
            blend_mode: BlendMode::default(),
            depth_config: DepthConfig::default(),
            vertex_config: VertexConfig::default(),
            rasterization: RasterizationConfig::default(),
            multisample: MultisampleConfig::default(),
            dynamic_state: DynamicState::default(),
        }
    }
}
