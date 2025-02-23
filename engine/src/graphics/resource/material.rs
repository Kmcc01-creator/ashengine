//! Material system for managing shader parameters and textures
//!
//! Provides a flexible material system that can be used with the ECS

use super::{ResourceHandle, ResourceManager};
use ash::vk;
use std::collections::HashMap;
use std::sync::Arc;

/// Material parameter types that can be passed to shaders
#[derive(Debug, Clone)]
pub enum MaterialParam {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Int(i32),
    UInt(u32),
    Bool(bool),
    TextureHandle(ResourceHandle),
}

/// Descriptor for creating a new material
#[derive(Debug, Clone)]
pub struct MaterialDescriptor {
    /// Name of the material for debugging
    pub name: String,
    /// Shader to use for this material
    pub shader_handle: ResourceHandle,
    /// Initial parameter values
    pub parameters: HashMap<String, MaterialParam>,
}

/// A material instance that can be used for rendering
pub struct Material {
    descriptor: MaterialDescriptor,
    uniform_buffer: ResourceHandle,
    descriptor_set: vk::DescriptorSet,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl Material {
    /// Create a new material from a descriptor
    pub fn new(
        resource_manager: &ResourceManager,
        device: &ash::Device,
        descriptor: MaterialDescriptor,
    ) -> crate::error::Result<Self> {
        // Create descriptor set layout
        let bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::ALL)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(
                    descriptor
                        .parameters
                        .values()
                        .filter(|p| matches!(p, MaterialParam::TextureHandle(_)))
                        .count() as u32,
                )
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&layout_info, None)
                .map_err(|e| {
                    crate::error::VulkanError::DescriptorSetLayoutCreation(e.to_string())
                })?
        };

        // Create uniform buffer
        let uniform_buffer = resource_manager.create_buffer(
            1024, // TODO: Calculate actual size needed
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            super::BufferType::Uniform,
        )?;

        // Create descriptor pool
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 16, // Max textures per material
            },
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(1);

        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(&pool_info, None)
                .map_err(|e| crate::error::VulkanError::DescriptorPoolCreation(e.to_string()))?
        };

        // Allocate descriptor set
        let layouts = [descriptor_set_layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_set = unsafe {
            device
                .allocate_descriptor_sets(&alloc_info)
                .map_err(|e| crate::error::VulkanError::DescriptorSetAllocation(e.to_string()))?[0]
        };

        Ok(Self {
            descriptor,
            uniform_buffer,
            descriptor_set,
            descriptor_pool,
            descriptor_set_layout,
        })
    }

    /// Update material parameters
    pub fn set_parameter(&mut self, name: &str, value: MaterialParam) {
        self.descriptor.parameters.insert(name.to_string(), value);
        // TODO: Update uniform buffer and descriptor sets
    }

    /// Get the descriptor set for this material
    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }

    /// Get the descriptor set layout
    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        // Cleanup Vulkan resources
        // Note: ResourceManager will clean up the uniform buffer
    }
}
