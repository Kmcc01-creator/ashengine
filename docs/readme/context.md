# Context

This document describes the `Context` in AshEngine.

The `Context` struct is a central component of AshEngine, responsible for managing the core Vulkan objects:

- Vulkan Instance
- Physical Device (GPU)
- Logical Device
- Surface (if a window is provided)
- Graphics Queue
- Loaders for surface and swapchain extensions

## Initialization

To create a `Context`, you typically use the `new` function, optionally providing a `winit::window::Window` if you want to render to a window:

```rust
use ashengine::engine::context::Context;
use winit::window::Window;

// Assuming you have a 'window' object of type winit::window::Window
let context = Context::new(Some(&window)).expect("Failed to create context");

// If you don't need a window (e.g., for offscreen rendering), pass None:
// let context = Context::new(None).expect("Failed to create context");

```

The `new` function performs the following steps:

1.  Loads the Vulkan library.
2.  Creates a Vulkan instance with the necessary extensions (surface extensions are automatically added based on the target operating system if a window is provided).
3.  If a window is provided, it creates a Vulkan surface.
4.  Selects a suitable physical device (GPU), preferring discrete GPUs if available.
5.  Creates a logical device with a graphics queue and the required extensions (swapchain extension).
6.  Retrieves the graphics queue.
7.  Creates loaders for the surface and swapchain extensions.

## Accessing Vulkan Objects

The `Context` provides methods to access the underlying Vulkan objects:

- `device()`: Returns an `Arc<Device>`.
- `physical_device()`: Returns the `vk::PhysicalDevice`.
- `surface()`: Returns the `vk::SurfaceKHR`.
- `queue_family_index()`: Returns the index of the graphics queue family.
- `graphics_queue()`: Returns the `vk::Queue`.
- `instance()`: Returns an `Arc<Instance>`.
- `surface_loader()`: Returns an `Arc<ash::extensions::khr::Surface>`.
- `swapchain_loader()`: Returns an `Arc<ash::extensions::khr::Swapchain>`.

## Cleanup

The `Context` implements the `Drop` trait, which automatically cleans up the Vulkan resources (surface, device, instance) when the `Context` goes out of scope. You don't need to manually destroy these objects.
