use crate::error::{Result, VulkanError};
use ash::{vk, Device};
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PushConstants {
    ray_origin: [f32; 2],
    ray_direction: [f32; 2],
}

pub struct TextPicker {
    compute_pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    device: Arc<Device>,
}

impl TextPicker {
    pub fn new(device: Arc<Device>) -> Result<Self> {
        // Create descriptor set layout for the bounding box buffer
        let binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .build();

        let bindings = [binding];
        let descriptor_layout_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&descriptor_layout_info, None)
                .map_err(|e| VulkanError::DescriptorSetLayoutCreation(e.to_string()))?
        };

        // Create descriptor pool
        let pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::STORAGE_BUFFER,
            descriptor_count: 1,
        };

        let pool_sizes = [pool_size];

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(1);

        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .map_err(|e| VulkanError::DescriptorPoolCreation(e.to_string()))?
        };

        // Allocate descriptor set
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));

        let descriptor_set = unsafe {
            device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .map_err(|e| VulkanError::DescriptorSetAllocation(e.to_string()))?[0]
        };

        // Create pipeline layout with push constants for ray data
        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .offset(0)
            .size(std::mem::size_of::<PushConstants>() as u32)
            .build();

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout))
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| VulkanError::PipelineLayoutCreation(e.to_string()))?
        };

        // Create compute pipeline
        let compute_pipeline = Self::create_compute_pipeline(&device, pipeline_layout)?;

        Ok(Self {
            compute_pipeline,
            pipeline_layout,
            descriptor_set_layout,
            descriptor_set,
            device,
        })
    }

    fn create_compute_pipeline(
        device: &Device,
        pipeline_layout: vk::PipelineLayout,
    ) -> Result<vk::Pipeline> {
        // Shader code would be loaded and created here
        // For now, we'll just create a placeholder
        let shader_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(vk::ShaderModule::null()) // TODO: Load actual shader
            .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();

        let compute_pipeline_info = vk::ComputePipelineCreateInfo::builder()
            .stage(shader_stage)
            .layout(pipeline_layout)
            .build();

        unsafe {
            device
                .create_compute_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&compute_pipeline_info),
                    None,
                )
                .map_err(|e| VulkanError::PipelineCreation(e.1.to_string()))
                .map(|pipelines| pipelines[0])
        }
    }

    pub fn test_intersection(
        &self,
        command_buffer: vk::CommandBuffer,
        bbox_buffer: vk::Buffer,
        result_buffer: vk::Buffer,
        _descriptor_set: vk::DescriptorSet,
        ray_origin: [f32; 2],
        ray_direction: [f32; 2],
        bbox_count: u32,
    ) {
        // Update descriptor set with buffer info
        let bbox_buffer_info = vk::DescriptorBufferInfo {
            buffer: bbox_buffer,
            offset: 0,
            range: vk::WHOLE_SIZE,
        };

        let result_buffer_info = vk::DescriptorBufferInfo {
            buffer: result_buffer,
            offset: 0,
            range: vk::WHOLE_SIZE,
        };

        let write_descriptor_sets = [
            vk::WriteDescriptorSet::builder()
                .dst_set(self.descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&bbox_buffer_info))
                .build(),
            vk::WriteDescriptorSet::builder()
                .dst_set(self.descriptor_set)
                .dst_binding(1) // Assuming result_buffer is bound to binding 1
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(std::slice::from_ref(&result_buffer_info))
                .build(),
        ];

        unsafe {
            self.device
                .update_descriptor_sets(&write_descriptor_sets, &[]);
        }

        let push_constants = PushConstants {
            ray_origin,
            ray_direction,
        };

        unsafe {
            // Bind compute pipeline and descriptor set
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.compute_pipeline,
            );

            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                self.pipeline_layout,
                0,
                std::slice::from_ref(&self.descriptor_set),
                &[],
            );

            // Push ray constants
            let push_constants_bytes = std::slice::from_raw_parts(
                (&push_constants as *const PushConstants) as *const u8,
                std::mem::size_of::<PushConstants>(),
            );

            self.device.cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                push_constants_bytes,
            );

            // Dispatch compute shader
            let workgroup_size = 256;
            let num_workgroups = (bbox_count + workgroup_size - 1) / workgroup_size;
            self.device
                .cmd_dispatch(command_buffer, num_workgroups, 1, 1);
        }
    }

    pub fn descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.descriptor_set_layout
    }

    pub fn descriptor_set(&self) -> vk::DescriptorSet {
        self.descriptor_set
    }
}

impl Drop for TextPicker {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.compute_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            // We let the implicit drop handler for Arc<Device> handle the device cleanup
            // The descriptor pool will be destroyed when the device is destroyed
        }
    }
}
