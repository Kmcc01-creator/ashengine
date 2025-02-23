# Material System

The material system provides a flexible and efficient way to manage shader parameters, textures, and render states. It's designed to work seamlessly with both the ECS and the render graph system.

## Overview

Materials in Ashengine are resource-managed objects that encapsulate:

- Shader parameters
- Texture references
- Render states
- Pipeline configurations

## Material Parameters

### Parameter Types

```rust
pub enum MaterialParam {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Int(i32),
    UInt(u32),
    Bool(bool),
    TextureHandle(ResourceHandle),
}
```

### Usage Example

```rust
// Create a material with parameters
let material = MaterialDescriptor {
    name: "Metal".into(),
    shader_handle: pbr_shader,
    parameters: {
        let mut params = HashMap::new();
        params.insert("albedo".into(), MaterialParam::Vec3([0.7, 0.7, 0.7]));
        params.insert("metallic".into(), MaterialParam::Float(1.0));
        params.insert("roughness".into(), MaterialParam::Float(0.3));
        params.insert("albedoMap".into(), MaterialParam::TextureHandle(texture));
        params
    },
};
```

## Material Types

### 1. PBR Materials

Standard physically-based rendering materials:

```rust
// Create a PBR material
let metal = factory.create_pbr_material(PbrMaterialConfig {
    albedo: [0.7, 0.7, 0.7],
    metallic: 1.0,
    roughness: 0.3,
    albedo_texture: Some(texture_handle),
    normal_texture: Some(normal_handle),
    metallic_roughness_texture: Some(mr_handle),
});
```

### 2. Skinned Materials

Materials for animated meshes:

```rust
// Create a skinned material
let character = factory.create_skinned_material(SkinnedMaterialConfig {
    albedo_texture: Some(diffuse_handle),
    normal_texture: Some(normal_handle),
    max_bones: 64,
});
```

### 3. Particle Materials

Materials for particle systems:

```rust
// Create a particle material
let particles = factory.create_particle_material(ParticleMaterialConfig {
    texture: Some(particle_texture),
    additive_blending: true,
    soft_particles: true,
});
```

### 4. UI Materials

Materials for user interface elements:

```rust
// Create a UI material
let ui = factory.create_ui_material(UiMaterialConfig {
    texture: Some(ui_texture),
    color: [1.0, 1.0, 1.0, 1.0],
    layer: 0,
});
```

## Material System Architecture

### Resource Management

Materials are managed through the resource system:

- Automatic cleanup when no longer referenced
- Efficient GPU resource allocation
- Descriptor set management
- Pipeline state caching

```rust
// Resource handle for materials
pub struct MaterialHandle(ResourceHandle);

// Material resource management
impl ResourceManager {
    pub fn create_material(&self, descriptor: MaterialDescriptor) -> Result<MaterialHandle>;
    pub fn get_material(&self, handle: MaterialHandle) -> Option<&Material>;
    pub fn update_material(&self, handle: MaterialHandle, params: &[(&str, MaterialParam)]);
}
```

### Descriptor Management

Materials automatically manage their descriptor sets:

- Uniform buffer allocation
- Texture binding
- Dynamic updates

```rust
impl Material {
    // Update material parameters
    pub fn set_parameter(&mut self, name: &str, value: MaterialParam);

    // Get descriptor set for rendering
    pub fn descriptor_set(&self) -> vk::DescriptorSet;

    // Get pipeline layout
    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout;
}
```

## ECS Integration

### Component Usage

```rust
// Add material to a static mesh renderer
world.spawn((
    Transform::default(),
    StaticMeshRenderer::new(mesh_handle)
        .with_material(material_handle)
        .with_layer(0),
));

// Update material parameters
if let Ok(mut renderer) = world.get_component_mut::<StaticMeshRenderer>(entity) {
    renderer.add_parameter("roughness", MaterialParam::Float(0.5));
}
```

### Material Batching

The render system automatically batches entities by material:

- Reduces state changes
- Optimizes draw calls
- Maintains render order

## Best Practices

### 1. Resource Management

- Reuse materials when possible
- Clean up unused materials
- Use appropriate texture formats
- Consider memory usage

### 2. Performance

```rust
// Group similar materials
let material = factory.create_pbr_material(PbrMaterialConfig {
    // Use same texture formats
    albedo_texture: Some(texture1),  // R8G8B8A8_UNORM
    normal_texture: Some(texture2),  // R8G8B8A8_UNORM
    // Use consistent parameter types
    metallic: 1.0,  // Float
    roughness: 0.5, // Float
});
```

### 3. Organization

- Use consistent naming conventions
- Group related materials
- Document material requirements

### 4. Updates

```rust
// Batch parameter updates
material.update_parameters(&[
    ("albedo", MaterialParam::Vec3([1.0, 0.0, 0.0])),
    ("metallic", MaterialParam::Float(0.8)),
    ("roughness", MaterialParam::Float(0.2)),
]);
```

## Common Patterns

### Material Instancing

```rust
// Create base material
let base_material = factory.create_pbr_material(base_config);

// Create instance with overrides
let instance = factory.create_material_instance(base_material)
    .with_parameter("albedo", MaterialParam::Vec3([1.0, 0.0, 0.0]))
    .with_parameter("metallic", MaterialParam::Float(0.8));
```

### Material Arrays

```rust
// Create material array for texture arrays
let material_array = factory.create_material_array(MaterialArrayConfig {
    capacity: 16,
    descriptor: base_material,
});

// Add variations
material_array.add_variation(0, &[
    ("albedoMap", MaterialParam::TextureHandle(texture1)),
    ("normalMap", MaterialParam::TextureHandle(normal1)),
]);
```

### Dynamic Materials

```rust
// Create dynamic material for runtime updates
let dynamic = factory.create_dynamic_material(DynamicMaterialConfig {
    update_frequency: UpdateFrequency::PerFrame,
    descriptor: base_material,
});

// Update per frame
dynamic.update(&[
    ("time", MaterialParam::Float(time)),
    ("color", MaterialParam::Vec3(color)),
]);
```
