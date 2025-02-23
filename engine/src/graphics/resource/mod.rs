//! Resource management system for graphics components
//!
//! Provides a centralized system for managing graphics resources including:
//! - Buffers (vertex, index, uniform)
//! - Textures
//! - Materials
//! - Shaders

mod buffer;
mod material;
mod mesh;
mod shader;
mod texture;

pub use buffer::{BufferType, MappedBuffer};
pub use material::{Material, MaterialDescriptor, MaterialParam};
pub use mesh::Mesh;
pub use shader::{ShaderDescriptor, ShaderManager, ShaderModule, ShaderStage};
pub use texture::{TextureDescriptor, TextureFormat, TextureManager};

use ash::vk;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::Result;

/// Unique identifier for a graphics resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle(u64);

impl ResourceHandle {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Types of graphics resources that can be managed
#[derive(Debug)]
pub enum ResourceType {
    Buffer(BufferType),
    MappedBuffer(BufferType),
    Texture,
    Material,
    Shader,
    Mesh,
}

/// Find suitable memory type for buffer allocation
fn find_memory_type(
    device: &ash::Device,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> Option<u32> {
    // TODO: Store physical device and memory properties in ResourceManager
    let mem_properties =
        unsafe { device.get_physical_device_memory_properties(vk::PhysicalDevice::null()) };

    for i in 0..mem_properties.memory_type_count {
        if (type_filter & (1 << i)) != 0
            && (mem_properties.memory_types[i as usize].property_flags & properties) == properties
        {
            return Some(i);
        }
    }
    None
}

/// Manager for mesh resources
pub struct MeshManager {
    device: Arc<ash::Device>,
    resource_manager: Arc<ResourceManager>,
}

impl MeshManager {
    /// Create a new mesh manager
    pub fn new(device: Arc<ash::Device>, resource_manager: Arc<ResourceManager>) -> Self {
        Self {
            device,
            resource_manager,
        }
    }

    /// Create a new mesh from vertex and index data.
    pub fn create_mesh(&self, vertices: &[Vertex], indices: &[u32]) -> Result<Mesh> {
        let vertex_buffer = self.resource_manager.create_buffer(
            (std::mem::size_of::<Vertex>() * vertices.len()) as vk::DeviceSize,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            BufferType::Vertex,
        )?;

        let index_buffer = self.resource_manager.create_buffer(
            (std::mem::size_of::<u32>() * indices.len()) as vk::DeviceSize,
            vk::BufferUsageFlags::INDEX_BUFFER,
            BufferType::Index,
        )?;

        // TODO: Actually upload data to buffers

        Ok(Mesh {
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
            vertex_buffer,
            index_buffer,
        })
    }

    /// Destroy a mesh and release its resources.
    pub fn destroy_mesh(&self, mesh: Mesh) -> Result<()> {
        self.resource_manager.destroy_resource(mesh.vertex_buffer);
        self.resource_manager.destroy_resource(mesh.index_buffer);
        Ok(())
    }
}

/// Central manager for all graphics resources
pub struct ResourceManager {
    device: Arc<ash::Device>,
    resources: RwLock<HashMap<ResourceHandle, ResourceType>>,
    buffers: RwLock<HashMap<ResourceHandle, vk::Buffer>>,
    buffer_memories: RwLock<HashMap<ResourceHandle, vk::DeviceMemory>>,
    texture_manager: TextureManager,
    shader_manager: ShaderManager,
    mesh_manager: Option<MeshManager>,
}

impl ResourceManager {
    /// Create a new resource manager.
    pub fn new(device: Arc<ash::Device>) -> Self {
        Self {
            device: device.clone(),
            resources: RwLock::new(HashMap::new()),
            buffers: RwLock::new(HashMap::new()),
            buffer_memories: RwLock::new(HashMap::new()),
            texture_manager: TextureManager::new(device.clone()),
            shader_manager: ShaderManager::new(device.clone()),
            mesh_manager: None,
        }
    }

    /// Create a new mapped buffer for efficient updates
    pub fn create_mapped_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        buffer_type: BufferType,
    ) -> Result<(ResourceHandle, *mut u8)> {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe {
            self.device
                .create_buffer(&buffer_info, None)
                .map_err(|e| crate::error::VulkanError::BufferCreation(e.to_string()))?
        };

        let mem_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

        // Request host visible and coherent memory
        let memory_flags =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        let memory_type_index = find_memory_type(
            &self.device,
            mem_requirements.memory_type_bits,
            memory_flags,
        )
        .ok_or_else(|| crate::error::VulkanError::NoSuitableMemory)?;

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index)
            .build();

        let memory = unsafe {
            self.device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| crate::error::VulkanError::MemoryAllocation(e.to_string()))?
        };

        unsafe {
            self.device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|e| crate::error::VulkanError::MemoryBinding(e.to_string()))?;
        }

        // Map the memory
        let mapped_ptr = unsafe {
            self.device
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
                .map_err(|e| crate::error::VulkanError::MemoryMapping(e.to_string()))?
        } as *mut u8;

        let handle = ResourceHandle::new();
        self.resources
            .write()
            .insert(handle, ResourceType::MappedBuffer(buffer_type));
        self.buffers.write().insert(handle, buffer);
        self.buffer_memories.write().insert(handle, memory);

        Ok((handle, mapped_ptr))
    }

    /// Create a new texture
    pub fn create_texture(&self, descriptor: TextureDescriptor) -> Result<ResourceHandle> {
        self.texture_manager.create_texture(descriptor)
    }

    /// Initialize mesh manager.
    pub fn init_mesh_manager(&mut self, mesh_manager: MeshManager) {
        self.mesh_manager = Some(mesh_manager);
    }

    /// Create a new buffer and return its handle
    pub fn create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        buffer_type: BufferType,
    ) -> Result<ResourceHandle> {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe {
            self.device
                .create_buffer(&buffer_info, None)
                .map_err(|e| crate::error::VulkanError::BufferCreation(e.to_string()))?
        };

        let mem_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

        // Use device local memory for non-mapped buffers
        let memory_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;

        let memory_type_index = find_memory_type(
            &self.device,
            mem_requirements.memory_type_bits,
            memory_flags,
        )
        .ok_or_else(|| crate::error::VulkanError::NoSuitableMemory)?;

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index)
            .build();

        let memory = unsafe {
            self.device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| crate::error::VulkanError::MemoryAllocation(e.to_string()))?
        };

        unsafe {
            self.device
                .bind_buffer_memory(buffer, memory, 0)
                .map_err(|e| crate::error::VulkanError::MemoryBinding(e.to_string()))?;
        }

        let handle = ResourceHandle::new();
        self.resources
            .write()
            .insert(handle, ResourceType::Buffer(buffer_type));
        self.buffers.write().insert(handle, buffer);
        self.buffer_memories.write().insert(handle, memory);

        Ok(handle)
    }

    /// Create a new shader module.
    pub fn create_shader(&self, descriptor: ShaderDescriptor) -> Result<ResourceHandle> {
        self.shader_manager.create_shader(descriptor)
    }

    /// Get a buffer handle if it exists
    pub fn get_buffer(&self, handle: ResourceHandle) -> Option<vk::Buffer> {
        self.buffers.read().get(&handle).copied()
    }

    /// Get texture information
    pub fn get_texture(&self, handle: ResourceHandle) -> Option<(vk::ImageView, vk::Sampler)> {
        self.texture_manager.get_texture(handle)
    }

    /// Get shader stage info for pipeline creation
    pub fn get_shader_stage_info(
        &self,
        handle: ResourceHandle,
    ) -> Option<vk::PipelineShaderStageCreateInfo> {
        self.shader_manager.get_stage_info(handle)
    }

    /// Destroy a resource
    pub fn destroy_resource(&self, handle: ResourceHandle) {
        if let Some(resource_type) = self.resources.write().remove(&handle) {
            match resource_type {
                ResourceType::Buffer(_) | ResourceType::MappedBuffer(_) => {
                    // Get buffer and memory handles
                    let buffer = self.buffers.write().remove(&handle);
                    let memory = self.buffer_memories.write().remove(&handle);

                    unsafe {
                        // For mapped buffers, unmap memory before cleanup
                        if let ResourceType::MappedBuffer(_) = resource_type {
                            if let Some(mem) = memory {
                                self.device.unmap_memory(mem);
                            }
                        }

                        // Cleanup buffer and memory
                        if let Some(buf) = buffer {
                            self.device.destroy_buffer(buf, None);
                        }
                        if let Some(mem) = memory {
                            self.device.free_memory(mem, None);
                        }
                    }
                }
                _ => {
                    // Handled by respective managers
                }
            }
        }
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        // Clean up all resources
        let handles: Vec<_> = self.resources.read().keys().copied().collect();
        for handle in handles {
            self.destroy_resource(handle);
        }
    }
}
