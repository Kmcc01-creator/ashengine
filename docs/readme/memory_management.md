# Memory Management

This document describes memory management in AshEngine.

AshEngine uses a custom memory allocator (`MemoryAllocator`) to manage Vulkan memory allocation. It uses a chunking strategy to reduce fragmentation and improve performance.

## MemoryAllocator

The `MemoryAllocator` is responsible for allocating and freeing memory blocks.

### Creation

A `MemoryAllocator` is created using the `new` function:

```rust
pub fn new(context: Arc<Context>) -> Self { ... }
```

- `context`: The AshEngine `Context`.

### Allocation

Memory is allocated using the `allocate` function:

```rust
pub fn allocate(
    &self,
    size: u64,
    requirements: vk::MemoryRequirements,
    properties: vk::MemoryPropertyFlags,
) -> Result<MemoryBlock> { ... }
```

- `size`: The requested size of the allocation.
- `requirements`: Vulkan memory requirements (alignment, memory type bits).
- `properties`: Desired memory properties (e.g., host visible, device local).

The `allocate` function:

1.  Finds a suitable memory type index based on the requirements and properties.
2.  Rounds up the requested size to the required alignment.
3.  Tries to find a free region in existing memory chunks of the selected memory type.
4.  If a suitable free region is found:
    - Removes or shrinks the free region.
    - Returns a `MemoryBlock` representing the allocated memory.
5.  If no suitable free region is found:
    - Creates a new memory chunk (minimum size 64MB).
    - Adds the remaining space in the new chunk (if any) as a free region.
    - Returns a `MemoryBlock` for the allocated memory at the beginning of the new chunk.

### Deallocation

Memory is freed using the `free` function:

```rust
pub fn free(&self, block: MemoryBlock) -> Result<()> { ... }
```

- `block`: The `MemoryBlock` to free.

The `free` function:

1.  Finds the chunk that contains the `MemoryBlock`.
2.  Adds the freed region back to the chunk's free list.
3.  Merges adjacent free regions to reduce fragmentation.

### MemoryChunk

The `MemoryChunk` struct represents a large chunk of allocated device memory. It contains:

- `memory`: The Vulkan `vk::DeviceMemory`.
- `size`: The total size of the chunk.
- `free_regions`: A `Vec` of `(u64, u64)` tuples, representing free regions within the chunk (offset, size).
- `memory_type_index`: The Vulkan memory type index.

### MemoryBlock

The `MemoryBlock` struct represents a block of allocated memory. It contains:

- `memory`: The Vulkan `vk::DeviceMemory`.
- `offset`: The offset of the block within the `vk::DeviceMemory`.
- `size`: The size of the block.
- `memory_type_index`: The Vulkan memory type index.

### Cleanup

The `MemoryAllocator` implements `Drop`. When it is dropped, it frees all allocated memory chunks, and logs warnings if any memory leaks are detected (i.e. if not all allocated memory was freed).

## Buffer

The `Buffer` struct (in `memory/buffer.rs`) provides a higher-level abstraction for creating and managing Vulkan buffers and their associated memory. It uses the `MemoryAllocator` to allocate memory. The `Buffer::map` function allows mapping the buffer's memory into host address space, returning a `BufferView` which provides safe access (using slices) to the mapped memory, and automatically unmaps the memory when dropped.
