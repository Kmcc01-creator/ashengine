//! Renderer component for ECS integration
//!
//! Provides a unified renderer component that works with the graphics system.

use crate::graphics::{render::PassType, resource::ResourceHandle};

/// Base renderer component shared by all renderer types
#[derive(Debug, Clone)]
pub struct RenderComponent {
    /// Whether the entity should be rendered
    pub visible: bool,
    /// Render layer for sorting
    pub layer: i32,
    /// Pass type this renderer should be processed in
    pub pass_type: PassType,
    /// Mesh resource handle
    pub mesh: ResourceHandle,
    /// Material handle
    pub material: Option<ResourceHandle>,
    /// Transform buffer handle
    pub transform_buffer: ResourceHandle,
    /// Enable frustum culling
    pub enable_culling: bool,
    /// UI Color
    pub color: [f32; 4],
}

impl RenderComponent {
    /// Create a new renderer component
    pub fn new(mesh: ResourceHandle, transform_buffer: ResourceHandle) -> Self {
        Self {
            visible: true,
            layer: 0,
            pass_type: PassType::Geometry,
            mesh,
            material: None,
            transform_buffer,
            enable_culling: true,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Set the material for this renderer
    pub fn with_material(mut self, material: ResourceHandle) -> Self {
        self.material = Some(material);
        self
    }

    /// Set the render layer
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    /// Set the pass type
    pub fn with_pass_type(mut self, pass_type: PassType) -> Self {
        self.pass_type = pass_type;
        self
    }

    ///  Set whether frustum culling is enabled
    pub fn with_culling(mut self, enable: bool) -> Self {
        self.enable_culling = enable;
        self
    }

    /// Set the color
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }

    /// Check if this renderer should be processed
    pub fn should_render(&self) -> bool {
        self.visible
    }

    /// Get the mesh resource
    pub fn mesh(&self) -> ResourceHandle {
        self.mesh
    }

    /// Get the material resource
    pub fn material(&self) -> Option<ResourceHandle> {
        self.material
    }

    /// Get the transform buffer
    pub fn transform_buffer(&self) -> ResourceHandle {
        self.transform_buffer
    }

    /// Get the pass type
    pub fn pass_type(&self) -> PassType {
        self.pass_type
    }

    /// Get the sort key for ordering within a pass
    pub fn sort_key(&self) -> i32 {
        self.layer
    }

    /// Check if frustum culling is enabled
    pub fn culling_enabled(&self) -> bool {
        self.enable_culling
    }

    /// Get the color
    pub fn color(&self) -> [f32; 4] {
        self.color
    }
}
