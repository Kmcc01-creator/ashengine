# Render Graph System

The render graph system provides a flexible and efficient way to manage complex rendering pipelines. It handles dependencies between render passes, manages resource transitions, and ensures optimal execution order.

## Overview

The render graph is a directed acyclic graph (DAG) where:

- Nodes represent render passes
- Edges represent dependencies between passes
- Resources (textures, buffers) flow between passes

```
[Geometry Pass] → [Lighting Pass] → [Post Process] → [UI Pass]
      ↓                  ↑
[Shadow Pass] ───────────
```

## Core Components

### PassId

A unique identifier for each render pass in the graph:

```rust
let gbuffer_pass = graph.add_pass(geometry_pass_desc)?;
let lighting_pass = graph.add_pass(lighting_pass_desc)?;
```

### AttachmentType

Describes the type and format of pass attachments:

```rust
pub enum AttachmentType {
    Color {
        format: TextureFormat,
        clear: bool,
    },
    Depth {
        format: TextureFormat,
        clear: bool,
    },
    Input {
        format: TextureFormat,
    },
}
```

### PassDesc

Configuration for a render pass:

```rust
let pass = PassDesc {
    name: "GBuffer".into(),
    attachments: vec![
        // Position buffer
        AttachmentDesc {
            ty: AttachmentType::Color {
                format: TextureFormat::R32G32B32A32_SFLOAT,
                clear: true,
            },
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        // ... other attachments
    ],
    color_attachments: vec![0, 1, 2],
    depth_attachment: Some(3),
    input_attachments: vec![],
    dependencies: HashSet::new(),
};
```

## Usage

### 1. Creating a Render Graph

```rust
// Create the render system
let mut render_system = RenderSystem::new(device.clone(), resource_manager.clone());

// Initialize deferred rendering
render_system.init_deferred(DeferredConfig {
    width: 1920,
    height: 1080,
    samples: vk::SampleCountFlags::TYPE_1,
})?;
```

### 2. Defining Passes

```rust
// Create geometry pass
let gbuffer_config = pass_manager.create_geometry_pass(
    width,
    height,
    samples,
);
let gbuffer_desc = pass_manager.create_pass_desc(&gbuffer_config);
let gbuffer_pass_id = graph.add_pass(gbuffer_desc)?;

// Create lighting pass with dependency
let mut lighting_desc = pass_manager.create_pass_desc(&lighting_config);
lighting_desc.dependencies.insert(gbuffer_pass_id);
lighting_desc.input_attachments = vec![0, 1, 2, 3]; // G-buffer attachments
let lighting_pass_id = graph.add_pass(lighting_desc)?;
```

### 3. Creating Resources

```rust
// Create attachment textures
let position_buffer = resource_manager.create_texture(TextureDescriptor {
    width: 1920,
    height: 1080,
    format: TextureFormat::R32G32B32A32_SFLOAT,
    usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
    ..Default::default()
})?;
```

### 4. Executing the Graph

```rust
// Begin frame
let command_buffer = render_system.begin_frame()?;

// Execute render graph
render_system.graph().execute(command_buffer)?;

// End frame
render_system.end_frame()?;
```

## Pass Types

### 1. Geometry Pass

Writes geometric information to the G-buffer:

- Position
- Normal
- Albedo/Base Color
- Metallic/Roughness
- Depth

### 2. Lighting Pass

Processes G-buffer data to compute final lighting:

- Reads G-buffer textures
- Applies light sources
- Computes PBR lighting
- Outputs final color

### 3. Post-Process Pass

Applies screen-space effects:

- Tone mapping
- Bloom
- Ambient Occlusion
- Anti-aliasing

### 4. UI Pass

Renders 2D interface elements:

- Transparent blending
- No depth testing
- Screen-space coordinates

## Advanced Features

### 1. Automatic Synchronization

The render graph automatically:

- Inserts necessary barriers
- Handles image layout transitions
- Manages resource dependencies

### 2. Resource Aliasing

```rust
// Resources can be reused when their lifetimes don't overlap
graph.alias_resource(resource_a, resource_b)?;
```

### 3. Dynamic Resolution

```rust
// Resize all resources
render_system.handle_resize([new_width, new_height])?;
```

### 4. Pass Reordering

The graph will automatically:

- Determine optimal execution order
- Detect cycles
- Minimize pipeline barriers

## Best Practices

1. **Resource Management**

   - Use appropriate formats for each attachment
   - Consider memory usage when creating high-resolution buffers
   - Clear attachments only when necessary

2. **Performance**

   - Group similar passes together
   - Minimize dependencies between passes
   - Use appropriate image layouts
   - Consider bandwidth when designing passes

3. **Debugging**

   - Give passes descriptive names
   - Use validation layers during development
   - Monitor resource usage and transitions

4. **Extensibility**
   - Design passes to be modular
   - Consider future requirements when defining dependencies
   - Document pass requirements and effects

## Common Patterns

### Deferred Rendering

```rust
// 1. G-buffer pass
let gbuffer_pass = graph.add_pass(geometry_pass_desc)?;

// 2. Lighting pass
let mut lighting_desc = lighting_pass_config.into_desc();
lighting_desc.dependencies.insert(gbuffer_pass);
let lighting_pass = graph.add_pass(lighting_desc)?;

// 3. Post-process pass
let mut post_desc = post_process_config.into_desc();
post_desc.dependencies.insert(lighting_pass);
let post_pass = graph.add_pass(post_desc)?;
```

### Multiple Viewports

```rust
// Main view
let main_pass = graph.add_pass(main_view_desc)?;

// Picture-in-picture view
let mut pip_desc = pip_view_config.into_desc();
pip_desc.dependencies.insert(main_pass);
let pip_pass = graph.add_pass(pip_desc)?;
```

## Error Handling

The render graph provides detailed error information:

- Cycle detection
- Resource misuse
- Invalid dependencies
- Missing resources

```rust
match graph.validate() {
    Ok(_) => println!("Graph is valid"),
    Err(e) => println!("Graph error: {}", e),
}
```
