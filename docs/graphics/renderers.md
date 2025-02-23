# Renderer Components

Ashengine provides a flexible system of renderer components that integrate the graphics system with the ECS architecture. These components handle different types of renderable entities and their specific requirements.

## Base Renderer

All renderer types share common functionality through the `BaseRenderer`:

```rust
pub struct BaseRenderer {
    pub visible: bool,
    pub layer: i32,
    pub pass_type: PassType,
    pub material: Option<ResourceHandle>,
    pub material_params: Vec<(String, MaterialParam)>,
}
```

This provides:

- Visibility control
- Render layer ordering
- Pass type specification
- Material handling
- Custom parameter support

## Renderer Types

### 1. Static Mesh Renderer

For rendering non-animated 3D objects:

```rust
// Basic usage
let renderer = StaticMeshRenderer::new(mesh_handle)
    .with_material(material_handle)
    .with_layer(0)
    .with_pass_type(PassType::Geometry);

// Add to an entity
world.spawn((
    Transform::default(),
    renderer,
));

// With factory
let renderer = factory.create_static_mesh(StaticMeshConfig {
    mesh: mesh_handle,
    albedo_texture: Some(texture_handle),
    normal_texture: Some(normal_handle),
    metallic_roughness_texture: Some(mr_handle),
    enable_culling: true,
    cast_shadows: true,
});
```

### 2. Skinned Mesh Renderer

For animated characters and objects:

```rust
// Basic usage
let renderer = SkinnedMeshRenderer::new(mesh_handle, skeleton_handle)
    .with_material(material_handle)
    .with_animation(animation_handle);

// With factory
let renderer = factory.create_skinned_mesh(SkinnedMeshConfig {
    mesh: mesh_handle,
    skeleton: skeleton_handle,
    albedo_texture: Some(texture_handle),
    normal_texture: Some(normal_handle),
    cast_shadows: true,
});

// Update animation
if let Ok(mut renderer) = world.get_component_mut::<SkinnedMeshRenderer>(entity) {
    renderer.animation_time += delta_time;
}
```

### 3. Particle Renderer

For particle systems:

```rust
// Basic usage
let renderer = ParticleRenderer::new(particle_buffer, 1000)
    .with_material(particle_material);

// With factory
let renderer = factory.create_particle_system(ParticleConfig {
    max_particles: 1000,
    texture: Some(particle_texture),
    additive_blending: true,
});

// Update particles
if let Ok(mut renderer) = world.get_component_mut::<ParticleRenderer>(entity) {
    renderer.active_particles = compute_active_particles();
}
```

### 4. UI Renderer

For user interface elements:

```rust
// Basic usage
let renderer = UIRenderer::new(quad_mesh)
    .with_material(ui_material)
    .with_uv_rect(0.0, 0.0, 1.0, 1.0)
    .with_color(1.0, 1.0, 1.0, 1.0);

// With factory
let renderer = factory.create_ui_element(UIConfig {
    texture: Some(ui_texture),
    color: [1.0, 1.0, 1.0, 1.0],
    layer: 10,
});
```

## Render System Integration

The render system automatically processes all renderer components:

```rust
pub struct RenderSystem {
    graphics: Arc<GraphicsRenderSystem>,
    resource_manager: Arc<ResourceManager>,
    batches: HashMap<PassType, PassBatches>,
    frustum_culling: bool,
}
```

### Batching Process

1. **Collection**:

```rust
// Renderers are collected by pass type
for (_, (transform, renderer)) in world.query::<(&TransformComponent, &StaticMeshRenderer)>() {
    if renderer.base.visible {
        let pass_batches = batches.entry(renderer.base.pass_type)
            .or_insert_with(PassBatches::default);
        // Add to appropriate batch
    }
}
```

2. **Sorting**:

```rust
// Batches are sorted by:
// 1. Material (minimize state changes)
// 2. Layer (render order)
batches.sort_by_key(|(material, entities)| {
    (*material, entities.first().map(|(_, r)| r.base.layer).unwrap_or(0))
});
```

3. **Execution**:

```rust
// Batches are executed in order:
// 1. Geometry pass batches
// 2. Lighting pass batches
// 3. Post-process batches
// 4. UI batches
```

## Performance Considerations

### 1. Batching

- Group similar materials
- Use consistent mesh formats
- Minimize material variations

```rust
// Good: Similar materials grouped
world.spawn((
    Transform::new(),
    StaticMeshRenderer::new(mesh)
        .with_material(shared_material)
));

// Bad: Unique material per entity
world.spawn((
    Transform::new(),
    StaticMeshRenderer::new(mesh)
        .with_material(unique_material)
));
```

### 2. Visibility Culling

```rust
// Enable frustum culling
renderer.enable_culling = true;

// Update visibility
if let Ok(mut renderer) = world.get_component_mut::<StaticMeshRenderer>(entity) {
    renderer.base.visible = is_in_view(transform);
}
```

### 3. Layer Management

```rust
// Organize layers consistently
const BACKGROUND_LAYER: i32 = 0;
const WORLD_LAYER: i32 = 10;
const EFFECT_LAYER: i32 = 20;
const UI_LAYER: i32 = 30;

// Apply to renderers
renderer.base.layer = WORLD_LAYER;
```

## Best Practices

### 1. Component Organization

```rust
// Group related components
world.spawn((
    Transform::default(),
    StaticMeshRenderer::new(mesh),
    PhysicsBody::new(),
    Collider::new(),
));
```

### 2. Resource Management

```rust
// Share resources when possible
let shared_material = factory.create_pbr_material(config);

// Multiple entities, same material
for position in positions {
    world.spawn((
        Transform::from_position(position),
        StaticMeshRenderer::new(mesh)
            .with_material(shared_material),
    ));
}
```

### 3. Update Patterns

```rust
// Batch updates
for (_, (transform, renderer)) in world.query_mut::<(&mut Transform, &mut StaticMeshRenderer)>() {
    // Update transform
    transform.update(delta_time);

    // Update renderer only if necessary
    if transform.is_dirty() {
        renderer.update_transform(transform);
    }
}
```

### 4. Memory Considerations

```rust
// Use appropriate component sizes
let small_renderer = StaticMeshRenderer::new(mesh);  // Basic rendering
let complex_renderer = SkinnedMeshRenderer::new(     // More memory, more features
    mesh,
    skeleton,
);

// Clean up unused resources
world.despawn(entity);  // Automatically cleans up renderer
```
