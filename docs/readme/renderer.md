# Renderer

This document describes the `Renderer` in AshEngine.

The `Renderer` struct is responsible for managing the rendering process, including:

- Synchronization (semaphores and fences)
- Command pool and buffer management
- Swapchain initialization and recreation
- Render pass and pipeline management
- Frame lifecycle (begin/end frame, acquire/present)
- Physics and lighting updates

## Initialization

The `Renderer` is created using the `new` function, which takes several arguments:

```rust
pub fn new(
    device: Arc<Device>,
    graphics_queue: vk::Queue,
    queue_family_index: u32,
    physical_device: vk::PhysicalDevice,
    instance: Arc<Instance>,
    surface_loader: Arc<ash::extensions::khr::Surface>,
    surface: vk::SurfaceKHR,
    shader_set: ShaderSet,
    descriptor_set_layouts: &[vk::DescriptorSetLayout],
) -> Result<Self> { ... }
```

- `device`: The Vulkan logical device.
- `graphics_queue`: The graphics queue.
- `queue_family_index`: The index of the graphics queue family.
- `physical_device`: The Vulkan physical device.
- `instance`: The Vulkan instance.
- `surface_loader`: The surface loader.
- `surface`: The Vulkan surface.
- `shader_set`: A set of shaders to be used for rendering.
- `descriptor_set_layouts`: Descriptor set layouts.

The `new` function creates:

- A command pool for allocating command buffers.
- Synchronization objects (semaphores and fences) for each frame in flight.
- Initial physics world with a couple of example objects.
- Initial lighting setup.

## Swapchain Initialization

Before rendering, you need to initialize the swapchain, render pass, and pipeline using the `initialize_swapchain` function:

```rust
pub fn initialize_swapchain(
    &mut self,
    swapchain: Swapchain,
    render_pass: RenderPass,
) -> Result<()> { ... }
```

This function:

- Allocates command buffers.
- Creates the graphics pipeline.
- Sets the internal `swapchain`, `render_pass`, and `pipeline` fields.

## Handling Resizes

When the window is resized, you need to call the `handle_resize` function:

```rust
pub fn handle_resize(&mut self, dimensions: [u32; 2]) -> Result<()> { ... }
```

This function:

- Waits for the device to be idle.
- Recreates the swapchain, render pass, and pipeline with the new dimensions.

## Frame Lifecycle

The rendering loop typically involves the following steps:

1.  **Begin Frame:** Call `begin_frame()` to start a new frame. This function:

    - Waits for the previous frame's fence to be signaled.
    - Resets the command buffer.
    - Acquires the next image from the swapchain.
    - Begins the render pass.
    - Binds the pipeline.
    - Updates the physics world.

2.  **Record Drawing Commands:** Record your drawing commands into the command buffer (this is not handled by the `Renderer` itself, but by your application code).

3.  **End Frame:** Call `end_frame()` to finish the frame. This function:
    - Ends the render pass.
    - Ends the command buffer recording.
    - Submits the command buffer to the graphics queue.
    - Presents the rendered image to the swapchain.
    - Advances to the next frame.

## Accessing Resources

The `Renderer` provides methods to access internal objects:

- `image_available_semaphore()`: Returns the semaphore that signals when an image is available.
- `render_finished_semaphore()`: Returns the semaphore that signals when rendering is finished.
- `in_flight_fence()`: Returns the fence that signals when the current frame is complete.
- `device()`: Returns a reference to the `Device`.
- `current_command_buffer()`: Returns the current frame's command buffer.
- `pipeline_layout()`: Returns the pipeline layout.

## Cleanup

The `Renderer` implements the `Drop` trait, which automatically cleans up resources (command pool, semaphores, fences) when the `Renderer` is dropped.
