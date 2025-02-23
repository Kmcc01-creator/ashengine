//! Texture management system
//!
//! Handles creation, storage, and lifecycle of texture resources

use super::{ResourceHandle, ResourceManager};
use ash::vk;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Format of the texture data
#[derive(Debug, Clone, Copy)]
pub enum TextureFormat {
    R8G8B8A8Unorm,
    B8G8R8A8Unorm,
    R8G8B8Unorm,
    R8Unorm,
    Depth32Float,
}

impl TextureFormat {
    fn to_vk_format(&self) -> vk::Format {
        match self {
            TextureFormat::R8G8B8A8Unorm => vk::Format::R8G8B8A8_UNORM,
            TextureFormat::B8G8R8A8Unorm => vk::Format::B8G8R8A8_UNORM,
            TextureFormat::R8G8B8Unorm => vk::Format::R8G8B8_UNORM,
            TextureFormat::R8Unorm => vk::Format::R8_UNORM,
            TextureFormat::Depth32Float => vk::Format::D32_SFLOAT,
        }
    }
}

/// Description for creating a new texture
#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    /// Width of the texture in pixels
    pub width: u32,
    /// Height of the texture in pixels
    pub height: u32,
    /// Format of the texture data
    pub format: TextureFormat,
    /// Initial data to upload to the texture
    pub data: Option<Vec<u8>>,
    /// Usage flags for the texture
    pub usage: vk::ImageUsageFlags,
}

/// Managed texture resource
pub struct Texture {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    sampler: vk::Sampler,
    format: vk::Format,
    extent: vk::Extent3D,
}

/// Manager for texture resources
pub struct TextureManager {
    device: Arc<ash::Device>,
    textures: RwLock<HashMap<ResourceHandle, Texture>>,
}

impl TextureManager {
    /// Create a new texture manager
    pub fn new(device: Arc<ash::Device>) -> Self {
        Self {
            device,
            textures: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new texture from a descriptor
    pub fn create_texture(
        &self,
        descriptor: TextureDescriptor,
    ) -> crate::error::Result<ResourceHandle> {
        let format = descriptor.format.to_vk_format();

        // Create image
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width: descriptor.width,
                height: descriptor.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(descriptor.usage | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = unsafe {
            self.device
                .create_image(&image_info, None)
                .map_err(|e| crate::error::VulkanError::ImageCreation(e.to_string()))?
        };

        // Allocate and bind memory
        let memory_requirements = unsafe { self.device.get_image_memory_requirements(image) };

        // TODO: Implement proper memory type selection
        let memory_type_index = 0;

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe {
            self.device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| crate::error::VulkanError::MemoryAllocation(e.to_string()))?
        };

        unsafe {
            self.device
                .bind_image_memory(image, memory, 0)
                .map_err(|e| crate::error::VulkanError::MemoryBinding(e.to_string()))?;
        }

        // Create image view
        let view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: if format == vk::Format::D32_SFLOAT {
                    vk::ImageAspectFlags::DEPTH
                } else {
                    vk::ImageAspectFlags::COLOR
                },
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let view = unsafe {
            self.device
                .create_image_view(&view_info, None)
                .map_err(|e| crate::error::VulkanError::ImageViewCreation(e.to_string()))?
        };

        // Create sampler
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT);

        let sampler = unsafe {
            self.device
                .create_sampler(&sampler_info, None)
                .map_err(|e| crate::error::VulkanError::SamplerCreation(e.to_string()))?
        };

        let texture = Texture {
            image,
            memory,
            view,
            sampler,
            format,
            extent: vk::Extent3D {
                width: descriptor.width,
                height: descriptor.height,
                depth: 1,
            },
        };

        let handle = ResourceHandle::new();
        self.textures.write().insert(handle, texture);

        Ok(handle)
    }

    /// Get a texture by its handle
    pub fn get_texture(&self, handle: ResourceHandle) -> Option<(vk::ImageView, vk::Sampler)> {
        self.textures
            .read()
            .get(&handle)
            .map(|texture| (texture.view, texture.sampler))
    }
}

impl Drop for TextureManager {
    fn drop(&mut self) {
        let textures = self.textures.get_mut();
        for texture in textures.values() {
            unsafe {
                self.device.destroy_sampler(texture.sampler, None);
                self.device.destroy_image_view(texture.view, None);
                self.device.destroy_image(texture.image, None);
                self.device.free_memory(texture.memory, None);
            }
        }
    }
}
