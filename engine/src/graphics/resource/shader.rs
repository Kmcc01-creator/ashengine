//! Shader resource management
//!
//! Handles compilation, loading, and lifecycle of shader modules

use super::ResourceHandle;
use ash::vk;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Type of shader module
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessellationControl,
    TessellationEvaluation,
}

impl ShaderStage {
    fn to_vk_stage_flags(&self) -> vk::ShaderStageFlags {
        match self {
            ShaderStage::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => vk::ShaderStageFlags::FRAGMENT,
            ShaderStage::Compute => vk::ShaderStageFlags::COMPUTE,
            ShaderStage::Geometry => vk::ShaderStageFlags::GEOMETRY,
            ShaderStage::TessellationControl => vk::ShaderStageFlags::TESSELLATION_CONTROL,
            ShaderStage::TessellationEvaluation => vk::ShaderStageFlags::TESSELLATION_EVALUATION,
        }
    }
}

/// Description for creating a new shader module
#[derive(Debug, Clone)]
pub struct ShaderDescriptor {
    /// SPIR-V bytecode
    pub code: Vec<u32>,
    /// Stage this shader runs in
    pub stage: ShaderStage,
    /// Entry point function name
    pub entry_point: String,
}

/// A compiled shader module
pub struct ShaderModule {
    module: vk::ShaderModule,
    stage: ShaderStage,
    entry_point: String,
}

/// Manager for shader resources
pub struct ShaderManager {
    device: Arc<ash::Device>,
    shaders: RwLock<HashMap<ResourceHandle, ShaderModule>>,
}

impl ShaderManager {
    /// Create a new shader manager
    pub fn new(device: Arc<ash::Device>) -> Self {
        Self {
            device,
            shaders: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new shader module from SPIR-V bytecode
    pub fn create_shader(
        &self,
        descriptor: ShaderDescriptor,
    ) -> crate::error::Result<ResourceHandle> {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(&descriptor.code);

        let module = unsafe {
            self.device
                .create_shader_module(&create_info, None)
                .map_err(|e| crate::error::VulkanError::ShaderModuleCreation(e.to_string()))?
        };

        let shader = ShaderModule {
            module,
            stage: descriptor.stage,
            entry_point: descriptor.entry_point,
        };

        let handle = ResourceHandle::new();
        self.shaders.write().insert(handle, shader);

        Ok(handle)
    }

    /// Get shader stage info for pipeline creation
    pub fn get_stage_info(
        &self,
        handle: ResourceHandle,
    ) -> Option<vk::PipelineShaderStageCreateInfo> {
        self.shaders.read().get(&handle).map(|shader| {
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(shader.stage.to_vk_stage_flags())
                .module(shader.module)
                .name(shader.entry_point.as_bytes())
                .build()
        })
    }

    /// Get raw shader module
    pub fn get_module(&self, handle: ResourceHandle) -> Option<vk::ShaderModule> {
        self.shaders.read().get(&handle).map(|s| s.module)
    }
}

impl Drop for ShaderManager {
    fn drop(&mut self) {
        let shaders = self.shaders.get_mut();
        for shader in shaders.values() {
            unsafe {
                self.device.destroy_shader_module(shader.module, None);
            }
        }
    }
}

/// Utility functions for shader management
pub mod util {
    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    /// Load SPIR-V shader from a file
    pub fn load_spirv<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u32>> {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        Ok(bytes
            .chunks(4)
            .map(|chunk| {
                chunk
                    .iter()
                    .enumerate()
                    .fold(0u32, |acc, (i, &byte)| acc | ((byte as u32) << (i * 8)))
            })
            .collect())
    }
}
