# Getting Started with AshEngine

This tutorial walks through setting up and creating a basic 3D scene using AshEngine.

## Setup

Add AshEngine to your Cargo.toml:

```toml
[dependencies]
ashengine = "0.1.0"
```

## Basic Scene Setup

```rust
use ashengine::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the engine
    let mut engine = Engine::new(WindowConfig {
        title: "My First Scene",
        width: 800,
        height: 600,
        ..Default::default()
    })?;

    // Create world and add systems
    let mut world = World::new();
    world.add_system(RenderSystem::new(), SystemStage::Late);

    // Run the engine
    engine.run(world)
}
```

## Adding 3D Objects

### Creating a Basic Mesh

```rust
// Create cube mesh
let cube = world.resource_manager().create_mesh(MeshDesc {
    vertices: cube_vertices(),
    indices: cube_indices(),
    topology: PrimitiveTopology::TriangleList,
})?;

// Create entity with mesh
let entity = world
    .create_entity()
    .with(TransformComponent::new())
    .with(StaticMeshRenderer::new(cube))
    .build();
```

## Setting Up Materials

### Basic Material

```rust
// Create a basic material
let material = world.resource_manager().create_material(MaterialDesc {
    name: "Basic".into(),
    shader: basic_shader,
    parameters: {
        let mut params = HashMap::new();
        params.insert("color".into(), MaterialParam::Vec4([1.0, 0.0, 0.0, 1.0]));
        params
    },
})?;

// Apply to mesh
world.get_component_mut::<StaticMeshRenderer>(entity)?.with_material(material);
```

## Pipeline Configuration

### Creating a Pipeline Variant

```rust
// Create base pipeline configuration
let base = PipelineKey {
    shaders: vec![
        (ShaderStage::Vertex, vertex_shader),
        (ShaderStage::Fragment, fragment_shader),
    ],
    render_pass: main_pass,
    subpass: 0,
};

// Create pipeline variant with specific state
let variant = PipelineVariant::new(base)
    .with_state(PipelineStateConfig {
        blend_mode: BlendMode::Alpha,
        depth_config: DepthConfig {
            test_enable: true,
            write_enable: true,
            compare_op: CompareOp::Less,
        },
        rasterization: RasterizationConfig {
            cull_mode: CullMode::Back,
            front_face: FrontFace::CounterClockwise,
            ..Default::default()
        },
        ..Default::default()
    });

// Get or create pipeline
let pipeline = world.resource_manager().get_pipeline(variant)?;
```

### Using Pipeline Cache

```rust
// Save pipeline cache to disk
world.resource_manager().save_pipeline_cache("pipeline_cache.bin")?;

// Load pipeline cache on startup
world.resource_manager().load_pipeline_cache("pipeline_cache.bin")?;
```

## Camera Setup

```rust
// Create camera entity
let camera = world
    .create_entity()
    .with(TransformComponent::new())
    .with(CameraComponent::new(CameraDesc {
        fov: 60.0,
        near: 0.1,
        far: 1000.0,
        ..Default::default()
    }))
    .build();

// Position camera
if let Some(transform) = world.get_component_mut::<TransformComponent>(camera) {
    transform.set_position([0.0, 0.0, -5.0]);
    transform.look_at([0.0, 0.0, 0.0]);
}
```

## Input Handling

```rust
// Add input system
world.add_system(InputSystem::new(), SystemStage::Early);

// Handle input in a custom system
struct PlayerSystem;

impl System for PlayerSystem {
    fn update(&mut self, world: &mut World) {
        let input = world.resource::<Input>();

        if input.key_pressed(KeyCode::Space) {
            // Handle input
        }
    }
}
```

## Scene Organization

```rust
// Create scene hierarchy
let parent = world
    .create_entity()
    .with(TransformComponent::new())
    .build();

let child = world
    .create_entity()
    .with(TransformComponent::new())
    .with(ParentComponent::new(parent))
    .build();
```

## Next Steps

- Check out the [Materials Guide](../graphics/materials.md) for advanced material setup
- Learn about the [Render Graph](../graphics/render_graph.md) for custom rendering
- Explore [Pipeline Configuration](../graphics/pipelines.md) for graphics customization
- See [Resource Management](../graphics/resources.md) for asset handling

## Common Issues

### Pipeline Creation

If you're seeing pipeline creation errors:

- Verify shader compatibility with pipeline configuration
- Check vertex attribute descriptions match shader inputs
- Ensure render pass format matches pipeline color attachments

### Performance

For optimal performance:

- Use pipeline variants for similar configurations
- Take advantage of the pipeline cache
- Group state changes efficiently
- Use dynamic state for frequently changing properties
