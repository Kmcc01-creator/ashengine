mod buffer;
mod error;
mod logging;

use ash::vk;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::context::Context;
pub use buffer::Buffer;
pub use error::{MemoryError, Result};
use logging::{MemoryLogStats, MemoryLogger};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryBlock {
    pub memory: vk::DeviceMemory,
    pub offset: u64,
    pub size: u64,
    pub memory_type_index: u32,
}

#[derive(Debug)]
struct MemoryChunk {
    memory: vk::DeviceMemory,
    size: u64,
    free_regions: Vec<(u64, u64)>, // (offset, size)
    memory_type_index: u32,
}

pub struct MemoryAllocator {
    context: Arc<Context>,
    chunks: Mutex<HashMap<u32, Vec<MemoryChunk>>>, // memory_type_index -> chunks
    logger: MemoryLogger,
}

impl MemoryAllocator {
    pub fn new(context: Arc<Context>) -> Self {
        info!("Initializing Memory Allocator");
        Self {
            context,
            chunks: Mutex::new(HashMap::new()),
            logger: MemoryLogger::new(),
        }
    }

    pub fn allocate(
        &self,
        size: u64,
        requirements: vk::MemoryRequirements,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<MemoryBlock> {
        let memory_type_index = self
            .find_memory_type_index(requirements.memory_type_bits, properties)
            .map_err(|_| MemoryError::UnsupportedMemoryType(requirements.memory_type_bits))?;

        // Round up size to alignment
        let aligned_size =
            ((size + requirements.alignment - 1) / requirements.alignment) * requirements.alignment;

        let mut chunks = self.chunks.lock().unwrap();
        let chunk_list = chunks.entry(memory_type_index).or_insert_with(Vec::new);

        // Try to find space in existing chunks
        for chunk in chunk_list.iter_mut() {
            if let Some((offset, _)) = chunk
                .free_regions
                .iter()
                .find(|(_, region_size)| *region_size >= aligned_size)
                .cloned()
            {
                // Remove or shrink the free region
                chunk
                    .free_regions
                    .retain(|&(region_offset, _)| region_offset != offset);
                if aligned_size < requirements.size {
                    chunk
                        .free_regions
                        .push((offset + aligned_size, requirements.size - aligned_size));
                }

                self.logger.log_allocation(aligned_size, memory_type_index);

                return Ok(MemoryBlock {
                    memory: chunk.memory,
                    offset,
                    size: aligned_size,
                    memory_type_index,
                });
            }
        }

        // Create new chunk if no suitable space found
        let chunk_size = aligned_size.max(64 * 1024 * 1024); // Minimum 64MB chunks
        match self.create_chunk(chunk_size, memory_type_index) {
            Ok(new_chunk) => {
                let memory = new_chunk.memory;

                // Add remaining space as a free region
                if chunk_size > aligned_size {
                    new_chunk
                        .free_regions
                        .push((aligned_size, chunk_size - aligned_size));
                }

                chunk_list.push(new_chunk);
                self.logger.log_allocation(aligned_size, memory_type_index);

                Ok(MemoryBlock {
                    memory,
                    offset: 0,
                    size: aligned_size,
                    memory_type_index,
                })
            }
            Err(e) => {
                self.logger
                    .log_error(&format!("Failed to create memory chunk: {}", e));
                Err(MemoryError::AllocationFailed(e.to_string()))
            }
        }
    }

    pub fn free(&self, block: MemoryBlock) -> Result<()> {
        let mut chunks = self.chunks.lock().unwrap();
        if let Some(chunk_list) = chunks.get_mut(&block.memory_type_index) {
            for chunk in chunk_list.iter_mut() {
                if chunk.memory == block.memory {
                    // Add freed region back to free list
                    chunk.free_regions.push((block.offset, block.size));
                    // Merge adjacent free regions
                    chunk.free_regions.sort_by_key(|&(offset, _)| offset);
                    let mut i = 0;
                    while i < chunk.free_regions.len() - 1 {
                        let current = chunk.free_regions[i];
                        let next = chunk.free_regions[i + 1];
                        if current.0 + current.1 == next.0 {
                            chunk.free_regions[i].1 += next.1;
                            chunk.free_regions.remove(i + 1);
                        } else {
                            i += 1;
                        }
                    }

                    self.logger
                        .log_deallocation(block.size, block.memory_type_index);
                    return Ok(());
                }
            }
        }

        Err(MemoryError::InvalidOperation(
            "Failed to find memory block for freeing".to_string(),
        ))
    }

    fn create_chunk(&self, size: u64, memory_type_index: u32) -> Result<MemoryChunk> {
        let device = self.context.device();
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(size)
            .memory_type_index(memory_type_index)
            .build();

        let memory = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| MemoryError::AllocationFailed(e.to_string()))?
        };

        debug!(
            "Created new memory chunk: size={}, type={}",
            size, memory_type_index
        );

        Ok(MemoryChunk {
            memory,
            size,
            free_regions: Vec::new(),
            memory_type_index,
        })
    }

    fn find_memory_type_index(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32> {
        let memory_properties = unsafe {
            self.context
                .instance()
                .get_physical_device_memory_properties(self.context.physical_device())
        };

        for i in 0..memory_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0
                && memory_properties.memory_types[i as usize]
                    .property_flags
                    .contains(properties)
            {
                return Ok(i);
            }
        }

        Err(MemoryError::UnsupportedMemoryType(type_filter))
    }

    pub fn get_stats(&self) -> MemoryLogStats {
        self.logger.get_stats()
    }

    pub fn print_memory_stats(&self) {
        self.logger.print_summary();
    }
}

impl Drop for MemoryAllocator {
    fn drop(&mut self) {
        let chunks = self.chunks.lock().unwrap();
        let device = self.context.device();

        for chunk_list in chunks.values() {
            for chunk in chunk_list {
                if !chunk.free_regions.is_empty() {
                    let unfreed_size =
                        chunk.size - chunk.free_regions.iter().map(|(_, size)| size).sum::<u64>();
                    if unfreed_size > 0 {
                        self.logger.warn_leak(unfreed_size, chunk.memory_type_index);
                    }
                }
                unsafe {
                    device.free_memory(chunk.memory, None);
                }
            }
        }

        self.logger.print_summary();
    }
}
