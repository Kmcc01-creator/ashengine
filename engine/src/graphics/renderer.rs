//! Main renderer implementation that processes render commands
//!
//! Handles the execution of render commands and manages the graphics pipeline

use std::sync::Arc;

use super::{
    command::{CommandBatch, RenderCommand, RenderOperation},
    render::{PassType, RenderGraph},
    resource::{ResourceHandle, ResourceManager},
};

use crate::error::Result;

/// Main renderer responsible for processing render commands
pub struct Renderer {
    resource_manager: Arc<ResourceManager>,
    render_graph: RenderGraph,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(device: Arc<ash::Device>, resource_manager: Arc<ResourceManager>) -> Result<Self> {
        Ok(Self {
            render_graph: RenderGraph::new(device, resource_manager.clone()),
            resource_manager,
        })
    }

    /// Submit a batch of render commands for processing
    pub fn submit_commands(&self, batch: &CommandBatch) -> Result<()> {
        let mut current_pass: Option<PassType> = None;

        for command in &batch.commands {
            // Start new render pass if needed
            if current_pass != Some(command.pass_type) {
                if let Some(pass_type) = current_pass {
                    // End previous pass
                    self.render_graph.end_pass()?;
                }
                // Begin new pass
                self.render_graph.begin_pass(command.pass_type)?;
                current_pass = Some(command.pass_type);
            }

            // Process the command
            match &command.operation {
                RenderOperation::Draw {
                    mesh,
                    material,
                    instance_count,
                } => {
                    // Bind material (if provided)
                    if let Some(material_handle) = material {
                        self.bind_material(*material_handle)?;
                    }

                    // Draw the mesh
                    self.draw_mesh(*mesh, *instance_count)?;
                }
                RenderOperation::UpdateBuffer {
                    buffer,
                    data,
                    offset,
                } => {
                    self.update_buffer(*buffer, data, *offset)?;
                }
                RenderOperation::SetPipeline(pipeline) => {
                    self.bind_pipeline(*pipeline)?;
                }
                RenderOperation::BindMaterial(material) => {
                    self.bind_material(*material)?;
                }
            }
        }

        // End the last pass if one was active
        if current_pass.is_some() {
            self.render_graph.end_pass()?;
        }

        Ok(())
    }

    // Private helper methods for command execution

    fn bind_pipeline(&self, pipeline: ResourceHandle) -> Result<()> {
        // Get pipeline from resource manager and bind it
        if let Some(pipeline) = self.resource_manager.get_pipeline(pipeline) {
            self.render_graph.bind_pipeline(pipeline)?;
        }
        Ok(())
    }

    fn bind_material(&self, material: ResourceHandle) -> Result<()> {
        // Get material from resource manager and bind its resources
        if let Some(material) = self.resource_manager.get_material(material) {
            self.render_graph.bind_material(&material)?;
        }
        Ok(())
    }

    fn draw_mesh(&self, mesh: ResourceHandle, instance_count: u32) -> Result<()> {
        // Get mesh from resource manager and draw it
        if let Some(mesh) = self.resource_manager.get_mesh(mesh) {
            self.render_graph.draw_mesh(&mesh, instance_count)?;
        }
        Ok(())
    }

    fn update_buffer(&self, buffer: ResourceHandle, data: &[u8], offset: u64) -> Result<()> {
        // Get buffer from resource manager and update its contents
        if let Some(buffer) = self.resource_manager.get_buffer(buffer) {
            self.render_graph.update_buffer(buffer, data, offset)?;
        }
        Ok(())
    }
}
