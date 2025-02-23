use crate::error::{Result, VulkanError};
use crate::text::vertex::TextVertex;
use ash::{vk, Device};
use std::sync::Arc;

pub struct Pipeline {
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    device: Arc<Device>,
    extent: vk::Extent2D,
}

impl Pipeline {
    pub fn new(
        device: Arc<Device>,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
        shader_stages: &[vk::PipelineShaderStageCreateInfo],
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
    ) -> Result<Self> {
        log::debug!(
            "Creating graphics pipeline for extent: {}x{}",
            extent.width,
            extent.height
        );

        // Dynamic state
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        // Vertex input state
        let binding_description = TextVertex::get_binding_description();
        let binding_descriptions = [binding_description];
        let attribute_descriptions_array = TextVertex::get_attribute_descriptions();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions_array);

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build();

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(extent)
            .build();

        let viewports = [viewport];
        let scissors = [scissor];

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // Enable alpha blending
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .build();

        let color_blend_attachments = [color_blend_attachment];

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        log::debug!("Creating pipeline layout with descriptor set layouts");
        let layout_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(descriptor_set_layouts);

        let layout = unsafe {
            device
                .create_pipeline_layout(&layout_info, None)
                .map_err(|e| VulkanError::PipelineLayoutCreation(e.to_string()))?
        };

        log::debug!("Creating graphics pipeline");
        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampling_info)
            .color_blend_state(&color_blend_info)
            .dynamic_state(&dynamic_state)
            .layout(layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[pipeline_info.build()],
                    None,
                )
                .map_err(|e| VulkanError::PipelineCreation(e.1.to_string()))?[0]
        };

        log::debug!("Pipeline created successfully");
        Ok(Self {
            pipeline,
            layout,
            device,
            extent,
        })
    }

    pub fn bind(&self, command_buffer: vk::CommandBuffer) {
        log::debug!(
            "Binding graphics pipeline with viewport size: {}x{}",
            self.extent.width,
            self.extent.height
        );

        unsafe {
            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            // Set dynamic viewport and scissor using stored extent
            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.extent.width as f32,
                height: self.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            self.device.cmd_set_viewport(command_buffer, 0, &[viewport]);

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.extent,
            };
            self.device.cmd_set_scissor(command_buffer, 0, &[scissor]);
        }
        log::debug!("Pipeline binding complete");
    }

    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    pub fn pipeline(&self) -> vk::Pipeline {
        self.pipeline
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
