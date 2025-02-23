//! Pipeline variants and specialization
//!
//! Provides support for pipeline variants through specialization constants
//! and state combinations

use ash::vk;
use std::hash::{Hash, Hasher};

use super::config::PipelineStateConfig;
use crate::graphics::resource::ResourceHandle;

/// Key identifying a base pipeline configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub shaders: Vec<(vk::ShaderStageFlags, ResourceHandle)>,
    pub render_pass: ResourceHandle,
    pub subpass: u32,
}

/// Specialization constant value
#[derive(Debug, Clone, PartialEq)]
pub enum SpecConstantValue {
    Bool(bool),
    Int32(i32),
    Int64(i64),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
}

impl Hash for SpecConstantValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            SpecConstantValue::Bool(v) => {
                0u8.hash(state);
                v.hash(state);
            }
            SpecConstantValue::Int32(v) => {
                1u8.hash(state);
                v.hash(state);
            }
            SpecConstantValue::Int64(v) => {
                2u8.hash(state);
                v.hash(state);
            }
            SpecConstantValue::UInt32(v) => {
                3u8.hash(state);
                v.hash(state);
            }
            SpecConstantValue::UInt64(v) => {
                4u8.hash(state);
                v.hash(state);
            }
            SpecConstantValue::Float32(v) => {
                5u8.hash(state);
                v.to_bits().hash(state);
            }
            SpecConstantValue::Float64(v) => {
                6u8.hash(state);
                v.to_bits().hash(state);
            }
        }
    }
}

impl Eq for SpecConstantValue {}

/// Information for specializing a shader
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecializationInfo {
    pub constants: Vec<(u32, SpecConstantValue)>,
    pub stages: vk::ShaderStageFlags,
}

impl SpecializationInfo {
    /// Create vulkan specialization info
    pub fn create_info(
        &self,
    ) -> (
        vk::SpecializationInfo,
        Vec<u8>,
        Vec<vk::SpecializationMapEntry>,
    ) {
        let mut data = Vec::new();
        let mut map_entries = Vec::new();
        let mut offset = 0;

        for (constant_id, value) in &self.constants {
            let (bytes, size) = match value {
                SpecConstantValue::Bool(v) => ((*v as u32).to_ne_bytes().to_vec(), 4),
                SpecConstantValue::Int32(v) => (v.to_ne_bytes().to_vec(), 4),
                SpecConstantValue::Int64(v) => (v.to_ne_bytes().to_vec(), 8),
                SpecConstantValue::UInt32(v) => (v.to_ne_bytes().to_vec(), 4),
                SpecConstantValue::UInt64(v) => (v.to_ne_bytes().to_vec(), 8),
                SpecConstantValue::Float32(v) => (v.to_bits().to_ne_bytes().to_vec(), 4),
                SpecConstantValue::Float64(v) => (v.to_bits().to_ne_bytes().to_vec(), 8),
            };

            map_entries.push(
                vk::SpecializationMapEntry::builder()
                    .constant_id(*constant_id)
                    .offset(offset)
                    .size(size)
                    .build(),
            );

            data.extend_from_slice(&bytes);
            offset += size as u32;
        }

        let info = vk::SpecializationInfo::builder()
            .map_entries(&map_entries)
            .data(&data)
            .build();

        (info, data, map_entries)
    }
}

/// Complete description of a pipeline variant
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineVariant {
    pub base: PipelineKey,
    pub state: PipelineStateConfig,
    pub specialization: Option<SpecializationInfo>,
}

impl PipelineVariant {
    pub fn new(base: PipelineKey) -> Self {
        Self {
            base,
            state: PipelineStateConfig::default(),
            specialization: None,
        }
    }

    pub fn with_state(mut self, state: PipelineStateConfig) -> Self {
        self.state = state;
        self
    }

    pub fn with_specialization(mut self, specialization: SpecializationInfo) -> Self {
        self.specialization = Some(specialization);
        self
    }
}

/// Cache for pipeline variants
pub struct VariantCache {
    variants: std::collections::HashMap<PipelineVariant, vk::Pipeline>,
}

impl VariantCache {
    pub fn new() -> Self {
        Self {
            variants: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, variant: &PipelineVariant) -> Option<vk::Pipeline> {
        self.variants.get(variant).copied()
    }

    pub fn insert(&mut self, variant: PipelineVariant, pipeline: vk::Pipeline) {
        self.variants.insert(variant, pipeline);
    }

    pub fn remove(&mut self, variant: &PipelineVariant) -> Option<vk::Pipeline> {
        self.variants.remove(variant)
    }
}
