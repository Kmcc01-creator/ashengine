use super::PhysicsError;
use ash::{self, vk};
use std::collections::VecDeque;
use std::sync::Arc;

const MIN_POOL_SIZE: u64 = 1024 * 1024; // 1MB minimum pool size
const MAX_POOL_SIZE: u64 = 256 * 1024 * 1024; // 256MB maximum pool size

pub struct MemoryBlock {
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    offset: vk::DeviceSize,
    is_free: bool,
}

pub struct MemoryPool {
    device: Arc<ash::Device>,
    memory_type_index: u32,
    blocks: Vec<MemoryBlock>,
    free_blocks: VecDeque<usize>,
    total_size: vk::DeviceSize,
    used_size: vk::DeviceSize,
    block_size: vk::DeviceSize,
}

impl MemoryPool {
    pub fn new(
        device: Arc<ash::Device>,
        memory_type_index: u32,
        initial_size: vk::DeviceSize,
    ) -> Result<Self, PhysicsError> {
        let block_size = initial_size.max(MIN_POOL_SIZE).min(MAX_POOL_SIZE);

        Ok(Self {
            device,
            memory_type_index,
            blocks: Vec::new(),
            free_blocks: VecDeque::new(),
            total_size: 0,
            used_size: 0,
            block_size,
        })
    }

    pub fn allocate(
        &mut self,
        size: vk::DeviceSize,
        alignment: vk::DeviceSize,
    ) -> Result<(vk::DeviceMemory, vk::DeviceSize), PhysicsError> {
        // Try to find a free block that fits
        if let Some(block_index) = self.find_free_block(size, alignment) {
            let block = &mut self.blocks[block_index];
            block.is_free = false;
            self.used_size += size;
            return Ok((block.memory, block.offset));
        }

        // Need to allocate a new block
        let new_block_size = size.max(self.block_size);
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(new_block_size)
            .memory_type_index(self.memory_type_index)
            .build();

        let memory = unsafe {
            self.device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| {
                    PhysicsError::OutOfMemory(format!("Failed to allocate memory block: {}", e))
                })?
        };

        let block = MemoryBlock {
            memory,
            size: new_block_size,
            offset: 0,
            is_free: false,
        };

        self.blocks.push(block);
        self.total_size += new_block_size;
        self.used_size += size;

        Ok((memory, 0))
    }

    pub fn free(&mut self, memory: vk::DeviceMemory, offset: vk::DeviceSize) {
        if let Some(index) = self
            .blocks
            .iter()
            .position(|block| block.memory == memory && block.offset == offset)
        {
            self.blocks[index].is_free = true;
            self.free_blocks.push_back(index);

            // Coalesce adjacent free blocks
            self.coalesce_free_blocks();
        }
    }

    pub fn cleanup(&mut self) {
        unsafe {
            for block in &self.blocks {
                self.device.free_memory(block.memory, None);
            }
        }
        self.blocks.clear();
        self.free_blocks.clear();
        self.total_size = 0;
        self.used_size = 0;
    }

    fn find_free_block(&self, size: vk::DeviceSize, alignment: vk::DeviceSize) -> Option<usize> {
        for &index in &self.free_blocks {
            let block = &self.blocks[index];
            if block.is_free && block.size >= size {
                // Check alignment
                let aligned_offset = (block.offset + alignment - 1) & !(alignment - 1);
                if aligned_offset + size <= block.offset + block.size {
                    return Some(index);
                }
            }
        }
        None
    }

    fn coalesce_free_blocks(&mut self) {
        let mut i = 0;
        while i < self.blocks.len() - 1 {
            if self.blocks[i].is_free && self.blocks[i + 1].is_free {
                // Merge blocks
                let next_block = self.blocks.remove(i + 1);
                let current_block = &mut self.blocks[i];
                current_block.size += next_block.size;

                // Update free blocks list
                if let Some(pos) = self.free_blocks.iter().position(|&idx| idx == i + 1) {
                    self.free_blocks.remove(pos);
                }
            } else {
                i += 1;
            }
        }
    }

    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            total_size: self.total_size,
            used_size: self.used_size,
            free_size: self.total_size - self.used_size,
            block_count: self.blocks.len() as u32,
            free_block_count: self.free_blocks.len() as u32,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    pub total_size: vk::DeviceSize,
    pub used_size: vk::DeviceSize,
    pub free_size: vk::DeviceSize,
    pub block_count: u32,
    pub free_block_count: u32,
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[derive(Debug)]
pub struct BufferPool {
    device: Arc<ash::Device>,
    memory_pool: MemoryPool,
    buffers: Vec<vk::Buffer>,
}

impl BufferPool {
    pub fn new(
        device: Arc<ash::Device>,
        memory_type_index: u32,
        initial_size: vk::DeviceSize,
    ) -> Result<Self, PhysicsError> {
        Ok(Self {
            device: device.clone(),
            memory_pool: MemoryPool::new(device, memory_type_index, initial_size)?,
            buffers: Vec::new(),
        })
    }

    pub fn allocate_buffer(
        &mut self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory, vk::DeviceSize), PhysicsError> {
        // Create buffer
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .build();

        let buffer = unsafe {
            self.device.create_buffer(&buffer_info, None).map_err(|e| {
                PhysicsError::InitializationFailed(format!("Failed to create buffer: {}", e))
            })?
        };

        // Get memory requirements
        let mem_requirements = unsafe { self.device.get_buffer_memory_requirements(buffer) };

        // Allocate memory from pool
        let (memory, offset) = self
            .memory_pool
            .allocate(mem_requirements.size, mem_requirements.alignment)?;

        // Bind buffer to memory
        unsafe {
            self.device
                .bind_buffer_memory(buffer, memory, offset)
                .map_err(|e| {
                    PhysicsError::InitializationFailed(format!(
                        "Failed to bind buffer memory: {}",
                        e
                    ))
                })?;
        }

        self.buffers.push(buffer);
        Ok((buffer, memory, offset))
    }

    pub fn free_buffer(
        &mut self,
        buffer: vk::Buffer,
        memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
    ) {
        unsafe {
            self.device.destroy_buffer(buffer, None);
        }
        self.memory_pool.free(memory, offset);
        if let Some(pos) = self.buffers.iter().position(|&b| b == buffer) {
            self.buffers.swap_remove(pos);
        }
    }

    pub fn cleanup(&mut self) {
        unsafe {
            for buffer in &self.buffers {
                self.device.destroy_buffer(*buffer, None);
            }
        }
        self.buffers.clear();
        self.memory_pool.cleanup();
    }

    pub fn get_memory_stats(&self) -> MemoryStats {
        self.memory_pool.get_stats()
    }
}

impl Drop for BufferPool {
    fn drop(&mut self) {
        self.cleanup();
    }
}
