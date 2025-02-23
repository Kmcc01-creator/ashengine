//! Render graph system for managing render passes and dependencies
//!
//! Provides a flexible system for defining render passes and their dependencies,
//! with support for command-based execution and resource transitions.

use crate::{
    error::Result,
    graphics::resource::{
        Material, Mesh, Pipeline, ResourceHandle, ResourceManager, TextureFormat,
    },
};
use ash::vk;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Unique identifier for a render pass in the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PassId(usize);

/// Resource state tracking for synchronization
#[derive(Debug, Clone, Copy)]
pub struct ResourceState {
    layout: vk::ImageLayout,
    access_mask: vk::AccessFlags,
    stage_mask: vk::PipelineStageFlags,
}

/// Pass execution state
#[derive(Debug)]
struct PassState {
    current_pipeline: Option<vk::Pipeline>,
    current_layout: Option<vk::PipelineLayout>,
    command_buffer: Option<vk::CommandBuffer>,
}

/// Resource usage within a pass
#[derive(Debug, Clone)]
struct ResourceUsage {
    access_mask: vk::AccessFlags,
    stage_mask: vk::PipelineStageFlags,
    layout: vk::ImageLayout,
}

/// Resource dependency information
#[derive(Debug)]
struct ResourceDependency {
    source_pass: PassId,
    destination_pass: PassId,
    resource: ResourceHandle,
    usage: ResourceUsage,
}

/// Pass dependency information
#[derive(Debug)]
struct PassDependency {
    dependencies: Vec<ResourceDependency>,
    barriers: Vec<vk::ImageMemoryBarrier>,
}

/// The render graph that manages render passes and their dependencies
pub struct RenderGraph {
    device: Arc<ash::Device>,
    resource_manager: Arc<ResourceManager>,
    current_pass: Option<PassType>,
    pass_state: PassState,
    resource_states: HashMap<ResourceHandle, ResourceState>,
    command_pool: vk::CommandPool,
    descriptor_pool: vk::DescriptorPool,
    graphics_queue: vk::Queue,
    graphics_queue_family: u32,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
    max_frames_in_flight: usize,

    // Resource tracking
    pass_dependencies: HashMap<PassId, PassDependency>,
    resource_lifetimes: HashMap<ResourceHandle, (PassId, PassId)>, // (first_use, last_use)
    current_pass_id: usize,
}

impl RenderGraph {
    /// Create a new render graph
    pub fn new(device: Arc<ash::Device>, resource_manager: Arc<ResourceManager>) -> Result<Self> {
        const MAX_FRAMES_IN_FLIGHT: usize = 2;

        // Find graphics queue family
        let queue_family_properties = unsafe {
            device.get_physical_device_queue_family_properties(
                // TODO: Store physical device in RenderGraph or pass as parameter
                vk::PhysicalDevice::null(),
            )
        };

        let graphics_queue_family = queue_family_properties
            .iter()
            .enumerate()
            .find(|(_, properties)| properties.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|(index, _)| index as u32)
            .ok_or_else(|| vk::Result::ERROR_INITIALIZATION_FAILED)?;

        // Get graphics queue
        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family, 0) };

        // Create command pool
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_queue_family)
            .build();

        let command_pool = unsafe { device.create_command_pool(&pool_info, None)? };

        // Create descriptor pool
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1000,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1000,
            },
        ];

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(1000)
            .build();

        let descriptor_pool =
            unsafe { device.create_descriptor_pool(&descriptor_pool_info, None)? };

        // Create synchronization primitives
        let mut in_flight_fences = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        let fence_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let fence = unsafe { device.create_fence(&fence_info, None)? };
            in_flight_fences.push(fence);
        }

        Ok(Self {
            device,
            resource_manager,
            current_pass: None,
            pass_state: PassState {
                current_pipeline: None,
                current_layout: None,
                command_buffer: None,
            },
            resource_states: HashMap::new(),
            command_pool,
            descriptor_pool,
            graphics_queue,
            graphics_queue_family,
            in_flight_fences,
            current_frame: 0,
            max_frames_in_flight: MAX_FRAMES_IN_FLIGHT,
            pass_dependencies: HashMap::new(),
            resource_lifetimes: HashMap::new(),
            current_pass_id: 0,
        })
    }

    /// Begin a render pass
    pub fn begin_pass(&mut self, pass_type: PassType) -> Result<()> {
        // End current pass if one is active
        if self.current_pass.is_some() {
            self.end_pass()?;
        }

        // Increment pass ID for new pass
        self.current_pass_id += 1;

        // Calculate and insert barriers for the new pass
        self.calculate_pass_barriers()?;

        // Allocate command buffer
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1)
            .build();

        let command_buffer = unsafe { self.device.allocate_command_buffers(&alloc_info)?[0] };

        // Begin command buffer
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)?;

            // Insert barriers if any exist for this pass
            if let Some(pass_dep) = self.pass_dependencies.get(&PassId(self.current_pass_id)) {
                if !pass_dep.barriers.is_empty() {
                    self.device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::DependencyFlags::empty(),
                        &[], // No memory barriers
                        &[], // No buffer barriers
                        &pass_dep.barriers,
                    );
                }
            }
        }

        self.current_pass = Some(pass_type);
        self.pass_state.command_buffer = Some(command_buffer);
        self.pass_state.current_pipeline = None;
        self.pass_state.current_layout = None;

        Ok(())
    }

    /// End the current render pass
    pub fn end_pass(&mut self) -> Result<()> {
        if let Some(command_buffer) = self.pass_state.command_buffer.take() {
            unsafe {
                // Wait for previous frame's fence
                self.device.wait_for_fences(
                    &[self.in_flight_fences[self.current_frame]],
                    true,
                    u64::MAX,
                )?;

                // Reset fence for current frame
                self.device
                    .reset_fences(&[self.in_flight_fences[self.current_frame]])?;

                // End command buffer recording
                self.device.end_command_buffer(command_buffer)?;

                // Submit command buffer with proper synchronization
                let submit_info = vk::SubmitInfo::builder()
                    .command_buffers(&[command_buffer])
                    .build();

                self.device.queue_submit(
                    self.graphics_queue,
                    &[submit_info],
                    self.in_flight_fences[self.current_frame],
                )?;

                // Update frame index
                self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;
            }
        }

        self.current_pass = None;
        Ok(())
    }

    /// Bind a pipeline
    pub fn bind_pipeline(&mut self, pipeline: vk::Pipeline) -> Result<()> {
        if let Some(command_buffer) = self.pass_state.command_buffer {
            if Some(pipeline) != self.pass_state.current_pipeline {
                unsafe {
                    self.device.cmd_bind_pipeline(
                        command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline,
                    );
                }
                self.pass_state.current_pipeline = Some(pipeline);
            }
        }
        Ok(())
    }

    /// Bind a material's resources
    pub fn bind_material(&mut self, material: &Material) -> Result<()> {
        if let Some(command_buffer) = self.pass_state.command_buffer {
            // Get descriptor sets from material
            let descriptor_sets = material.descriptor_sets();
            let pipeline_layout = material.pipeline_layout();

            if descriptor_sets.is_empty() {
                return Ok(());
            }

            // Bind descriptor sets
            unsafe {
                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0, // First set
                    descriptor_sets,
                    &[], // No dynamic offsets
                );
            }

            // Update pipeline layout state
            self.pass_state.current_layout = Some(pipeline_layout);
        }
        Ok(())
    }

    /// Draw a mesh
    pub fn draw_mesh(&mut self, mesh: &Mesh, instance_count: u32) -> Result<()> {
        if let Some(command_buffer) = self.pass_state.command_buffer {
            // Bind vertex buffers
            let vertex_buffers = mesh.vertex_buffers();
            let vertex_offsets = vec![0; vertex_buffers.len()];

            unsafe {
                self.device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0, // First binding
                    vertex_buffers,
                    &vertex_offsets,
                );
            }

            // Bind index buffer if mesh is indexed
            if let Some(index_buffer) = mesh.index_buffer() {
                unsafe {
                    self.device.cmd_bind_index_buffer(
                        command_buffer,
                        index_buffer,
                        0, // Offset
                        mesh.index_type(),
                    );

                    // Draw indexed
                    self.device.cmd_draw_indexed(
                        command_buffer,
                        mesh.index_count(),
                        instance_count,
                        0, // First index
                        0, // Vertex offset
                        0, // First instance
                    );
                }
            } else {
                // Draw non-indexed
                unsafe {
                    self.device.cmd_draw(
                        command_buffer,
                        mesh.vertex_count(),
                        instance_count,
                        0, // First vertex
                        0, // First instance
                    );
                }
            }
        }
        Ok(())
    }

    /// Update a buffer's contents
    pub fn update_buffer(&mut self, buffer: vk::Buffer, data: &[u8], offset: u64) -> Result<()> {
        if let Some(command_buffer) = self.pass_state.command_buffer {
            let size = data.len() as u64;

            // Create staging buffer
            let staging_buffer_info = vk::BufferCreateInfo::builder()
                .size(size)
                .usage(vk::BufferUsageFlags::TRANSFER_SRC)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .build();

            let staging_buffer = unsafe { self.device.create_buffer(&staging_buffer_info, None)? };

            // Allocate and map staging memory
            let memory_reqs = unsafe { self.device.get_buffer_memory_requirements(staging_buffer) };

            let memory_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_reqs.size)
                .memory_type_index(0) // TODO: Find proper memory type
                .build();

            let staging_memory = unsafe { self.device.allocate_memory(&memory_info, None)? };

            // Bind staging memory
            unsafe {
                self.device
                    .bind_buffer_memory(staging_buffer, staging_memory, 0)?;
            }

            // Copy data to staging buffer
            unsafe {
                let ptr =
                    self.device
                        .map_memory(staging_memory, 0, size, vk::MemoryMapFlags::empty())?
                        as *mut u8;

                std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

                self.device.unmap_memory(staging_memory);
            }

            // Copy from staging to destination buffer
            let copy_region = vk::BufferCopy::builder()
                .src_offset(0)
                .dst_offset(offset)
                .size(size)
                .build();

            unsafe {
                self.device
                    .cmd_copy_buffer(command_buffer, staging_buffer, buffer, &[copy_region]);
            }

            // Clean up staging resources
            unsafe {
                self.device.destroy_buffer(staging_buffer, None);
                self.device.free_memory(staging_memory, None);
            }
        }
        Ok(())
    }

    /// Register resource usage in the current pass
    fn register_resource_usage(
        &mut self,
        resource: ResourceHandle,
        usage: ResourceUsage,
    ) -> Result<()> {
        let pass_id = PassId(self.current_pass_id);

        // Update resource lifetime
        match self.resource_lifetimes.get(&resource) {
            Some(&(first_use, _)) => {
                self.resource_lifetimes
                    .insert(resource, (first_use, pass_id));
            }
            None => {
                self.resource_lifetimes.insert(resource, (pass_id, pass_id));
            }
        }

        // Add dependency if resource was used in previous passes
        if let Some(&(first_use, _)) = self.resource_lifetimes.get(&resource) {
            if first_use.0 < pass_id.0 {
                let dependency = ResourceDependency {
                    source_pass: first_use,
                    destination_pass: pass_id,
                    resource,
                    usage: usage.clone(),
                };

                self.pass_dependencies
                    .entry(pass_id)
                    .or_insert_with(|| PassDependency {
                        dependencies: Vec::new(),
                        barriers: Vec::new(),
                    })
                    .dependencies
                    .push(dependency);
            }
        }

        Ok(())
    }

    /// Track resource state changes and insert barriers
    fn transition_resource(
        &mut self,
        resource: ResourceHandle,
        new_state: ResourceState,
    ) -> Result<()> {
        // Register resource usage
        self.register_resource_usage(
            resource,
            ResourceUsage {
                access_mask: new_state.access_mask,
                stage_mask: new_state.stage_mask,
                layout: new_state.layout,
            },
        )?;

        // Perform immediate barrier if in a command buffer
        if let Some(command_buffer) = self.pass_state.command_buffer {
            if let Some(old_state) = self.resource_states.get(&resource) {
                if old_state.layout != new_state.layout
                    || old_state.access_mask != new_state.access_mask
                    || old_state.stage_mask != new_state.stage_mask
                {
                    // Get image from resource manager
                    let image = if let Some(image) = self.resource_manager.get_image(resource) {
                        image
                    } else {
                        // Resource is not an image, skip barrier
                        self.resource_states.insert(resource, new_state);
                        return Ok(());
                    };

                    // Create image memory barrier
                    let barrier = vk::ImageMemoryBarrier::builder()
                        .old_layout(old_state.layout)
                        .new_layout(new_state.layout)
                        .src_access_mask(old_state.access_mask)
                        .dst_access_mask(new_state.access_mask)
                        .src_queue_family_index(self.graphics_queue_family)
                        .dst_queue_family_index(self.graphics_queue_family)
                        .image(image)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .base_mip_level(0)
                                .level_count(1)
                                .base_array_layer(0)
                                .layer_count(1)
                                .build(),
                        )
                        .build();

                    unsafe {
                        self.device.cmd_pipeline_barrier(
                            command_buffer,
                            old_state.stage_mask,
                            new_state.stage_mask,
                            vk::DependencyFlags::empty(),
                            &[],        // No memory barriers
                            &[],        // No buffer memory barriers
                            &[barrier], // Image memory barriers
                        );
                    }
                }
            }
            self.resource_states.insert(resource, new_state);
        }
        Ok(())
    }

    /// Calculate barriers for the current pass
    fn calculate_pass_barriers(&mut self) -> Result<()> {
        let pass_id = PassId(self.current_pass_id);

        if let Some(pass_dep) = self.pass_dependencies.get_mut(&pass_id) {
            let mut barriers = Vec::new();

            for dep in &pass_dep.dependencies {
                if let Some(old_state) = self.resource_states.get(&dep.resource) {
                    let new_state = ResourceState {
                        layout: dep.usage.layout,
                        access_mask: dep.usage.access_mask,
                        stage_mask: dep.usage.stage_mask,
                    };

                    if old_state.layout != new_state.layout
                        || old_state.access_mask != new_state.access_mask
                        || old_state.stage_mask != new_state.stage_mask
                    {
                        let barrier = vk::ImageMemoryBarrier::builder()
                            .old_layout(old_state.layout)
                            .new_layout(new_state.layout)
                            .src_access_mask(old_state.access_mask)
                            .dst_access_mask(new_state.access_mask)
                            .src_queue_family_index(self.graphics_queue_family)
                            .dst_queue_family_index(self.graphics_queue_family)
                            // TODO: Get actual image from resource manager
                            .image(vk::Image::null())
                            .subresource_range(
                                vk::ImageSubresourceRange::builder()
                                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                                    .base_mip_level(0)
                                    .level_count(1)
                                    .base_array_layer(0)
                                    .layer_count(1)
                                    .build(),
                            )
                            .build();

                        barriers.push(barrier);
                    }
                }
            }

            pass_dep.barriers = barriers;
        }

        Ok(())
    }

    /// Track resource state changes and insert barriers
    fn transition_resource(
        &mut self,
        resource: ResourceHandle,
        new_state: ResourceState,
    ) -> Result<()> {
        if let Some(command_buffer) = self.pass_state.command_buffer {
            if let Some(old_state) = self.resource_states.get(&resource) {
                if old_state.layout != new_state.layout
                    || old_state.access_mask != new_state.access_mask
                    || old_state.stage_mask != new_state.stage_mask
                {
                    // Create image memory barrier
                    let barrier = vk::ImageMemoryBarrier::builder()
                        .old_layout(old_state.layout)
                        .new_layout(new_state.layout)
                        .src_access_mask(old_state.access_mask)
                        .dst_access_mask(new_state.access_mask)
                        .src_queue_family_index(self.graphics_queue_family)
                        .dst_queue_family_index(self.graphics_queue_family)
                        // TODO: Get actual image from resource manager
                        .image(vk::Image::null())
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .base_mip_level(0)
                                .level_count(1)
                                .base_array_layer(0)
                                .layer_count(1)
                                .build(),
                        )
                        .build();

                    unsafe {
                        self.device.cmd_pipeline_barrier(
                            command_buffer,
                            old_state.stage_mask,
                            new_state.stage_mask,
                            vk::DependencyFlags::empty(),
                            &[],        // No memory barriers
                            &[],        // No buffer memory barriers
                            &[barrier], // Image memory barriers
                        );
                    }
                }
            }
            self.resource_states.insert(resource, new_state);
        }
        Ok(())
    }
}

impl Drop for RenderGraph {
    fn drop(&mut self) {
        unsafe {
            // Wait for device to be idle before cleanup
            let _ = self.device.device_wait_idle();

            // Clean up all synchronization primitives
            for fence in &self.in_flight_fences {
                self.device.destroy_fence(*fence, None);
            }

            // Clean up pools
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_command_pool(self.command_pool, None);
        }
    }
}
