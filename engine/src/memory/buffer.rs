use ash::vk;
use log::debug;
use std::sync::Arc;

use super::{MemoryAllocator, MemoryBlock};
use crate::context::Context;
use crate::error::{Result, VulkanError};

pub struct Buffer {
    buffer: vk::Buffer,
    memory_block: MemoryBlock,
    size: u64,
    context: Arc<Context>,
}

impl Buffer {
    pub fn new(
        context: Arc<Context>,
        allocator: &MemoryAllocator,
        size: u64,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        let device = context.device();

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe {
            device
                .create_buffer(&buffer_info, None)
                .map_err(|e| VulkanError::BufferCreation(e.to_string()))?
        };

        let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory_block = allocator.allocate(size, memory_requirements, properties)?;

        unsafe {
            device
                .bind_buffer_memory(buffer, memory_block.memory, memory_block.offset)
                .map_err(|e| VulkanError::MemoryBinding(e.to_string()))?;
        }

        debug!(
            "Created buffer: size={}, usage={:?}, memory_type={}",
            size, usage, memory_block.memory_type_index
        );

        Ok(Self {
            buffer,
            memory_block,
            size,
            context,
        })
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn memory(&self) -> vk::DeviceMemory {
        self.memory_block.memory
    }

    pub fn memory_offset(&self) -> u64 {
        self.memory_block.offset
    }

    pub fn map<T>(&self) -> Result<BufferView<T>> {
        if self.size == 0 {
            return Err(VulkanError::MemoryMapping(
                "Cannot map empty buffer".to_string(),
            ));
        }

        let ptr = unsafe {
            self.context
                .device()
                .map_memory(
                    self.memory_block.memory,
                    self.memory_block.offset,
                    self.size,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| VulkanError::MemoryMapping(e.to_string()))?
        };

        Ok(BufferView {
            ptr: ptr as *mut T,
            len: (self.size / std::mem::size_of::<T>() as u64) as usize,
            buffer: self,
        })
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.context.device().destroy_buffer(self.buffer, None);
        }
    }
}

pub struct BufferView<'a, T> {
    ptr: *mut T,
    len: usize,
    buffer: &'a Buffer,
}

impl<'a, T> BufferView<'a, T> {
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl<'a, T> Drop for BufferView<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.buffer
                .context
                .device()
                .unmap_memory(self.buffer.memory_block.memory);
        }
    }
}

unsafe impl<'a, T: Send> Send for BufferView<'a, T> {}
unsafe impl<'a, T: Sync> Sync for BufferView<'a, T> {}
