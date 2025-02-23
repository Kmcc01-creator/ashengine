//! Render component for graphics data
//!
//! Provides rendering information for entities, acting as a bridge
//! between the ECS and the graphics system.

use ash::vk;
use std::sync::Arc;

use super::Component;
use crate::graphics::Pipeline;

/// Component for entity rendering data
#[derive(Debug)]
pub struct RenderComponent {
    /// Mesh buffer containing vertex data
    pub mesh_buffer: Arc<vk::Buffer>,
    /// Index buffer for the mesh
    pub index_buffer: Option<Arc<vk::Buffer>>,
    /// Pipeline to use for rendering
    pub pipeline: Arc<Pipeline>,
    /// Whether the entity is visible
    pub visible: bool,
    /// Custom shader parameters
    pub shader_data: Option<Vec<u8>>,
    /// Layer for render ordering
    pub render_layer: i32,
}

impl RenderComponent {
    /// Create a new render component
    pub fn new(mesh_buffer: Arc<vk::Buffer>, pipeline: Arc<Pipeline>) -> Self {
        Self {
            mesh_buffer,
            index_buffer: None,
            pipeline,
            visible: true,
            shader_data: None,
            render_layer: 0,
        }
    }

    /// Set the index buffer
    pub fn with_index_buffer(mut self, index_buffer: Arc<vk::Buffer>) -> Self {
        self.index_buffer = Some(index_buffer);
        self
    }

    /// Set custom shader data
    pub fn with_shader_data(mut self, data: Vec<u8>) -> Self {
        self.shader_data = Some(data);
        self
    }

    /// Set the render layer
    pub fn with_render_layer(mut self, layer: i32) -> Self {
        self.render_layer = layer;
        self
    }

    /// Check if the entity should be rendered
    pub fn should_render(&self) -> bool {
        self.visible
    }
}

impl Component for RenderComponent {}

// Bridge implementation for graphics system integration
impl RenderComponent {
    /// Convert to render data for the graphics system
    pub(crate) fn to_render_data(&self) -> crate::graphics::RenderData {
        crate::graphics::RenderData {
            mesh_buffer: Arc::clone(&self.mesh_buffer),
            index_buffer: self.index_buffer.clone(),
            pipeline: Arc::clone(&self.pipeline),
            shader_data: self.shader_data.clone(),
            render_layer: self.render_layer,
        }
    }
}
