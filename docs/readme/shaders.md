# Shaders

This document describes how shaders are handled in AshEngine.

AshEngine uses two main structs for managing shaders: `ShaderModule` and `ShaderSet`.

## ShaderModule

The `ShaderModule` struct represents a single compiled SPIR-V shader module.

### Creation

A `ShaderModule` can be created from a SPIR-V file using the `from_file` or `new` function:

```rust
pub fn from_file(device: Arc<Device>, spirv_path: impl AsRef<Path>) -> Result<Self> { ... }
pub fn new(device: Arc<Device>, spirv_path: impl AsRef<Path>) -> Result<Self> { ... }

```

- `device`: The Vulkan logical device.
- `spirv_path`: The path to the SPIR-V file.

The functions:

1.  Reads the SPIR-V file.
2.  Validates that data is valid SPIR-V.
3.  Creates a `vk::ShaderModule`.

### Shader Stage Creation

The `create_shader_stage` function creates a `vk::PipelineShaderStageCreateInfo` for use in a graphics pipeline:

```rust
pub fn create_shader_stage(
    &self,
    stage: vk::ShaderStageFlags,
) -> vk::PipelineShaderStageCreateInfo { ... }
```

- `stage`: The shader stage (e.g., `vk::ShaderStageFlags::VERTEX`, `vk::ShaderStageFlags::FRAGMENT`).

This function sets the entry point name to "main".

### Cleanup

The `ShaderModule` implements the `Drop` trait. When a `ShaderModule` is dropped, it automatically destroys the underlying Vulkan shader module.

## ShaderSet

The `ShaderSet` struct groups together a vertex shader and a fragment shader, typically used together in a graphics pipeline.

### Creation

A `ShaderSet` is created using the `new` function:

```rust
pub fn new(
    device: Arc<Device>,
    vert_path: impl AsRef<Path>,
    frag_path: impl AsRef<Path>,
) -> Result<Self> { ... }
```

- `device`: The Vulkan logical device.
- `vert_path`: The path to the vertex shader SPIR-V file.
- `frag_path`: The path to the fragment shader SPIR-V file.

This function creates `ShaderModule` instances for both the vertex and fragment shaders.

### Shader Stage Creation

The `create_shader_stages` function creates an array of `vk::PipelineShaderStageCreateInfo` for both the vertex and fragment shaders:

```rust
pub fn create_shader_stages(&self) -> [vk::PipelineShaderStageCreateInfo; 2] { ... }
```

This array can be directly used when creating a graphics pipeline.

### Cleanup

The `ShaderSet` ensures that shader modules are destroyed.
