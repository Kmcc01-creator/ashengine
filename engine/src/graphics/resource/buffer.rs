//! Buffer management and types
//!
//! Provides buffer implementations including persistently mapped buffers
//! for efficient updates of frequently changing data like transforms.

use ash::vk;
use parking_lot::RwLock;
use std::sync::Arc;

use crate::error::Result;

/// Types of buffers that can be managed
#[derive(Debug, Clone, Copy)]
pub enum BufferType {
    Vertex,
    Index,
    Uniform,
    Storage,
    TransformStorage, // New type for transform data
}

/// Configuration for buffer creation
#[derive(Debug, Clone)]
pub struct BufferConfig {
    pub size: vk::DeviceSize,
    pub usage: vk::BufferUsageFlags,
    pub memory_flags: vk::MemoryPropertyFlags,
    pub buffer_type: BufferType,
}

/// A persistently mapped buffer for efficient updates
pub struct MappedBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    mapped_ptr: *mut u8,
    size: vk::DeviceSize,
    device: Arc<ash::Device>,
    // Optional ring buffer tracking
    ring_offset: RwLock<vk::DeviceSize>,
    ring_size: vk::DeviceSize,
}

unsafe impl Send for MappedBuffer {}
unsafe impl Sync for MappedBuffer {}

impl MappedBuffer {
    /// Create a new mapped buffer
    pub fn new(
        device: Arc<ash::Device>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        ring_buffer: bool,
    ) -> Result<Self> {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        // Request host visible and coherent memory
        let memory_flags =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        // TODO: Proper memory type selection
        let memory_type_index = 0;

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index)
            .build();

        let memory = unsafe { device.allocate_memory(&alloc_info, None)? };

        unsafe {
            device.bind_buffer_memory(buffer, memory, 0)?;
        }

        // Persistently map the memory
        let mapped_ptr =
            unsafe { device.map_memory(memory, 0, size, vk::MemoryMapFlags::empty())? as *mut u8 };

        Ok(Self {
            buffer,
            memory,
            mapped_ptr,
            size,
            device,
            ring_offset: RwLock::new(0),
            ring_size: if ring_buffer { size } else { 0 },
        })
    }

    /// Write data to the buffer at the specified offset
    pub fn write_data(&self, data: &[u8], offset: vk::DeviceSize) {
        assert!(offset + data.len() as u64 <= self.size);
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                self.mapped_ptr.add(offset as usize),
                data.len(),
            );
        }
    }

    /// Allocate space in the ring buffer, returning the offset
    pub fn ring_allocate(&self, size: vk::DeviceSize) -> Option<vk::DeviceSize> {
        if self.ring_size == 0 {
            return None;
        }

        let mut offset = self.ring_offset.write();
        let new_offset = *offset + size;

        if new_offset > self.ring_size {
            *offset = 0;
            Some(0)
        } else {
            let current = *offset;
            *offset = new_offset;
            Some(current)
        }
    }

    /// Get the underlying buffer handle
    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }
}

impl Drop for MappedBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.unmap_memory(self.memory);
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}
