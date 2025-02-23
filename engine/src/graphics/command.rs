//! Render command system for graphics operations
//!
//! Provides an abstraction layer between ECS and graphics systems
//! for better separation of concerns.

use super::resource::ResourceHandle;

/// Type of render resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Mesh,
    Material,
    Texture,
    Pipeline,
    Buffer,
}

/// Operation to perform with a render resource
#[derive(Debug, Clone)]
pub enum RenderOperation {
    Draw {
        mesh: ResourceHandle,
        material: Option<ResourceHandle>,
        instance_count: u32,
    },
    UpdateBuffer {
        buffer: ResourceHandle,
        data: Vec<u8>,
        offset: u64,
    },
    SetPipeline(ResourceHandle),
    BindMaterial(ResourceHandle),
}

/// Command for the render system
#[derive(Debug)]
pub struct RenderCommand {
    pub pass_type: super::render::PassType,
    pub operation: RenderOperation,
    pub sort_key: i32, // For ordering within pass
}

/// Batch of render commands for a frame
#[derive(Debug, Default)]
pub struct CommandBatch {
    pub commands: Vec<RenderCommand>,
}

impl CommandBatch {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn add(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn sort(&mut self) {
        // Sort by pass type first, then by sort key
        self.commands
            .sort_by_key(|cmd| (cmd.pass_type, cmd.sort_key));
    }
}
