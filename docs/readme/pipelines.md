# Pipelines

This document describes the graphics pipelines in AshEngine.

The `Pipeline` struct represents a Vulkan graphics pipeline, which defines the stages of the rendering process.

## Creation

The `Pipeline` is created using the `new` function:

```rust
pub fn new(
    device: Arc<Device>,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    shader_stages: &[vk::PipelineShaderStageCreateInfo],
    descriptor_set_layouts: &[vk::DescriptorSetLayout],
) -> Result<Self> { ... }
```

- `device`: The Vulkan logical device.
- `render_pass`: The render pass the pipeline will be used with.
- `extent`: The extent (width and height) of the viewport.
- `shader_stages`: An array of shader stage creation infos.
- `descriptor_set_layouts`: Descriptor set layouts used by the pipeline.

The `new` function creates the pipeline by setting up the following state:

- **Dynamic State:** Viewport and scissor are set as dynamic state, meaning they can be changed without recreating the pipeline.
- **Vertex Input State:** Configured using `TextVertex::get_binding_description()` and `TextVertex::get_attribute_descriptions()`, indicating that the pipeline is designed to work with vertices of type `TextVertex`.
- **Input Assembly State:**
  - `topology`: Set to `vk::PrimitiveTopology::TRIANGLE_LIST`, meaning the pipeline will render triangles.
  - `primitive_restart_enable`: Set to `false`.
- **Viewport State:** Sets up a single viewport and scissor based on the provided `extent`.
- **Rasterization State:**
  - `depth_clamp_enable`: `false`.
  - `rasterizer_discard_enable`: `false`.
  - `polygon_mode`: `vk::PolygonMode::FILL` (filled triangles).
  - `line_width`: `1.0`.
  - `cull_mode`: `vk::CullModeFlags::BACK` (backface culling).
  - `front_face`: `vk::FrontFace::CLOCKWISE`.
  - `depth_bias_enable`: `false`.
- **Multisample State:**
  - `sample_shading_enable`: `false`.
  - `rasterization_samples`: `vk::SampleCountFlags::TYPE_1` (no multisampling).
- **Color Blend State:** Enables alpha blending with the following settings:
  - `src_color_blend_factor`: `vk::BlendFactor::SRC_ALPHA`
  - `dst_color_blend_factor`: `vk::BlendFactor::ONE_MINUS_SRC_ALPHA`
  - `color_blend_op`: `vk::BlendOp::ADD`
  - `src_alpha_blend_factor`: `vk::BlendFactor::ONE`
  - `dst_alpha_blend_factor`: `vk::BlendFactor::ZERO`
  - `alpha_blend_op`: `vk::BlendOp::ADD`
- **Pipeline Layout:** Creates a pipeline layout based on provided descriptor set layouts.

## Binding

The `bind` function binds the pipeline to a command buffer:

```rust
pub fn bind(&self, command_buffer: vk::CommandBuffer) { ... }
```

This function sets dynamic viewport and scissor based on the pipeline's extent.

## Accessors

The `Pipeline` struct provides methods to access its properties:

- `layout()`: Returns the `vk::PipelineLayout`.
- `pipeline()`: Returns the `vk::Pipeline`.
- `extent()`: Returns the `vk::Extent2D`.

## Cleanup

The `Pipeline` implements the `Drop` trait, which automatically cleans up resources (pipeline and pipeline layout) when the `Pipeline` is dropped.
