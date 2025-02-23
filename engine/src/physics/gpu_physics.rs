use crate::physics::memory::{BufferPool, MemoryStats};
use crate::physics::shaders::{compile_shader, ShaderModule};
use ash::{self, vk};
use std::ptr;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub use crate::physics::debug::{DebugStats, DebugVisualization};

use crate::physics::logging::{error_with_context, log_error_chain};
use std::error::Error;

#[derive(Debug)]
pub enum PhysicsError {
    DeviceLost {
        message: String,
        source: Option<Box<dyn Error + Send + Sync>>,
    },
    OutOfMemory {
        message: String,
        size: u64,
        available: u64,
    },
    InitializationFailed {
        message: String,
        component: String,
        source: Option<Box<dyn Error + Send + Sync>>,
    },
    InvalidOperation {
        message: String,
        operation: String,
        state: String,
    },
    BufferOverflow {
        message: String,
        required: u64,
        available: u64,
    },
    SynchronizationError {
        message: String,
        source: Option<Box<dyn Error + Send + Sync>>,
    },
}

impl std::error::Error for PhysicsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DeviceLost { source, .. } => source.as_ref().map(|e| e.as_ref()),
            Self::InitializationFailed { source, .. } => source.as_ref().map(|e| e.as_ref()),
            Self::SynchronizationError { source, .. } => source.as_ref().map(|e| e.as_ref()),
            _ => None,
        }
    }
}

impl std::fmt::Display for PhysicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhysicsError::DeviceLost { message, .. } => {
                write!(f, "Device Lost: {}", message)
            }
            PhysicsError::OutOfMemory {
                message,
                size,
                available,
            } => {
                write!(
                    f,
                    "Out of Memory: {} (Required: {} bytes, Available: {} bytes)",
                    message, size, available
                )
            }
            PhysicsError::InitializationFailed {
                message, component, ..
            } => {
                write!(f, "Initialization Failed for {}: {}", component, message)
            }
            PhysicsError::InvalidOperation {
                message,
                operation,
                state,
            } => {
                write!(
                    f,
                    "Invalid Operation ({}): {} - Current State: {}",
                    operation, message, state
                )
            }
            PhysicsError::BufferOverflow {
                message,
                required,
                available,
            } => {
                write!(
                    f,
                    "Buffer Overflow: {} (Required: {} bytes, Available: {} bytes)",
                    message, required, available
                )
            }
            PhysicsError::SynchronizationError { message, .. } => {
                write!(f, "Synchronization Error: {}", message)
            }
        }
    }
}

impl PhysicsError {
    pub(crate) fn log_error(&self, file: &'static str, line: u32) {
        let context = match self {
            Self::DeviceLost { .. } => "DEVICE_LOST",
            Self::OutOfMemory { .. } => "OUT_OF_MEMORY",
            Self::InitializationFailed { component, .. } => component,
            Self::InvalidOperation { operation, .. } => operation,
            Self::BufferOverflow { .. } => "BUFFER_OVERFLOW",
            Self::SynchronizationError { .. } => "SYNC_ERROR",
        };

        log_error_chain(self, context, file, line);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub position: [f32; 4], // Using vec4 for GPU alignment
    pub velocity: [f32; 4], // Using vec4 for GPU alignment
}

struct ParticleBufferPair {
    front: (vk::Buffer, vk::DeviceMemory, vk::DeviceSize),
    back: (vk::Buffer, vk::DeviceMemory, vk::DeviceSize),
    mapped_front: *mut std::ffi::c_void,
    mapped_back: *mut std::ffi::c_void,
}

pub struct ParticleDescriptorSets {
    layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    sets: Vec<vk::DescriptorSet>,
}

pub struct SynchronizationPrimitives {
    compute_fence: vk::Fence,
    compute_semaphore: vk::Semaphore,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
}

#[derive(Debug)]
pub struct SystemState {
    pub is_initialized: bool,
    pub last_error: Option<PhysicsError>,
    pub recovery_attempts: u32,
    pub needs_reset: bool,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            is_initialized: false,
            last_error: None,
            recovery_attempts: 0,
            needs_reset: false,
        }
    }
}

pub struct GpuPhysicsSystem {
    device: Arc<ash::Device>,
    physical_device: vk::PhysicalDevice,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    particle_buffers: Option<ParticleBufferPair>,
    buffer_pool: BufferPool,
    buffer_size: vk::DeviceSize,
    descriptor_sets: Option<ParticleDescriptorSets>,
    sync_primitives: Option<SynchronizationPrimitives>,
    compute_pipeline: Option<vk::Pipeline>,
    pipeline_layout: Option<vk::PipelineLayout>,
    compute_queue: vk::Queue,
    queue_family_index: u32,
    current_frame: usize,
    state: SystemState,
    max_recovery_attempts: u32,
    pub debug_enabled: bool, // Make this field public
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PushConstants {
    delta_time: f32,
    max_velocity: f32,
    bounds: [f32; 2],
}

impl GpuPhysicsSystem {
    pub fn new(
        device: Arc<ash::Device>,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<Self, PhysicsError> {
        unsafe {
            let memory_properties = device.get_physical_device_memory_properties(physical_device);
            let compute_queue = device.get_device_queue(queue_family_index, 0);

            // Create buffer pool with initial size
            let initial_pool_size = 1024 * 1024; // 1MB initial size
            let buffer_pool =
                BufferPool::new(device.clone(), queue_family_index, initial_pool_size)?;

            Ok(Self {
                device,
                physical_device,
                memory_properties,
                particle_buffers: None,
                buffer_pool,
                buffer_size: 0,
                descriptor_sets: None,
                sync_primitives: None,
                compute_pipeline: None,
                pipeline_layout: None,
                compute_queue,
                queue_family_index,
                current_frame: 0,
                state: SystemState::default(),
                max_recovery_attempts: 3,
                debug_enabled: false,
            })
        }
    }

    pub fn initialize(
        &mut self,
        particle_count: usize,
        shader_module: ShaderModule,
    ) -> Result<(), PhysicsError> {
        use crate::physics::logging::info_with_context;

        info_with_context!(
            "INIT",
            "Initializing GPU Physics System with {} particles",
            particle_count
        );

        if self.state.needs_reset {
            info_with_context!("RECOVERY", "System needs reset, attempting recovery");
            self.try_recover()?;
        }

        let buffer_size = (particle_count * std::mem::size_of::<Particle>()) as u64;
        info_with_context!(
            "MEMORY",
            "Allocating particle buffers with size: {} bytes",
            buffer_size
        );

        // Create particle buffers using buffer pool
        let (front_buffer, front_memory, front_offset) = self.buffer_pool.allocate_buffer(
            buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        )?;

        let (back_buffer, back_memory, back_offset) = self.buffer_pool.allocate_buffer(
            buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        )?;

        // Map buffers
        unsafe {
            let front_ptr = self
                .device
                .map_memory(
                    front_memory,
                    front_offset,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| {
                    error_with_context!(
                        "MEMORY",
                        "Failed to map front buffer memory at offset {}: {}",
                        front_offset,
                        e
                    );
                    PhysicsError::InitializationFailed {
                        message: format!("Failed to map front buffer memory: {}", e),
                        component: "BufferMapping".to_string(),
                        source: Some(Box::new(e)),
                    }
                })?;

            let back_ptr = self
                .device
                .map_memory(
                    back_memory,
                    back_offset,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| {
                    error_with_context!(
                        "MEMORY",
                        "Failed to map back buffer memory at offset {}: {}",
                        back_offset,
                        e
                    );
                    PhysicsError::InitializationFailed {
                        message: format!("Failed to map back buffer memory: {}", e),
                        component: "BufferMapping".to_string(),
                        source: Some(Box::new(e)),
                    }
                })?;

            self.particle_buffers = Some(ParticleBufferPair {
                front: (front_buffer, front_memory, front_offset),
                back: (back_buffer, back_memory, back_offset),
                mapped_front: front_ptr,
                mapped_back: back_ptr,
            });
        }

        self.buffer_size = buffer_size;

        // Create rest of resources
        self.create_descriptor_sets()?;
        self.create_compute_pipeline(shader_module)?;
        self.create_sync_primitives()?;

        self.state.is_initialized = true;
        Ok(())
    }

    fn create_compute_pipeline(&mut self, shader_module: ShaderModule) -> Result<(), PhysicsError> {
        use crate::physics::logging::{debug_with_context, info_with_context};

        info_with_context!("PIPELINE", "Creating compute pipeline");

        // Create pipeline layout
        debug_with_context!("PIPELINE", "Building pipeline layout with push constants");
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&self.descriptor_sets.as_ref().unwrap().layout)
            .push_constant_ranges(&[vk::PushConstantRange {
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                offset: 0,
                size: std::mem::size_of::<PushConstants>() as u32,
            }])
            .build();

        let pipeline_layout = unsafe {
            self.device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| {
                    error_with_context!("PIPELINE", "Failed to create pipeline layout: {}", e);
                    PhysicsError::InitializationFailed {
                        message: format!("Failed to create pipeline layout: {}", e),
                        component: "PipelineLayout".to_string(),
                        source: Some(Box::new(e)),
                    }
                })?
        };

        info_with_context!("PIPELINE", "Pipeline layout created successfully");
        self.pipeline_layout = Some(pipeline_layout);

        // Create compute pipeline
        let shader_entry_name = std::ffi::CString::new("main").unwrap();

        debug_with_context!("SHADER", "Configuring shader compilation options");
        let mut compile_options = shaderc::CompileOptions::new().unwrap();
        if self.debug_enabled {
            debug_with_context!("SHADER", "Debug mode enabled, adding DEBUG macro");
            compile_options.add_macro_definition("DEBUG", Some("1"));
        }

        info_with_context!("SHADER", "Compiling particle update compute shader");
        let spirv_code = match compile_shader(
            include_str!("shaders/particle_update.comp"),
            shaderc::ShaderKind::Compute,
            "main",
            Some(&compile_options),
        ) {
            Ok(code) => code,
            Err(e) => {
                error_with_context!("SHADER", "Failed to compile compute shader: {}", e);
                return Err(PhysicsError::InitializationFailed {
                    message: format!("Failed to compile compute shader: {}", e),
                    component: "ShaderCompilation".to_string(),
                    source: Some(Box::new(e)),
                });
            }
        };

        info_with_context!("SHADER", "Creating shader module from SPIR-V code");
        let shader_module = match ShaderModule::new(self.device.clone(), &spirv_code) {
            Ok(module) => module,
            Err(e) => {
                error_with_context!("SHADER", "Failed to create shader module: {}", e);
                return Err(PhysicsError::InitializationFailed {
                    message: format!("Failed to create shader module: {}", e),
                    component: "ShaderModule".to_string(),
                    source: Some(Box::new(e)),
                });
            }
        };

        let shader_stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module.get_module())
            .name(&shader_entry_name);

        // Set specialization constants for workgroup size
        let specialization_map_entries = [
            vk::SpecializationMapEntry {
                constant_id: 0,
                offset: 0,
                size: std::mem::size_of::<u32>(),
            },
            vk::SpecializationMapEntry {
                constant_id: 1,
                offset: std::mem::size_of::<u32>() as u32,
                size: std::mem::size_of::<u32>(),
            },
            vk::SpecializationMapEntry {
                constant_id: 2,
                offset: 2 * std::mem::size_of::<u32>() as u32,
                size: std::mem::size_of::<u32>(),
            },
        ];

        let workgroup_size_x = 256u32;
        let workgroup_size_y = 1u32;
        let workgroup_size_z = 1u32;

        let specialization_data: [u32; 3] = [workgroup_size_x, workgroup_size_y, workgroup_size_z];

        let specialization_info = vk::SpecializationInfo::builder()
            .map_entries(&specialization_map_entries)
            .data(unsafe {
                std::slice::from_raw_parts(
                    specialization_data.as_ptr() as *const u8,
                    specialization_data.len() * std::mem::size_of::<u32>(),
                )
            });

        let shader_stage_info = shader_stage_info
            .specialization_info(&specialization_info)
            .build();

        let pipeline_info = vk::ComputePipelineCreateInfo::builder()
            .layout(pipeline_layout)
            .stage(shader_stage_info)
            .build();

        let compute_pipeline = unsafe {
            self.device
                .create_compute_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|e| {
                    PhysicsError::InitializationFailed(format!(
                        "Failed to create compute pipeline: {:?}",
                        e
                    ))
                })?[0]
        };

        self.compute_pipeline = Some(compute_pipeline);

        Ok(())
    }

    pub fn resize(&mut self, new_particle_count: usize) -> Result<(), PhysicsError> {
        let new_size = (new_particle_count * std::mem::size_of::<Particle>()) as u64;

        // Free old buffers
        if let Some(buffers) = &self.particle_buffers {
            self.buffer_pool
                .free_buffer(buffers.front.0, buffers.front.1, buffers.front.2);
            self.buffer_pool
                .free_buffer(buffers.back.0, buffers.back.1, buffers.back.2);
        }

        // Allocate new buffers
        let (front_buffer, front_memory, front_offset) = self.buffer_pool.allocate_buffer(
            new_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        )?;

        let (back_buffer, back_memory, back_offset) = self.buffer_pool.allocate_buffer(
            new_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        )?;

        // Map new buffers
        unsafe {
            let front_ptr = self
                .device
                .map_memory(
                    front_memory,
                    front_offset,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| {
                    PhysicsError::InitializationFailed(format!(
                        "Failed to map front buffer memory: {}",
                        e
                    ))
                })?;

            let back_ptr = self
                .device
                .map_memory(
                    back_memory,
                    back_offset,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| {
                    PhysicsError::InitializationFailed(format!(
                        "Failed to map back buffer memory: {}",
                        e
                    ))
                })?;

            self.particle_buffers = Some(ParticleBufferPair {
                front: (front_buffer, front_memory, front_offset),
                back: (back_buffer, back_memory, back_offset),
                mapped_front: front_ptr,
                mapped_back: back_ptr,
            });
        }

        self.buffer_size = new_size;

        // Update descriptor sets
        self.update_descriptor_sets()?;

        Ok(())
    }

    // ... [Previous implementations with updated error handling] ...
    pub fn get_memory_stats(&self) -> MemoryStats {
        self.buffer_pool.get_memory_stats()
    }

    pub fn cleanup(&mut self) {
        if let Some(buffers) = &self.particle_buffers {
            unsafe {
                // Unmap memory
                self.device.unmap_memory(buffers.front.1);
                self.device.unmap_memory(buffers.back.1);
            }
        }

        // Cleanup buffer pool
        self.buffer_pool.cleanup();

        // ... [Rest of cleanup] ...
    }
}

impl Drop for GpuPhysicsSystem {
    fn drop(&mut self) {
        self.cleanup();
    }
}
