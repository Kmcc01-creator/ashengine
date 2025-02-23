use crate::error::{Result, VulkanError};
use ash::{vk, Device};
use std::sync::Arc;

pub struct CommandPool {
    pool: vk::CommandPool,
    device: Arc<Device>,
}

impl CommandPool {
    pub fn new(
        device: Arc<Device>,
        queue_family_index: u32,
        flags: vk::CommandPoolCreateFlags,
    ) -> Result<Self> {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(flags);

        let pool = unsafe {
            device
                .create_command_pool(&create_info, None)
                .map_err(|e| VulkanError::CommandPoolCreation(e.to_string()))?
        };

        Ok(Self { pool, device })
    }

    pub fn allocate_buffers(
        &self,
        level: vk::CommandBufferLevel,
        count: u32,
    ) -> Result<Vec<CommandBuffer>> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .level(level)
            .command_buffer_count(count);

        let buffers = unsafe {
            self.device
                .allocate_command_buffers(&alloc_info)
                .map_err(|e| VulkanError::CommandPoolCreation(e.to_string()))?
        };

        Ok(buffers
            .into_iter()
            .map(|buffer| CommandBuffer {
                buffer,
                pool: self.pool,
                device: self.device.clone(),
                state: CommandBufferState::Initial,
            })
            .collect())
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_command_pool(self.pool, None);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum CommandBufferState {
    Initial,
    Recording,
    Executable,
    Pending,
    #[allow(dead_code)]
    // Reserved for future error handling (e.g., device loss scenarios)
    Invalid,
}

pub struct CommandBuffer {
    buffer: vk::CommandBuffer,
    pool: vk::CommandPool,
    device: Arc<Device>,
    state: CommandBufferState,
}

impl CommandBuffer {
    pub fn begin(&mut self, flags: vk::CommandBufferUsageFlags) -> Result<CommandBufferRecording> {
        if self.state != CommandBufferState::Initial {
            return Err(VulkanError::ValidationError(
                "Command buffer must be in initial state to begin recording".to_string(),
            ));
        }

        let begin_info = vk::CommandBufferBeginInfo::builder().flags(flags);

        unsafe {
            self.device
                .begin_command_buffer(self.buffer, &begin_info)
                .map_err(|e| VulkanError::ValidationError(e.to_string()))?;
        }

        self.state = CommandBufferState::Recording;

        Ok(CommandBufferRecording { cmd: self })
    }

    pub fn reset(&mut self, release_resources: bool) -> Result<()> {
        let flags = if release_resources {
            vk::CommandBufferResetFlags::RELEASE_RESOURCES
        } else {
            vk::CommandBufferResetFlags::empty()
        };

        unsafe {
            self.device
                .reset_command_buffer(self.buffer, flags)
                .map_err(|e| VulkanError::ValidationError(e.to_string()))?;
        }

        self.state = CommandBufferState::Initial;
        Ok(())
    }

    pub fn submit(
        &mut self,
        queue: vk::Queue,
        wait_semaphores: &[vk::Semaphore],
        wait_stages: &[vk::PipelineStageFlags],
        signal_semaphores: &[vk::Semaphore],
        fence: vk::Fence,
    ) -> Result<()> {
        if self.state != CommandBufferState::Executable {
            return Err(VulkanError::ValidationError(
                "Command buffer must be executable to submit".to_string(),
            ));
        }

        let command_buffers = [self.buffer];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores);

        unsafe {
            self.device
                .queue_submit(queue, &[submit_info.build()], fence)
                .map_err(|e| VulkanError::ValidationError(e.to_string()))?;
        }

        self.state = CommandBufferState::Pending;
        Ok(())
    }

    // Returns the raw Vulkan command buffer handle
    pub fn handle(&self) -> vk::CommandBuffer {
        self.buffer
    }
}

// RAII guard for command buffer recording
pub struct CommandBufferRecording<'a> {
    cmd: &'a mut CommandBuffer,
}

impl<'a> CommandBufferRecording<'a> {
    pub fn end(self) -> Result<()> {
        unsafe {
            self.cmd
                .device
                .end_command_buffer(self.cmd.buffer)
                .map_err(|e| VulkanError::ValidationError(e.to_string()))?;
        }

        self.cmd.state = CommandBufferState::Executable;
        Ok(())
    }

    // Returns the raw Vulkan command buffer handle
    pub fn handle(&self) -> vk::CommandBuffer {
        self.cmd.buffer
    }

    // Delegate command buffer functions
    pub fn bind_pipeline(&mut self, bind_point: vk::PipelineBindPoint, pipeline: vk::Pipeline) {
        unsafe {
            self.cmd
                .device
                .cmd_bind_pipeline(self.cmd.buffer, bind_point, pipeline);
        }
    }

    pub fn bind_vertex_buffers(
        &mut self,
        first_binding: u32,
        buffers: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        unsafe {
            self.cmd.device.cmd_bind_vertex_buffers(
                self.cmd.buffer,
                first_binding,
                buffers,
                offsets,
            );
        }
    }

    pub fn bind_index_buffer(
        &mut self,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
        unsafe {
            self.cmd
                .device
                .cmd_bind_index_buffer(self.cmd.buffer, buffer, offset, index_type);
        }
    }

    pub fn begin_render_pass(
        &mut self,
        render_pass_begin: &vk::RenderPassBeginInfo,
        contents: vk::SubpassContents,
    ) {
        unsafe {
            self.cmd
                .device
                .cmd_begin_render_pass(self.cmd.buffer, render_pass_begin, contents);
        }
    }

    pub fn end_render_pass(&mut self) {
        unsafe {
            self.cmd.device.cmd_end_render_pass(self.cmd.buffer);
        }
    }

    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.cmd.device.cmd_draw(
                self.cmd.buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            self.cmd.device.cmd_draw_indexed(
                self.cmd.buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }
}
