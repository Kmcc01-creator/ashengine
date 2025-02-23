use crate::error::{Result, VulkanError};
use ash::{vk, Device};
use std::ffi::CStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

static MAIN_ENTRY_POINT: &CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

pub struct ShaderModule {
    device: Arc<Device>,
    module: vk::ShaderModule,
}

impl ShaderModule {
    pub fn from_file(device: Arc<Device>, spirv_path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(spirv_path).map_err(|e| {
            VulkanError::ShaderCreation(format!("Failed to open shader file: {}", e))
        })?;

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|e| {
            VulkanError::ShaderCreation(format!("Failed to read shader file: {}", e))
        })?;

        // Ensure the byte array length is a multiple of 4
        if bytes.len() % 4 != 0 {
            return Err(VulkanError::ShaderCreation(
                "Invalid SPIR-V format".to_string(),
            ));
        }

        let (prefix, words, suffix) = unsafe { bytes.align_to::<u32>() };
        if !prefix.is_empty() || !suffix.is_empty() {
            return Err(VulkanError::ShaderCreation(
                "Invalid SPIR-V alignment".to_string(),
            ));
        }

        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(words)
            .flags(vk::ShaderModuleCreateFlags::empty());

        let module = unsafe {
            device
                .create_shader_module(&create_info, None)
                .map_err(|e| VulkanError::ShaderCreation(e.to_string()))?
        };

        Ok(Self { device, module })
    }

    pub fn new(device: Arc<Device>, spirv_path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(spirv_path).map_err(|e| {
            VulkanError::ShaderCreation(format!("Failed to open shader file: {}", e))
        })?;

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|e| {
            VulkanError::ShaderCreation(format!("Failed to read shader file: {}", e))
        })?;

        // Ensure the byte array length is a multiple of 4
        if bytes.len() % 4 != 0 {
            return Err(VulkanError::ShaderCreation(
                "Invalid SPIR-V format".to_string(),
            ));
        }

        let (prefix, words, suffix) = unsafe { bytes.align_to::<u32>() };
        if !prefix.is_empty() || !suffix.is_empty() {
            return Err(VulkanError::ShaderCreation(
                "Invalid SPIR-V alignment".to_string(),
            ));
        }

        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(words)
            .flags(vk::ShaderModuleCreateFlags::empty());

        let module = unsafe {
            device
                .create_shader_module(&create_info, None)
                .map_err(|e| VulkanError::ShaderCreation(e.to_string()))?
        };

        Ok(Self { module, device })
    }

    pub fn create_shader_stage(
        &self,
        stage: vk::ShaderStageFlags,
    ) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(stage)
            .module(self.module)
            .name(MAIN_ENTRY_POINT)
            .build()
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(self.module, None);
        }
    }
}

pub struct ShaderSet {
    vertex: Option<ShaderModule>,
    fragment: Option<ShaderModule>,
    device: Arc<Device>,
}

impl ShaderSet {
    pub fn new(
        device: Arc<Device>,
        vert_path: impl AsRef<Path>,
        frag_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let vertex = ShaderModule::from_file(device.clone(), vert_path)?;
        let fragment = ShaderModule::from_file(device.clone(), frag_path)?;

        Ok(Self {
            vertex: Some(vertex),
            fragment: Some(fragment),
            device,
        })
    }

    pub fn create_shader_stages(&self) -> [vk::PipelineShaderStageCreateInfo; 2] {
        [
            self.vertex
                .as_ref()
                .unwrap()
                .create_shader_stage(vk::ShaderStageFlags::VERTEX),
            self.fragment
                .as_ref()
                .unwrap()
                .create_shader_stage(vk::ShaderStageFlags::FRAGMENT),
        ]
    }

    pub fn wait_idle(&self) -> Result<()> {
        unsafe {
            self.device
                .device_wait_idle()
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;
        }
        Ok(())
    }
}

impl Drop for ShaderSet {
    fn drop(&mut self) {
        if let Ok(_) = self.wait_idle() {
            self.vertex.take();
            self.fragment.take();
        }
    }
}
