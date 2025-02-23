//! Render pass management and configuration
//!
//! Provides abstractions for creating and managing different types of render passes,
//! particularly focused on deferred rendering pipeline passes.

use super::{
    graph::{AttachmentDesc, AttachmentType, PassDesc},
    pipeline::{DepthConfig, PipelineBuilder, RasterizationConfig},
};
use crate::{
    error::Result,
    graphics::resource::{ResourceHandle, ResourceManager, TextureFormat},
};
use ash::vk;
use std::sync::Arc;

/// Types of render passes supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassType {
    Geometry,
    Lighting,
    PostProcess,
    UI,
}

/// Configuration for a render pass
#[derive(Debug, Clone)]
pub struct PassConfig {
    pub pass_type: PassType,
    pub width: u32,
    pub height: u32,
    pub samples: vk::SampleCountFlags,
    pub color_formats: Vec<TextureFormat>,
    pub depth_format: Option<TextureFormat>,
    pub clear_colors: Vec<[f32; 4]>,
    pub clear_depth: Option<f32>,
}

/// Manager for creating and configuring render passes
pub struct PassManager {
    device: Arc<ash::Device>,
    resource_manager: Arc<ResourceManager>,
}

impl PassManager {
    /// Create a new pass manager
    pub fn new(device: Arc<ash::Device>, resource_manager: Arc<ResourceManager>) -> Self {
        Self {
            device,
            resource_manager,
        }
    }

    /// Create a geometry pass configuration for deferred rendering
    pub fn create_geometry_pass(
        &self,
        width: u32,
        height: u32,
        samples: vk::SampleCountFlags,
    ) -> PassConfig {
        PassConfig {
            pass_type: PassType::Geometry,
            width,
            height,
            samples,
            color_formats: vec![
                TextureFormat::R32G32B32A32_SFLOAT, // Position
                TextureFormat::R16G16B16A16_SFLOAT, // Normal
                TextureFormat::R8G8B8A8_UNORM,      // Albedo + Metallic
            ],
            depth_format: Some(TextureFormat::D32_SFLOAT),
            clear_colors: vec![
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            clear_depth: Some(1.0),
        }
    }

    /// Create a lighting pass configuration for deferred rendering
    pub fn create_lighting_pass(
        &self,
        width: u32,
        height: u32,
        samples: vk::SampleCountFlags,
    ) -> PassConfig {
        PassConfig {
            pass_type: PassType::Lighting,
            width,
            height,
            samples,
            color_formats: vec![TextureFormat::R8G8B8A8_UNORM],
            depth_format: None,
            clear_colors: vec![[0.0, 0.0, 0.0, 1.0]],
            clear_depth: None,
        }
    }

    /// Convert a pass configuration into a pass descriptor
    pub fn create_pass_desc(&self, config: &PassConfig) -> PassDesc {
        let mut attachments = Vec::new();
        let mut color_attachments = Vec::new();
        let mut depth_attachment = None;

        // Add color attachments
        for (i, (format, clear_color)) in config
            .color_formats
            .iter()
            .zip(config.clear_colors.iter())
            .enumerate()
        {
            attachments.push(AttachmentDesc {
                ty: AttachmentType::Color {
                    format: *format,
                    clear: true,
                },
                samples: config.samples,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: match config.pass_type {
                    PassType::Geometry => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    PassType::Lighting => vk::ImageLayout::PRESENT_SRC_KHR,
                    PassType::PostProcess => vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    PassType::UI => vk::ImageLayout::PRESENT_SRC_KHR,
                },
            });
            color_attachments.push(i);
        }

        // Add depth attachment if specified
        if let Some(format) = config.depth_format {
            let depth_index = attachments.len();
            attachments.push(AttachmentDesc {
                ty: AttachmentType::Depth {
                    format,
                    clear: config.clear_depth.is_some(),
                },
                samples: config.samples,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
            });
            depth_attachment = Some(depth_index);
        }

        PassDesc {
            name: format!("{:?}Pass", config.pass_type),
            attachments,
            color_attachments,
            depth_attachment,
            input_attachments: Vec::new(), // Set by render graph based on dependencies
            dependencies: std::collections::HashSet::new(),
        }
    }

    /// Create a default pipeline configuration for a pass type
    pub fn create_default_pipeline(&self, pass_type: PassType) -> PipelineBuilder {
        let mut builder = PipelineBuilder::new(self.device.clone(), self.resource_manager.clone());

        match pass_type {
            PassType::Geometry => {
                builder = builder
                    .rasterization(RasterizationConfig {
                        cull_mode: vk::CullModeFlags::BACK,
                        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
                        ..Default::default()
                    })
                    .depth(DepthConfig {
                        test_enable: true,
                        write_enable: true,
                        compare_op: vk::CompareOp::LESS,
                    })
                    .blend(false);
            }
            PassType::Lighting => {
                builder = builder
                    .rasterization(RasterizationConfig {
                        cull_mode: vk::CullModeFlags::NONE,
                        ..Default::default()
                    })
                    .depth(DepthConfig {
                        test_enable: false,
                        write_enable: false,
                        compare_op: vk::CompareOp::ALWAYS,
                    })
                    .blend(true);
            }
            PassType::PostProcess => {
                builder = builder
                    .rasterization(RasterizationConfig {
                        cull_mode: vk::CullModeFlags::NONE,
                        ..Default::default()
                    })
                    .depth(DepthConfig {
                        test_enable: false,
                        write_enable: false,
                        compare_op: vk::CompareOp::ALWAYS,
                    })
                    .blend(true);
            }
            PassType::UI => {
                builder = builder
                    .rasterization(RasterizationConfig {
                        cull_mode: vk::CullModeFlags::NONE,
                        ..Default::default()
                    })
                    .depth(DepthConfig {
                        test_enable: false,
                        write_enable: false,
                        compare_op: vk::CompareOp::ALWAYS,
                    })
                    .blend(true);
            }
        }

        builder
    }
}
