# Resource Management System

The resource management system provides a safe and efficient way to handle Vulkan resources through a handle-based approach. It manages resource lifecycles, memory allocation, and state tracking.

## Overview

The system uses unique handles to reference resources:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle(u64);
```

Resources are categorized by type:

```rust
pub enum ResourceType {
    Buffer(BufferType),
    Texture,
    Material,
    Shader,
}

pub enum BufferType {
    Vertex,
    Index,
    Uniform,
    Storage,
}
```

## Resource Manager

The central manager for all graphics resources:

```rust
pub struct ResourceManager {
    device: Arc<ash::Device>,
    resources: RwLock<HashMap<ResourceHandle, ResourceType>>,
    buffers: RwLock<HashMap<ResourceHandle, vk::Buffer>>,
    buffer_memories: RwLock<HashMap<ResourceHandle, vk::DeviceMemory>>,
    texture_manager: TextureManager,
    shader_manager: ShaderManager,
}
```

### Usage Example

```rust
// Create buffer resource
let vertex_buffer = resource_manager.create_buffer(
    size,
    vk::BufferUsageFlags::VERTEX_BUFFER,
    BufferType::Vertex,
)?;

// Create texture resource
let texture = resource_manager.create_texture(TextureDescriptor {
    width: 1024,
    height: 1024,
    format: TextureFormat::R8G8B8A8_UNORM,
    usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
    ..Default::default()
})?;

// Create shader resource
let shader = resource_manager.create_shader(ShaderDescriptor {
    code: spirv_code,
    stage: ShaderStage::Vertex,
    entry_point: "main".into(),
})?;
```

## Specialized Resource Managers

### 1. Texture Manager

Handles image resources:

```rust
// Create texture manager
let texture_manager = TextureManager::new(device.clone());

// Create texture
let texture = texture_manager.create_texture(TextureDescriptor {
    width: 1024,
    height: 1024,
    format: TextureFormat::R8G8B8A8_UNORM,
    data: Some(image_data),
    usage: vk::ImageUsageFlags::SAMPLED,
})?;

// Get texture info
if let Some((view, sampler)) = texture_manager.get_texture(texture) {
    // Use texture in rendering
}
```

### 2. Shader Manager

Manages shader modules:

```rust
// Create shader manager
let shader_manager = ShaderManager::new(device.clone());

// Create shader
let shader = shader_manager.create_shader(ShaderDescriptor {
    code: spirv_code,
    stage: ShaderStage::Fragment,
    entry_point: "main".into(),
})?;

// Get shader info for pipeline creation
if let Some(stage_info) = shader_manager.get_stage_info(shader) {
    // Use in pipeline creation
}
```

### 3. Material Manager

Handles material resources:

```rust
// Create material
let material = material_manager.create_material(MaterialDescriptor {
    name: "Metal".into(),
    shader_handle: shader,
    parameters: material_params,
})?;

// Update material
material_manager.update_material(material, &[
    ("albedo", MaterialParam::Vec3([1.0, 0.0, 0.0])),
    ("roughness", MaterialParam::Float(0.5)),
]);
```

## Memory Management

### Buffer Memory Allocation

```rust
impl ResourceManager {
    fn allocate_buffer_memory(
        &self,
        buffer: vk::Buffer,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<vk::DeviceMemory> {
        let requirements = unsafe {
            self.device.get_buffer_memory_requirements(buffer)
        };

        let memory_type = self.find_memory_type(
            requirements.memory_type_bits,
            properties,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type);

        unsafe {
            self.device
                .allocate_memory(&alloc_info, None)
                .map_err(|e| VulkanError::MemoryAllocation(e.to_string()))
        }
    }
}
```

### Memory Pooling

```rust
// Group similar allocations
struct MemoryPool {
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    used: vk::DeviceSize,
    blocks: Vec<MemoryBlock>,
}

// Allocate from pool
impl MemoryPool {
    fn allocate(&mut self, size: vk::DeviceSize) -> Option<MemoryBlock> {
        // Find suitable block using best-fit strategy
        if let Some(block) = self.find_free_block(size) {
            self.used += size;
            Some(block)
        } else {
            None
        }
    }
}
```

## Resource Lifecycle

### 1. Creation

```rust
// Create resource with automatic cleanup
let handle = resource_manager.create_buffer(size, usage, buffer_type)?;

// Resource is tracked internally
self.resources.write().insert(handle, ResourceType::Buffer(buffer_type));
```

### 2. Usage

```rust
// Safe access through handle
if let Some(buffer) = resource_manager.get_buffer(handle) {
    // Use buffer
}

// Type-safe resource access
match resource_manager.get_resource_type(handle) {
    Some(ResourceType::Buffer(_)) => { /* Handle buffer */ }
    Some(ResourceType::Texture) => { /* Handle texture */ }
    _ => { /* Handle error */ }
}
```

### 3. Cleanup

```rust
impl Drop for ResourceManager {
    fn drop(&mut self) {
        // Clean up all resources
        let handles: Vec<_> = self.resources.read().keys().copied().collect();
        for handle in handles {
            self.destroy_resource(handle);
        }
    }
}
```

## Best Practices

### 1. Resource Creation

```rust
// Group similar resources
let buffers = (0..COUNT).map(|_| {
    resource_manager.create_buffer(
        BUFFER_SIZE,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        BufferType::Vertex,
    )
}).collect::<Result<Vec<_>>>()?;
```

### 2. Memory Management

```rust
// Use appropriate memory types
let vertex_buffer = resource_manager.create_buffer(
    size,
    vk::BufferUsageFlags::VERTEX_BUFFER,
    BufferType::Vertex,
)?;

// Use staging buffers for GPU-only memory
let staging = resource_manager.create_staging_buffer(size, data)?;
resource_manager.copy_buffer(staging, vertex_buffer)?;
```

### 3. Resource Updates

```rust
// Batch updates
resource_manager.update_buffers(&[
    (buffer1, offset1, data1),
    (buffer2, offset2, data2),
]);
```

### 4. Error Handling

```rust
// Handle resource errors
match resource_manager.create_buffer(size, usage, type) {
    Ok(handle) => { /* Use resource */ }
    Err(VulkanError::OutOfMemory(_)) => { /* Handle memory error */ }
    Err(e) => { /* Handle other errors */ }
}
```

## Performance Considerations

1. **Memory Allocation**:

   - Use memory pools for similar resources
   - Align allocations to device requirements
   - Batch small allocations

2. **Resource Access**:

   - Cache frequently used resources
   - Use handle lookup tables
   - Minimize state changes

3. **Updates**:
   - Batch resource updates
   - Use staging buffers appropriately
   - Consider update frequency in memory type selection
