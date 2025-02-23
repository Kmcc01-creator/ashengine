# Pipeline System

The AshEngine pipeline system provides a flexible and efficient way to manage graphics pipelines, with support for pipeline variants, caching, and specialized configurations.

## Core Concepts

### Pipeline Variants

Pipelines can have multiple variants based on different state configurations and specialization constants. This allows for efficient creation of pipeline variations for different rendering scenarios.

```rust
// Create base pipeline configuration
let base = PipelineKey {
    shaders: vec![
        (vk::ShaderStageFlags::VERTEX, vertex_shader),
        (vk::ShaderStageFlags::FRAGMENT, fragment_shader),
    ],
    render_pass: pass_handle,
    subpass: 0,
};

// Create variant with specific state
let variant = PipelineVariant::new(base)
    .with_state(PipelineStateConfig {
        blend_mode: BlendMode::Alpha,
        depth_config: DepthConfig::default(),
        ..Default::default()
    })
    .with_specialization(spec_info);

// Get or create pipeline
let pipeline = pipeline_manager.get_pipeline(variant)?;
```

### Pipeline State Configuration

The state configuration system allows for detailed control over various pipeline aspects:

- **Vertex Input**: Configure vertex attributes and bindings
- **Rasterization**: Control polygon mode, culling, and line width
- **Depth Testing**: Configure depth test and write operations
- **Blending**: Choose from preset modes or create custom blend configurations
- **Dynamic State**: Specify which states can be changed dynamically

```rust
let config = PipelineStateConfig {
    vertex_config: VertexConfig {
        bindings: vec![
            VertexBinding {
                binding: 0,
                stride: size_of::<Vertex>() as u32,
                input_rate: VertexInputRate::Vertex,
            }
        ],
        attributes: vec![
            VertexAttribute {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            }
        ],
    },
    rasterization: RasterizationConfig {
        cull_mode: vk::CullModeFlags::BACK,
        ..Default::default()
    },
    ..Default::default()
};
```

### Pipeline Caching

The system includes a pipeline cache that:

- Reuses existing pipelines when possible
- Serializes cache data to disk
- Tracks cache statistics
- Manages pipeline lifecycle

```rust
// Save cache to disk
pipeline_manager.save_cache(Path::new("pipeline_cache.bin"))?;

// Load cache from disk
pipeline_manager.load_cache(Path::new("pipeline_cache.bin"))?;

// Get cache statistics
let stats = pipeline_manager.cache_stats();
println!("Cache hit rate: {:.2}%", stats.hit_rate() * 100.0);
```

### Layout Management

Pipeline layouts and descriptor sets are managed efficiently:

- Automatic layout creation based on shader reflection
- Descriptor set layout caching
- Bind group abstraction for logical resource grouping

```rust
// Create a bind group layout
let layout = BindGroupLayout::new()
    .add_binding(BindingDesc {
        binding: 0,
        ty: BindingType::UniformBuffer,
        count: 1,
        stages: vk::ShaderStageFlags::VERTEX,
    });

// Layout is automatically cached and reused
let pipeline_layout = pipeline_manager.create_layout(&PipelineLayoutDesc {
    bind_group_layouts: vec![layout],
    push_constant_ranges: vec![],
})?;
```

## Best Practices

1. **Pipeline Variants**:

   - Group similar pipelines into variants
   - Use specialization constants for minor variations
   - Keep track of common configurations

2. **State Management**:

   - Use dynamic state for frequently changing properties
   - Group static state into reusable configurations
   - Consider performance implications of state changes

3. **Caching**:

   - Save cache to disk between sessions
   - Monitor cache statistics
   - Adjust cache size based on application needs

4. **Resource Binding**:
   - Group related resources into bind groups
   - Minimize descriptor set changes
   - Use push constants for small, frequently updated data

## Future Improvements

The pipeline system roadmap includes:

- [ ] Shader reflection for automatic layout generation
- [ ] Pipeline derivatives for faster variant creation
- [ ] Extended specialization support
- [ ] Dynamic pipeline recompilation
