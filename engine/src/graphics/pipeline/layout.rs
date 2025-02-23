//! Pipeline layout and descriptor set management
//!
//! Provides abstractions for managing descriptor set layouts and pipeline layouts
//! with efficient caching and reuse.

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
};

use ash::vk;

use crate::error::Result;

/// Type of resource binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingType {
    UniformBuffer,
    StorageBuffer,
    CombinedImageSampler,
    StorageImage,
    UniformTexelBuffer,
    StorageTexelBuffer,
}

impl BindingType {
    fn to_vk_descriptor_type(&self) -> vk::DescriptorType {
        match self {
            BindingType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
            BindingType::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            BindingType::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            BindingType::StorageImage => vk::DescriptorType::STORAGE_IMAGE,
            BindingType::UniformTexelBuffer => vk::DescriptorType::UNIFORM_TEXEL_BUFFER,
            BindingType::StorageTexelBuffer => vk::DescriptorType::STORAGE_TEXEL_BUFFER,
        }
    }
}

/// Description of a binding in a descriptor set
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindingDesc {
    pub binding: u32,
    pub ty: BindingType,
    pub count: u32,
    pub stages: vk::ShaderStageFlags,
}

/// Layout for a group of bindings
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindGroupLayout {
    pub bindings: Vec<BindingDesc>,
}

impl BindGroupLayout {
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn add_binding(mut self, binding: BindingDesc) -> Self {
        self.bindings.push(binding);
        self
    }

    fn create_descriptor_set_layout(
        &self,
        device: &ash::Device,
    ) -> Result<vk::DescriptorSetLayout> {
        let bindings: Vec<_> = self
            .bindings
            .iter()
            .map(|binding| {
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(binding.binding)
                    .descriptor_type(binding.ty.to_vk_descriptor_type())
                    .descriptor_count(binding.count)
                    .stage_flags(binding.stages)
                    .build()
            })
            .collect();

        let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

        unsafe {
            device
                .create_descriptor_set_layout(&create_info, None)
                .map_err(|e| crate::error::VulkanError::DescriptorSetLayoutCreation(e.to_string()))
        }
    }
}

/// Cache for descriptor set layouts
pub struct DescriptorSetLayoutCache {
    device: Arc<ash::Device>,
    layouts: HashMap<BindGroupLayout, vk::DescriptorSetLayout>,
}

impl DescriptorSetLayoutCache {
    pub fn new(device: Arc<ash::Device>) -> Self {
        Self {
            device,
            layouts: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, layout: &BindGroupLayout) -> Result<vk::DescriptorSetLayout> {
        if let Some(&descriptor_set_layout) = self.layouts.get(layout) {
            return Ok(descriptor_set_layout);
        }

        let descriptor_set_layout = layout.create_descriptor_set_layout(&self.device)?;
        self.layouts.insert(layout.clone(), descriptor_set_layout);
        Ok(descriptor_set_layout)
    }
}

impl Drop for DescriptorSetLayoutCache {
    fn drop(&mut self) {
        for layout in self.layouts.values() {
            unsafe {
                self.device.destroy_descriptor_set_layout(*layout, None);
            }
        }
    }
}

/// Description of push constant range
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PushConstantRange {
    pub stages: vk::ShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}

/// Description of a pipeline layout
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineLayoutDesc {
    pub bind_group_layouts: Vec<BindGroupLayout>,
    pub push_constant_ranges: Vec<PushConstantRange>,
}

/// Cache for pipeline layouts
pub struct PipelineLayoutCache {
    device: Arc<ash::Device>,
    descriptor_layout_cache: DescriptorSetLayoutCache,
    layouts: HashMap<PipelineLayoutDesc, vk::PipelineLayout>,
}

impl PipelineLayoutCache {
    pub fn new(device: Arc<ash::Device>) -> Self {
        Self {
            device: device.clone(),
            descriptor_layout_cache: DescriptorSetLayoutCache::new(device),
            layouts: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, desc: &PipelineLayoutDesc) -> Result<vk::PipelineLayout> {
        if let Some(&pipeline_layout) = self.layouts.get(desc) {
            return Ok(pipeline_layout);
        }

        // Create descriptor set layouts
        let descriptor_set_layouts: Result<Vec<_>> = desc
            .bind_group_layouts
            .iter()
            .map(|layout| self.descriptor_layout_cache.get_or_create(layout))
            .collect();

        let descriptor_set_layouts = descriptor_set_layouts?;

        // Create push constant ranges
        let push_constant_ranges: Vec<_> = desc
            .push_constant_ranges
            .iter()
            .map(|range| {
                vk::PushConstantRange::builder()
                    .stage_flags(range.stages)
                    .offset(range.offset)
                    .size(range.size)
                    .build()
            })
            .collect();

        // Create pipeline layout
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        let pipeline_layout = unsafe {
            self.device
                .create_pipeline_layout(&create_info, None)
                .map_err(|e| crate::error::VulkanError::PipelineLayoutCreation(e.to_string()))?
        };

        self.layouts.insert(desc.clone(), pipeline_layout);
        Ok(pipeline_layout)
    }
}

impl Drop for PipelineLayoutCache {
    fn drop(&mut self) {
        for layout in self.layouts.values() {
            unsafe {
                self.device.destroy_pipeline_layout(*layout, None);
            }
        }
    }
}
