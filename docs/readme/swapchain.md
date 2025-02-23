# Swapchain

This document describes the `Swapchain` in AshEngine.

The `Swapchain` struct represents a Vulkan swapchain, which is a collection of images used for presenting rendered frames to the screen.

## Creation

The `Swapchain` is created using the `new` function:

```rust
pub fn new(
    context: Arc<Context>,
    surface: vk::SurfaceKHR,
    width: u32,
    height: u32,
) -> Result<Self> { ... }
```

- `context`: The AshEngine `Context`.
- `surface`: The Vulkan surface.
- `width`: The desired width of the swapchain images.
- `height`: The desired height of the swapchain images.

The `new` function:

- Retrieves surface capabilities, formats, and present modes.
- Chooses an appropriate extent, image count, format, and present mode.
- Creates the Vulkan swapchain.
- Retrieves the swapchain images.
- Creates image views for each image.

## Recreation

When the window is resized, the swapchain needs to be recreated. This is done using the `recreate` function:

```rust
pub fn recreate(&mut self, width: u32, height: u32, surface: vk::SurfaceKHR) -> Result<()> { ... }
```

- `width`: The new width.
- `height`: The new height.
- `surface`: The Vulkan surface.

The `recreate` function:

- Gets new surface capabilities.
- Creates a new swapchain, reusing the old one as `old_swapchain` to potentially improve performance.
- Destroys old image views and the old swapchain.
- Retrieves new swapchain images.
- Creates new image views.
- Updates internal fields.

## Acquiring and Presenting Images

To render, you need to acquire an image from the swapchain, and after rendering, present it. This is done using `acquire_next_image` and `present`:

```rust
pub fn acquire_next_image(
    &self,
    semaphore: vk::Semaphore,
    fence: vk::Fence,
) -> Result<(u32, bool)> { ... }

pub fn present(
    &self,
    queue: vk::Queue,
    image_index: u32,
    wait_semaphores: &[vk::Semaphore],
) -> Result<bool> { ... }
```

- `acquire_next_image`: Acquires the next available image index.
  - `semaphore`: A semaphore to signal when the image is acquired.
  - `fence`: An optional fence to signal when the image is acquired.
- `present`: Presents a rendered image to the screen.
  - `queue`: The graphics queue.
  - `image_index`: The index of the image to present.
  - `wait_semaphores`: Semaphores to wait on before presenting.

## Accessors

The `Swapchain` struct provides methods to access its properties:

- `extent()`: Returns the `vk::Extent2D` of the swapchain images.
- `surface_format()`: Returns the `vk::Format` of the swapchain images.
- `image_views()`: Returns a slice of `vk::ImageView` for the swapchain images.

## Cleanup

The `Swapchain` implements the `Drop` trait, which automatically cleans up resources (image views and swapchain) when the `Swapchain` is dropped.
