# Graphics System

The Ashengine graphics system is built on Vulkan and provides a flexible, high-performance rendering solution with full ECS integration.

## Architecture Overview

The graphics system is organized into several key subsystems:

```
graphics/
├── resource/       # Resource management
│   ├── buffers
│   ├── textures
│   ├── materials
│   └── shaders
├── render/         # Render pipeline
│   ├── graph
│   ├── pipeline
│   └── passes
├── ecs/           # ECS integration
│   ├── components
│   └── systems
└── archetypes/    # Standard configurations
```

## Key Features

- **Render Graph System**: Multi-pass rendering with automatic dependency resolution
- **Material System**: PBR-based material pipeline with custom parameter support
- **Resource Management**: Handle-based resource system with automatic lifecycle management
- **ECS Integration**: Efficient component-based rendering with batching support
- **Multiple Renderer Types**: Support for static meshes, skinned meshes, particles, and UI

## Render Pipeline

The rendering pipeline uses a graph-based approach that enables:

1. **Deferred Rendering**: G-buffer based rendering for efficient lighting
2. **Multiple Passes**: Support for geometry, lighting, post-process, and UI passes
3. **Automatic Synchronization**: Dependency-based resource transitions
4. **Efficient State Management**: Batching and state sorting for optimal performance

### Pass Types

- **Geometry Pass**: Writes position, normal, albedo, and material properties
- **Lighting Pass**: Processes G-buffer for final lighting calculations
- **Post-Process Pass**: Applies screen-space effects
- **UI Pass**: Renders 2D interface elements

## Resource Management

Resources are managed through a handle-based system that provides:

- **Type Safety**: Resource handles are typed to prevent misuse
- **Automatic Cleanup**: Resources are automatically cleaned up when no longer referenced
- **Efficient Access**: Fast lookup of resource data through handle indirection
- **State Tracking**: Resource state and usage tracking for synchronization

## ECS Integration

The graphics system integrates with the ECS through:

1. **Renderer Components**:

   - `StaticMeshRenderer`: For basic 3D objects
   - `SkinnedMeshRenderer`: For animated characters
   - `ParticleRenderer`: For particle systems
   - `UIRenderer`: For interface elements

2. **Render System**:

   - Batches similar renderers
   - Sorts by material and render layer
   - Handles frustum culling
   - Manages render state transitions

3. **Resource Components**:
   - Material parameters
   - Transform data
   - Custom shader data

## Performance Features

- **Batching**: Similar meshes are batched to reduce draw calls
- **State Sorting**: Renders are sorted to minimize state changes
- **Resource Pooling**: Resources are pooled and reused when possible
- **Async Loading**: Resources can be loaded asynchronously
- **Culling**: Automatic frustum culling of invisible objects

## Usage Examples

Basic static mesh rendering:

```rust
// Create a static mesh renderer
let renderer = factory.create_static_mesh(StaticMeshConfig {
    mesh: mesh_handle,
    albedo_texture: Some(texture_handle),
    normal_texture: Some(normal_handle),
    enable_culling: true,
    cast_shadows: true,
});

// Add to an entity
world.spawn((
    Transform::default(),
    renderer,
));
```

Particle system setup:

```rust
// Create a particle system
let particles = factory.create_particle_system(ParticleConfig {
    max_particles: 1000,
    texture: Some(particle_texture),
    additive_blending: true,
});

// Add to an entity
world.spawn((
    Transform::from_position(position),
    particles,
));
```

See the individual subsystem documentation for more detailed information and examples.

## Further Reading

- [Render Graph](render_graph.md): Detailed render graph documentation
- [Materials](materials.md): Material system documentation
- [Renderers](renderers.md): Renderer component documentation
- [Resources](resources.md): Resource management documentation
