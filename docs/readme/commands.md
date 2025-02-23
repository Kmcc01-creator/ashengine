# Commands

This document describes the command buffers and pools in AshEngine.

AshEngine uses two main structs for command management: `CommandPool` and `CommandBuffer`.

## CommandPool

The `CommandPool` is responsible for allocating and managing command buffers.

### Creation

A `CommandPool` is created using the `new` function:

```rust
pub fn new(
    device: Arc<Device>,
    queue_family_index: u32,
    flags: vk::CommandPoolCreateFlags,
) -> Result<Self> { ... }
```

- `device`: The Vulkan logical device.
- `queue_family_index`: The index of the queue family this pool will allocate from.
- `flags`: Flags controlling the behavior of the command pool (e.g., `vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER` to allow individual command buffer resets).

### Allocation

Command buffers are allocated from the pool using the `allocate_buffers` function:

```rust
pub fn allocate_buffers(
    &self,
    level: vk::CommandBufferLevel,
    count: u32,
) -> Result<Vec<CommandBuffer>> { ... }
```

- `level`: The level of the command buffers (Primary or Secondary).
- `count`: The number of command buffers to allocate.

### Cleanup

The `CommandPool` implements the `Drop` trait. When a `CommandPool` is dropped, it automatically destroys the underlying Vulkan command pool, freeing associated resources.

## CommandBuffer

The `CommandBuffer` struct represents a Vulkan command buffer, which is used to record and submit graphics commands to the GPU.

### States

A `CommandBuffer` has different states to manage its lifecycle:

- `Initial`: The initial state after allocation or reset.
- `Recording`: The state while commands are being recorded.
- `Executable`: The state after recording has finished and the buffer is ready for submission.
- `Pending`: The state after the buffer has been submitted to a queue but has not yet completed execution.
- `Invalid`: An error state (currently unused, reserved for future use).

### Recording

Command recording begins with the `begin` function and ends with the `end` function (provided by the RAII guard `CommandBufferRecording`):

```rust
pub fn begin(&mut self, flags: vk::CommandBufferUsageFlags) -> Result<CommandBufferRecording> { ... }
```

- `flags`: Specify usage flags for the command buffer.

The `begin` function returns a `CommandBufferRecording` object, which acts as an RAII guard.

### CommandBufferRecording (RAII Guard)

The `CommandBufferRecording` is an RAII guard that ensures the command buffer is properly ended when it goes out of scope. You obtain a `CommandBufferRecording` object when you call `begin` on a `CommandBuffer`.

```rust
pub struct CommandBufferRecording<'a> {
    cmd: &'a mut CommandBuffer,
}
impl<'a> CommandBufferRecording<'a> {
    pub fn end(self) -> Result<()> { ... }
    // ... other command recording functions ...
}
```

You use the methods of `CommandBufferRecording` to record commands into the buffer. When the `CommandBufferRecording` object is dropped (when it goes out of scope), the `end` method is automatically called, ending the command buffer recording and transitioning the command buffer to the `Executable` state.

The `CommandBufferRecording` struct provides functions for recording commands, such as:

- `bind_pipeline`: Binds a graphics or compute pipeline.
- `bind_vertex_buffers`: Binds vertex buffers.
- `bind_index_buffer`: Binds an index buffer.
- `begin_render_pass`: Begins a render pass.
- `end_render_pass`: Ends the current render pass.
- `draw`: Performs a draw call.
- `draw_indexed`: Performs an indexed draw call.

### Resetting

A command buffer can be reset to the initial state using the `reset` function:

```rust
pub fn reset(&mut self, release_resources: bool) -> Result<()> { ... }
```

- `release_resources`: If `true`, releases resources held by the command buffer.

### Submission

A command buffer is submitted to a queue using the `submit` function:

```rust
pub fn submit(
    &mut self,
    queue: vk::Queue,
    wait_semaphores: &[vk::Semaphore],
    wait_stages: &[vk::PipelineStageFlags],
    signal_semaphores: &[vk::Semaphore],
    fence: vk::Fence,
) -> Result<()> { ... }
```

- `queue`: The queue to submit to.
- `wait_semaphores`: Semaphores to wait on before executing the command buffer.
- `wait_stages`: Pipeline stages at which to wait for the semaphores.
- `signal_semaphores`: Semaphores to signal when the command buffer finishes execution.
- `fence`: A fence to signal when the command buffer finishes execution.

### Handle

The raw Vulkan command buffer handle can be accessed using the `handle()` method (available on both `CommandBuffer` and `CommandBufferRecording`).
